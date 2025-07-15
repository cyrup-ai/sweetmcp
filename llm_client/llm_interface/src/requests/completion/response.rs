// Internal modules
use crate::requests::{
    common::tools::ToolCallSummary, // Import ToolCallSummary
    completion::settings::GenerationSettings,
    res_components::{InferenceProbabilities, TimingUsage, TokenUsage},
    stop_sequence::StoppingSequence,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompletionResponse {
    /// A unique identifier for the chat completion.
    pub id: String,
    /// If batched, the index of the choice in the list of choices.
    pub index: Option<u32>,
    /// The generated completion.
    pub content: String,
    pub finish_reason: CompletionFinishReason,
    #[serde(skip_serializing_if = "Option::is_none")] // Added serde attribute
    pub completion_probabilities: Option<Vec<InferenceProbabilities>>,
    /// True if the context size was exceeded during generation.
    pub truncated: bool,
    /// Details about the tool calls requested by the model, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<crate::requests::common::tools::ToolCall>>,
    pub generation_settings: GenerationSettings, // Use the imported GenerationSettings
    pub timing_usage: TimingUsage,
    pub token_usage: TokenUsage,
}

impl std::fmt::Display for CompletionResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        writeln!(f, "CompletionResponse:")?;
        writeln!(f, "    id: {}", self.id)?;
        writeln!(f, "    index: {:?}", self.index)?;
        writeln!(f, "    content: {:?}", self.content)?;
        writeln!(f, "    finish_reason: {}", self.finish_reason)?;
        if let Some(calls) = &self.tool_calls {
            writeln!(f, "    tool_calls:")?;
            for call in calls {
                writeln!(
                    f,
                    "      - id: {}, name: {}, args: {}",
                    call.id, call.name, call.arguments
                )?;
            }
        }
        writeln!(f, "    truncated: {}", self.truncated)?;
        // Note: Display for GenerationSettings, TimingUsage, TokenUsage needs implementation in their respective files
        // For now, just indicate their presence or use Debug print
        writeln!(f, "    generation_settings: {:?}", self.generation_settings)?; // Using Debug for now
        writeln!(f, "    timing_usage: {:?}", self.timing_usage)?; // Using Debug for now
        write!(f, "    token_usage: {:?}", self.token_usage)?; // Using Debug for now
                                                               // Ensure the Display impls for the sub-structs are updated for better output later
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CompletionFinishReason {
    /// The completion finished because the model generated the EOS token.
    #[serde(rename = "eos")] // Explicit rename if needed, snake_case should handle it
    Eos,
    /// The completion finished because the model generated a stop sequence that matches one of the provided stop sequences.
    MatchingStoppingSequence(StoppingSequence),
    /// The completion finished because the model generated a stop sequence that does not match any of the provided stop sequences.
    NonMatchingStoppingSequence(Option<String>),
    /// The completion finished because the model reached the maximum token limit.
    #[serde(rename = "stop_limit")] // Explicit rename if needed
    StopLimit,
    /// The completion finished because the model wants to use a tool.
    /// Contains summaries of the requested tool calls if available.
    ToolUse(Option<Vec<ToolCallSummary>>),
    /// Alias for ToolUse to maintain compatibility with OpenAI's API
    #[serde(rename = "tool_calls")]
    ToolCall(Option<Vec<ToolCallSummary>>),
}

// ToolCallSummary is now imported from common::tools

impl std::fmt::Display for CompletionFinishReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompletionFinishReason::Eos => write!(f, "Eos"),
            CompletionFinishReason::MatchingStoppingSequence(seq) => {
                write!(f, "MatchingStoppingSequence({})", seq.as_str())
            }
            CompletionFinishReason::NonMatchingStoppingSequence(seq) => {
                write!(f, "NonMatchingStoppingSequence({:?})", seq)
            }
            CompletionFinishReason::StopLimit => write!(f, "StopLimit"),
            CompletionFinishReason::ToolUse(calls) => write!(f, "ToolUse({:?})", calls), // Updated display
            CompletionFinishReason::ToolCall(calls) => write!(f, "ToolCall({:?})", calls), // Also handle ToolCall variant
        }
    }
}
