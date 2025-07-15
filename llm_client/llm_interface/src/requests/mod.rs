// Internal modules
pub mod common;
mod completion;
pub mod error;
mod logit_bias;
mod req_components;
mod res_components;
mod stop_sequence;

// Public exports
pub use error::CompletionError;
pub use completion::{
    CompletionFinishReason, CompletionRequest, CompletionRequestConfig,
    CompletionResponse, CompletionResponseChunk,
};
pub use logit_bias::{LlamaCppLogitBias, LogitBias, LogitBiasTrait, OpenAiLogitBias};
pub use req_components::{RequestConfig, RequestConfigTrait};
pub use completion::GenerationSettings;
pub use res_components::{
    InferenceProbabilities, TimingUsage, TokenUsage, TopProbabilities,
};
pub use stop_sequence::{StopSequences, StoppingSequence};
