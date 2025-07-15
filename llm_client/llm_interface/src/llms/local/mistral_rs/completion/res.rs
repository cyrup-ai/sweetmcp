use crate::requests::*;
use crate::llms::local::mistral_rs::tool_use::from_mistral_tool_call;
use mistralrs::CompletionResponse as MistralCompletionResponse;

impl CompletionResponse {
    #[cfg(feature = "mistral_rs_backend")]
    pub fn new_from_mistral(
        req: &CompletionRequest,
        res: MistralCompletionResponse,
    ) -> Result<Self, CompletionError> {
        let choice = if res.choices.is_empty() || res.choices[0].text.is_empty() {
            return Err(CompletionError::ReponseContentEmpty);
        } else {
            &res.choices[0]
        };
        let finish_reason = match choice.finish_reason.as_str() {
            "stop" => CompletionFinishReason::Eos,
            "length" => CompletionFinishReason::StopLimit,
            "tool_calls" => CompletionFinishReason::ToolCalls,
            _ => {
                return Err(CompletionError::StopReasonUnsupported(
                    "No stop reason provided".to_owned(),
                ))
            }
        };
        
        // Process tool calls if present
        let tool_calls = if let Some(ref mistral_tool_calls) = choice.tool_calls {
            if !mistral_tool_calls.is_empty() {
                // Convert mistral tool calls to common format
                Some(mistral_tool_calls.iter().map(from_mistral_tool_call).collect())
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            id: "mistral_rs".to_owned(),
            index: None,
            content: choice.text.to_owned(),
            finish_reason,
            completion_probabilities: None,
            truncated: false,
            generation_settings: GenerationSettings::new_from_mistral(req, &res),
            timing_usage: TimingUsage::new_from_mistral(&res, req.start_time),
            token_usage: TokenUsage::new_from_mistral(&res),
            tool_calls,
        })
    }
}
