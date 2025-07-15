use crate::requests::*;
use serde::{Deserialize, Serialize};
use crate::llms::api::openai::tool_use::ToolCall;

impl CompletionResponse {
    pub fn new_from_openai(
        req: &CompletionRequest,
        res: OpenAiCompletionResponse,
    ) -> Result<Self, CompletionError> {
        let choice = if res.choices.is_empty() {
            return Err(CompletionError::ReponseContentEmpty);
        } else {
            &res.choices[0]
        };
        
        // Handle tool calls if present
        if let Some(ref tool_calls) = choice.message.tool_calls {
            // First create the ToolCall objects
            let common_tool_calls: Vec<crate::requests::common::tools::ToolCall> = tool_calls
                .iter()
                .map(|call| crate::requests::common::tools::ToolCall {
                    id: call.id.clone(),
                    name: call.function.name.clone(),
                    arguments: serde_json::from_str(&call.function.arguments).unwrap_or_else(|_| serde_json::Value::Null),
                })
                .collect();
                
            // Convert to ToolCallSummary objects for the response
            let tool_call_summaries: Vec<crate::requests::common::tools::ToolCallSummary> = common_tool_calls
                .iter()
                .map(|call| crate::requests::common::tools::ToolCallSummary {
                    name: call.name.clone(),
                    input: call.arguments.clone(),
                })
                .collect();

            let finish_reason = match choice.finish_reason {
                Some(FinishReason::ToolCalls) => CompletionFinishReason::ToolCall(Some(tool_call_summaries)),
                Some(FinishReason::Stop) => CompletionFinishReason::Eos,
                Some(FinishReason::Length) => CompletionFinishReason::StopLimit,
                Some(FinishReason::ContentFilter) => {
                    return Err(CompletionError::StopReasonUnsupported(
                        "FinishReason::ContentFilter is not supported".to_owned(),
                    ))
                }
                Some(FinishReason::FunctionCall) => CompletionFinishReason::ToolCall(Some(tool_call_summaries.clone())), // Treat as tool call
                None => CompletionFinishReason::Eos,
            };

            return Ok(Self {
                id: res.id.to_owned(),
                index: None,
                // With tool calls, content might be None
                content: choice.message.content.clone().unwrap_or_default(),
                finish_reason,
                completion_probabilities: None,
                truncated: false,
                generation_settings: GenerationSettings::new_from_openai(req, &res),
                timing_usage: TimingUsage::new_from_generic(req.start_time),
                token_usage: TokenUsage::new_from_generic(&res),
                tool_calls: Some(common_tool_calls),
            });
        }

        // Handle regular responses
        if choice.message.content.is_none() {
            return Err(CompletionError::ReponseContentEmpty);
        }
        
        let finish_reason = match choice.finish_reason {
            Some(FinishReason::Stop) => CompletionFinishReason::Eos,
            Some(FinishReason::Length) => CompletionFinishReason::StopLimit,
            Some(FinishReason::ToolCalls) => CompletionFinishReason::ToolCall(None), // Should not happen here
            Some(FinishReason::ContentFilter) => {
                return Err(CompletionError::StopReasonUnsupported(
                    "FinishReason::ContentFilter is not supported".to_owned(),
                ))
            }
            Some(FinishReason::FunctionCall) => CompletionFinishReason::ToolCall(None), // Treat as tool call
            None => CompletionFinishReason::Eos,
        };
        
        Ok(Self {
            id: res.id.to_owned(),
            index: None,
            content: choice.message.content.as_ref().unwrap().to_owned(),
            finish_reason,
            completion_probabilities: None,
            truncated: false,
            generation_settings: GenerationSettings::new_from_openai(req, &res),
            timing_usage: TimingUsage::new_from_generic(req.start_time),
            token_usage: TokenUsage::new_from_generic(&res),
            tool_calls: None,
        })
    }
}

/// Represents a chat completion response returned by model, based on the provided input.
#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
pub struct OpenAiCompletionResponse {
    /// A unique identifier for the chat completion.
    pub id: String,
    /// A list of chat completion choices. Can be more than one if `n` is greater than 1.
    pub choices: Vec<ChatChoice>,
    /// The Unix timestamp (in seconds) of when the chat completion was created.
    pub created: u32,
    /// The model used for the chat completion.
    pub model: String,
    pub usage: Option<CompletionUsage>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ChatChoice {
    /// The index of the choice in the list of choices.
    pub index: u32,
    pub message: ChatCompletionResponseMessage,
    /// The reason the model stopped generating tokens. This will be `stop` if the model hit a natural stop point or a provided stop sequence,
    /// `length` if the maximum number of tokens specified in the request was reached,
    /// `content_filter` if content was omitted due to a flag from our content filters,
    /// `tool_calls` if the model called a tool, or `function_call` (deprecated) if the model called a function.
    pub finish_reason: Option<FinishReason>,
    /// Log probability information for the choice.
    pub logprobs: Option<ChatChoiceLogprobs>,
}

/// Usage statistics for the completion request.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct CompletionUsage {
    /// Number of tokens in the prompt.
    pub prompt_tokens: usize,
    /// Number of tokens in the generated completion.
    pub completion_tokens: usize,
    /// Total number of tokens used in the request (prompt + completion).
    pub total_tokens: usize,
}

/// A chat completion message generated by the model.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ChatCompletionResponseMessage {
    /// The contents of the message.
    pub content: Option<String>,

    /// The role of the author of this message.
    pub role: Role,
    
    /// The tool calls made by the assistant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
    ContentFilter,
    FunctionCall,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ChatChoiceLogprobs {
    /// A list of message content tokens with log probability information.
    pub content: Option<Vec<ChatCompletionTokenLogprob>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ChatCompletionTokenLogprob {
    /// The token.
    pub token: String,
    /// The log probability of this token, if it is within the top 20 most likely tokens. Otherwise, the value `-9999.0` is used to signify that the token is very unlikely.
    pub logprob: f32,
    /// A list of integers representing the UTF-8 bytes representation of the token. Useful in instances where characters are represented by multiple tokens and their byte representations must be combined to generate the correct text representation. Can be `null` if there is no bytes representation for the token.
    pub bytes: Option<Vec<u8>>,
    ///  List of the most likely tokens and their log probability, at this token position. In rare cases, there may be fewer than the number of requested `top_logprobs` returned.
    pub top_logprobs: Vec<TopLogprobs>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct TopLogprobs {
    /// The token.
    pub token: String,
    /// The log probability of this token.
    pub logprob: f32,
    /// A list of integers representing the UTF-8 bytes representation of the token. Useful in instances where characters are represented by multiple tokens and their byte representations must be combined to generate the correct text representation. Can be `null` if there is no bytes representation for the token.
    pub bytes: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    #[default]
    User,
    Assistant,
    Tool,
    Function,
}
