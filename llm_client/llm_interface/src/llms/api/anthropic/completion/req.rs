use crate::llms::api::anthropic::tool_use::{Tool, ToolChoice, ToolResultBlock};
use crate::requests::{
    common::tools as common_tools,
    CompletionRequest, CompletionError,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Define prefix conventions for special content within the prompt history string
const TOOL_RESULT_PREFIX: &str = "TOOL_RESULT:::";
const TOOL_USE_REQUEST_PREFIX: &str = "TOOL_USE_REQUEST:::";

/// Represents the `tool_use` content block structure for requests.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ToolUseBlock {
    #[serde(rename = "type")]
    pub type_: String, // Should always be "tool_use"
    pub id: String, // Ensure this matches the structure expected by serialization/deserialization
    pub name: String,
    pub input: Value,
}

#[derive(Clone, Serialize, Default, Debug, Deserialize, PartialEq)]
pub struct AnthropicCompletionRequest {
    /// ID of the model to use.
    ///
    /// See [models](https://docs.anthropic.com/claude/docs/models-overview) for additional details and options.
    pub model: String,

    /// Input messages.
    ///
    /// Our models are trained to operate on alternating user and assistant conversational turns. When creating a new Message, you specify the prior conversational turns with the messages parameter, and the model then generates the next Message in the conversation.
    ///
    /// See [examples](https://docs.anthropic.com/claude/reference/messages-examples) for more input examples.
    ///
    /// Note that if you want to include a [system prompt](https://docs.anthropic.com/claude/docs/system-prompts), you can use the top-level system parameter — there is no "system" role for input messages in the Messages API.
    pub messages: Vec<AnthropicCompletionRequestMessage>, // Use specific type

    /// The maximum number of tokens to generate before stopping.
    ///
    /// Note that our models may stop before reaching this maximum. This parameter only specifies the absolute maximum number of tokens to generate.
    ///
    /// Different models have different maximum values for this parameter. See [models](https://docs.anthropic.com/claude/docs/models-overview) for details.
    pub max_tokens: usize,

    /// Custom text sequences that will cause the model to stop generating.
    ///
    /// Our models will normally stop when they have naturally completed their turn, which will result in a response stop_reason of "end_turn".
    ///
    /// If you want the model to stop generating when it encounters custom strings of text, you can use the stop_sequences parameter. If the model encounters one of the custom sequences, the response stop_reason value will be "stop_sequence" and the response stop_sequence value will contain the matched stop sequence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,

    /// System prompt.
    ///
    /// A system prompt is a way of providing context and instructions to Claude, such as specifying a particular goal or role. See our [guide to system prompts](https://docs.anthropic.com/claude/docs/system-prompts).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,

    /// Amount of randomness injected into the response.
    ///
    /// Defaults to 0.5. Ranges from 0.0 to 1.0. Use temperature closer to 0.0 for analytical / multiple choice, and closer to 1.0 for creative and generative tasks.
    ///
    /// Note that even with temperature of 0.0, the results will not be fully deterministic.
    pub temperature: f32,

    /// min: 0.0, max: 1.0, default: None
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Use nucleus sampling. In nucleus sampling, we compute the cumulative distribution over all the options for each subsequent token in decreasing probability order and cut it off once it reaches a particular probability specified by top_p. You should either alter temperature or top_p, but not both. Recommended for most use cases.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<usize>,

    /// Definitions of tools that the model may use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    /// How the model should use the provided tools.
    /// If `tools` are provided, defaults to `ToolChoice::Auto`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    /// Whether to incrementally stream the response using server-sent events.
    ///
    /// See [streaming](https://docs.anthropic.com/claude/reference/messages-streaming) for details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

impl AnthropicCompletionRequest {
    /// Creates a new AnthropicCompletionRequest from a generic CompletionRequest.
    /// Handles mapping of prompt messages, tools, and configuration.
    pub fn new(req: &CompletionRequest) -> crate::Result<Self, CompletionError> {
        let mut messages = Vec::new();
        let mut system_prompt = None;
        match &req
            .prompt
            .api_prompt()
            .map_err(|e| CompletionError::RequestBuilderError(e.to_string()))?
            .get_built_prompt()
        {
            Ok(prompt_message) => {
                for m in prompt_message {
                    let role = m.get("role").ok_or_else(|| {
                        CompletionError::RequestBuilderError("Role not found".to_string())
                    })?;
                    let content = m.get("content").ok_or_else(|| {
                        CompletionError::RequestBuilderError("Content not found".to_string())
                    })?;

                    match role.as_str() {
                        "user" => {
                            // Check for the TOOL_RESULT prefix convention
                            if let Some(json_str) = content.strip_prefix(TOOL_RESULT_PREFIX) {
                                // Attempt to deserialize as Vec<ToolResultBlock>
                                match serde_json::from_str::<Vec<ToolResultBlock>>(json_str) {
                                    Ok(tool_results) => messages.push(
                                        AnthropicCompletionRequestMessage::user_tool_results(
                                            tool_results,
                                        ),
                                    ),
                                    Err(e) => {
                                        return Err(CompletionError::RequestBuilderError(format!(
                                            "Failed to deserialize tool result JSON: {}. Content: '{}'",
                                            e, content
                                        )));
                                    }
                                }
                            } else {
                                // Treat as regular text content
                                messages.push(AnthropicCompletionRequestMessage::user_text(
                                    content.to_string(),
                                ));
                            }
                        }
                        "assistant" => {
                            // Check if the content represents a tool use request
                            if let Some(json_str) = content.strip_prefix(TOOL_USE_REQUEST_PREFIX) {
                                // Attempt to deserialize as Vec<ToolUseBlock>
                                match serde_json::from_str::<Vec<ToolUseBlock>>(json_str) {
                                    Ok(tool_uses) => {
                                        // Ensure the type field is correctly set
                                        let validated_tool_uses = tool_uses
                                            .into_iter()
                                            .map(|mut tu| {
                                                tu.type_ = "tool_use".to_string(); // Ensure type is correct
                                                tu
                                            })
                                            .collect();
                                        messages.push(
                                            AnthropicCompletionRequestMessage::assistant_tool_use(
                                                validated_tool_uses,
                                            ),
                                        );
                                    }
                                    Err(e) => {
                                        return Err(CompletionError::RequestBuilderError(format!(
                                            "Failed to deserialize assistant tool use JSON: {}. Content: '{}'",
                                            e, content
                                        )));
                                    }
                                }
                            } else {
                                // Treat as regular text content
                                messages.push(AnthropicCompletionRequestMessage::assistant_text(
                                    content.to_string(),
                                ));
                            }
                        }
                        "system" => system_prompt = Some(content.to_string()), // System prompt handled separately

                        _ => {
                            return Err(CompletionError::RequestBuilderError(format!(
                                "Role {} not supported",
                                role
                            )))
                        }
                    }
                }
            }
            Err(e) => {
                return Err(CompletionError::RequestBuilderError(format!(
                    "Error building prompt: {}",
                    e
                )))
            }
        }

        let stop = req.stop_sequences.to_vec();
        let stop_sequences = if stop.is_empty() { None } else { Some(stop) };

        // Map generic tools to Anthropic specific tools
        let tools: Option<Vec<Tool>> = req.config.tools.as_ref().map(|generic_tools| {
            generic_tools
                .iter()
                .map(|gt| Tool {
                    name: gt.name.clone(),
                    description: gt.description.clone(),
                    input_schema: gt.input_schema.clone(),
                })
                .collect()
        });

        // Map generic tool choice to Anthropic specific tool choice
        let tool_choice: Option<ToolChoice> =
            req.config.tool_choice.as_ref().and_then(|generic_choice| {
                match generic_choice {
                    common_tools::ToolChoice::Auto => Some(ToolChoice::Auto),
                    common_tools::ToolChoice::Any => Some(ToolChoice::Any),
                    common_tools::ToolChoice::Tool { name } => {
                        Some(ToolChoice::Tool { name: name.clone() })
                    }
                    // Anthropic represents 'None' by omitting the field, so map it to None
                    common_tools::ToolChoice::None => None,
                }
            });

        // Default to Auto if tools are provided but no choice is specified
        let tool_choice =
            if tools.is_some() && tool_choice.is_none() && req.config.tool_choice.is_none() {
                Some(ToolChoice::Auto)
            } else {
                tool_choice
            };

        // Set stream flag if requested
        let stream = if req.config.stream_response {
            Some(true)
        } else {
            None
        };

        Ok(AnthropicCompletionRequest {
            model: req.backend.model_id().to_owned(),
            messages,
            max_tokens: req.config.actual_request_tokens.unwrap(),
            stop_sequences,
            system: system_prompt,
            temperature: temperature(req.config.temperature)?,
            top_p: top_p(req.config.top_p)?,
            top_k: top_k(req.config.top_k)?,
            tools,
            tool_choice,
            stream, // Set based on req.config.stream_response
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llms::LlmBackend; // Assuming LlmBackend is accessible
    use crate::requests::common::tools::{ToolChoice as CommonToolChoice, ToolDefinition};
    use llm_models::ApiLlmModel;
    use llm_prompt::LlmPrompt;
    use std::sync::Arc;

    // Helper to create a basic CompletionRequest for testing
    fn create_test_request(
        backend: Arc<LlmBackend>,
        temperature: f32,
        prompt_modifier: impl FnOnce(LlmPrompt) -> LlmPrompt,
    ) -> CompletionRequest {
        // Create a new request with standard parameters
        let mut req = CompletionRequest::new(backend);
        
        // Only modify publicly available settings through the proper API
        req.set_temperature(temperature);
        req.prompt = prompt_modifier(req.prompt);
        
        req
    }

    // Helper to create a dummy Anthropic backend Arc
    fn dummy_anthropic_backend() -> Arc<LlmBackend> {
        // We don't need a real client, just the config/model part for request building
        let config = crate::llms::api::anthropic::AnthropicConfig::default();
        let model = ApiLlmModel::default(); // Use default Claude 3.5 Sonnet
        let backend = crate::llms::api::anthropic::AnthropicBackend {
            client: crate::llms::api::ApiClient::new(config), // Dummy client
            model,
        };
        Arc::new(LlmBackend::Anthropic(backend))
    }

    #[test]
    fn test_anthropic_request_new_basic() {
        let backend = dummy_anthropic_backend();
        let req = create_test_request(
            backend,
            |c| c.with_temperature(1.5).with_top_k(50), // Temp 1.5 -> 0.75
            |mut p| {
                p.add_user_message().unwrap().set_content("Hello");
                p
            },
        );

        let anthropic_req = AnthropicCompletionRequest::new(&req).unwrap();

        assert_eq!(anthropic_req.model, "claude-3-5-sonnet-20240620");
        assert_eq!(anthropic_req.temperature, 0.75); // Check temp conversion
        assert_eq!(anthropic_req.top_k, Some(50));
        assert_eq!(anthropic_req.messages.len(), 1);
        assert_eq!(anthropic_req.messages[0].role, "user");
        assert!(
            matches!(anthropic_req.messages[0].content, AnthropicMessageContent::Text(ref t) if t == "Hello")
        );
        assert_eq!(anthropic_req.stream, None);
        assert!(anthropic_req.tools.is_none());
        assert!(anthropic_req.tool_choice.is_none());
    }

    #[test]
    fn test_anthropic_request_new_with_stream() {
        let backend = dummy_anthropic_backend();
        let req = create_test_request(
            backend,
            |c| c.with_stream_response(true),
            |mut p| {
                p.add_user_message().unwrap().set_content("Stream test");
                p
            },
        );

        let anthropic_req = AnthropicCompletionRequest::new(&req).unwrap();
        assert_eq!(anthropic_req.stream, Some(true));
    }

    #[test]
    fn test_anthropic_request_new_with_tools() {
        let backend = dummy_anthropic_backend();
        let tools = vec![ToolDefinition {
            name: "get_weather".to_string(),
            description: "Gets weather".to_string(),
            input_schema: serde_json::json!({"type": "object", "properties": {"location": {"type": "string"}}}),
        }];
        let req = create_test_request(
            backend,
            |c| {
                c.with_tools(tools.clone())
                    .with_tool_choice(CommonToolChoice::Any)
            },
            |mut p| {
                p.add_user_message().unwrap().set_content("Weather?");
                p
            },
        );

        let anthropic_req = AnthropicCompletionRequest::new(&req).unwrap();

        assert!(anthropic_req.tools.is_some());
        assert_eq!(anthropic_req.tools.as_ref().unwrap().len(), 1);
        assert_eq!(anthropic_req.tools.as_ref().unwrap()[0].name, "get_weather");
        assert_eq!(anthropic_req.tool_choice, Some(ToolChoice::Any));
    }

    #[test]
    fn test_anthropic_request_new_with_tools_auto_default() {
        let backend = dummy_anthropic_backend();
        let tools = vec![ToolDefinition {
            name: "get_weather".to_string(),
            description: "Gets weather".to_string(),
            input_schema: serde_json::json!({"type": "object", "properties": {"location": {"type": "string"}}}),
        }];
        // No explicit tool_choice set
        let req = create_test_request(
            backend,
            |c| c.with_tools(tools.clone()),
            |mut p| {
                p.add_user_message().unwrap().set_content("Weather?");
                p
            },
        );

        let anthropic_req = AnthropicCompletionRequest::new(&req).unwrap();

        assert!(anthropic_req.tools.is_some());
        // Should default to Auto when tools are present but choice isn't specified
        assert_eq!(anthropic_req.tool_choice, Some(ToolChoice::Auto));
    }

    #[test]
    fn test_anthropic_request_new_tool_choice_none() {
        let backend = dummy_anthropic_backend();
        let tools = vec![/* ... tools ... */]; // Tools might still be defined
        let req = create_test_request(
            backend,
            |c| {
                c.with_tools(tools.clone())
                    .with_tool_choice(CommonToolChoice::None)
            },
            |mut p| {
                p.add_user_message().unwrap().set_content("Don't use tools");
                p
            },
        );

        let anthropic_req = AnthropicCompletionRequest::new(&req).unwrap();

        assert!(anthropic_req.tools.is_some()); // Tools are sent
        assert!(anthropic_req.tool_choice.is_none()); // But choice is omitted for 'None'
    }

    #[test]
    fn test_anthropic_request_history_conventions() {
        let backend = dummy_anthropic_backend();

        // Simulate history stored with conventions
        let tool_result_json = serde_json::to_string(&vec![ToolResultBlock {
            block_type: "tool_result".to_string(),
            tool_use_id: "toolu_1".to_string(),
            content: Some("15 degrees".to_string()),
            is_error: None,
        }])
        .unwrap();

        let tool_use_json = serde_json::to_string(&vec![ToolUseBlock {
            type_: "tool_use".to_string(),
            id: "toolu_1".to_string(),
            name: "get_weather".to_string(),
            input: serde_json::json!({"location": "SFO"}),
        }])
        .unwrap();

        let req = create_test_request(
            backend,
            |c| c,
            |mut p| {
                p.add_user_message().unwrap().set_content("Weather?");
                // Assistant requested tool use
                p.add_assistant_message()
                    .unwrap()
                    .set_content(format!("{}{}", TOOL_USE_REQUEST_PREFIX, tool_use_json));
                // User provided tool result
                p.add_user_message()
                    .unwrap()
                    .set_content(format!("{}{}", TOOL_RESULT_PREFIX, tool_result_json));
                p
            },
        );

        let anthropic_req = AnthropicCompletionRequest::new(&req).unwrap();

        assert_eq!(anthropic_req.messages.len(), 3);
        // First user message
        assert!(
            matches!(anthropic_req.messages[0].content, AnthropicMessageContent::Text(ref t) if t == "Weather?")
        );
        // Assistant message (should be ToolUse block)
        assert!(
            matches!(&anthropic_req.messages[1].content, AnthropicMessageContent::ToolUse(ref v) if v.len() == 1 && v[0].id == "toolu_1")
        );
        // Second user message (should be ToolResult block)
        assert!(
            matches!(&anthropic_req.messages[2].content, AnthropicMessageContent::ToolResults(ref v) if v.len() == 1 && v[0].tool_use_id == "toolu_1")
        );
    }

    #[test]
    fn test_anthropic_request_invalid_history_json() {
        let backend = dummy_anthropic_backend();
        let req = create_test_request(
            backend,
            |c| c,
            |mut p| {
                p.add_user_message().unwrap().set_content("Weather?");
                // Malformed JSON for tool result
                p.add_user_message()
                    .unwrap()
                    .set_content(format!("{}{}", TOOL_RESULT_PREFIX, "{invalid json"));
                p
            },
        );

        let result = AnthropicCompletionRequest::new(&req);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CompletionError::RequestBuilderError(_)
        ));
    }
}

/// Convert the native temperature from 0.0 to 2.0 to 0.0 to 1.0
fn temperature(value: f32) -> crate::Result<f32, CompletionError> {
    if (0.0..=2.0).contains(&value) {
        Ok(value / 2.0)
    } else {
        Err(CompletionError::RequestBuilderError(
            "Temperature must be between 0.0 and 2.0".to_string(),
        ))
    }
}

fn top_p(value: Option<f32>) -> crate::Result<Option<f32>, CompletionError> {
    match value {
        Some(v) => {
            if (0.0..=1.0).contains(&v) {
                Ok(Some(v))
            } else {
                Err(CompletionError::RequestBuilderError(
                    "Top p must be between 0.0 and 1.0".to_string(),
                ))
            }
        }
        None => Ok(None),
    }
}

// Validate top_k (Anthropic doesn't specify a range, but it must be positive if set)
fn top_k(value: Option<usize>) -> crate::Result<Option<usize>, CompletionError> {
    match value {
        Some(k) if k == 0 => Err(CompletionError::RequestBuilderError(
            "Top k must be a positive integer if set.".to_string(),
        )),
        Some(k) => Ok(Some(k)),
        None => Ok(None),
    }
}

/// Represents a message in the Anthropic conversation history.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AnthropicCompletionRequestMessage {
    pub role: String,
    /// Content can be a simple string (for text) or a Vec of content blocks.
    /// Using an enum to represent this structure more accurately for serialization.
    pub content: AnthropicMessageContent,
}

/// Represents the different structures the 'content' field can take in an Anthropic request message.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)] // Allows serializing/deserializing as String, Vec<ToolResultBlock>, or Vec<ToolUseBlock>
pub enum AnthropicMessageContent {
    Text(String),
    ToolResults(Vec<ToolResultBlock>),
    ToolUse(Vec<ToolUseBlock>), // For assistant tool use requests
                                // Anthropic also supports image content blocks for user messages, add later if needed.
}

impl AnthropicCompletionRequestMessage {
    /// Creates a user message with text content.
    pub fn user_text(text: String) -> Self {
        Self {
            role: "user".to_string(),
            content: AnthropicMessageContent::Text(text),
        }
    }

    /// Creates a user message with tool result content blocks.
    pub fn user_tool_results(results: Vec<ToolResultBlock>) -> Self {
        Self {
            role: "user".to_string(),
            content: AnthropicMessageContent::ToolResults(results),
        }
    }

    /// Creates an assistant message with text content.
    pub fn assistant_text(text: String) -> Self {
        Self {
            role: "assistant".to_string(),
            content: AnthropicMessageContent::Text(text),
        }
    }

    /// Creates an assistant message containing tool use request blocks.
    pub fn assistant_tool_use(tool_uses: Vec<ToolUseBlock>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: AnthropicMessageContent::ToolUse(tool_uses),
        }
    }
}
