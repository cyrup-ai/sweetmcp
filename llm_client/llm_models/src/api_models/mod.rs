// Public generated modules
pub mod models;
pub mod providers;

// Public exports
pub use self::{
    models::ApiLlmPreset,
    providers::{AnthropicModelTrait, ApiLlmProvider, OpenAiModelTrait, PerplexityModelTrait},
};

// Internal imports
use super::LlmModelBase;
use crate::Error;
use std::sync::Arc;

// Feature-specific internal imports
#[cfg(feature = "model-tokenizers")]
use crate::tokenizer::LlmTokenizer;

#[derive(Clone)]
pub struct ApiLlmModel {
    pub model_base: LlmModelBase,
    pub provider: ApiLlmProvider,
    pub cost_per_m_in_tokens: usize,
    pub cost_per_m_out_tokens: usize, // to usize
    pub tokens_per_message: usize,
    pub tokens_per_name: Option<isize>,
}

impl Default for ApiLlmModel {
    fn default() -> Self {
        ApiLlmModel::from_preset(ApiLlmPreset::CLAUDE_3_5_SONNET).unwrap()
    }
}


impl ApiLlmModel {
    pub fn from_preset(preset: ApiLlmPreset) -> crate::Result<Self> {
        #[cfg(feature = "model-tokenizers")]
        let tokenizer_result = preset.provider.model_tokenizer(preset.model_id);

        // Handle potential tokenizer loading error without anyhow::Context
        #[cfg(feature = "model-tokenizers")]
        let tokenizer = tokenizer_result.map_err(|e| {
            Error::Tokenizer(format!(
                "Failed to load tokenizer for preset model_id '{}': {}",
                preset.model_id, e
            ))
        })?;

        Ok(Self {
            model_base: LlmModelBase {
                #[cfg(feature = "model-tokenizers")]
                tokenizer, // Use the loaded tokenizer
                model_id: preset.model_id.to_string(),
                friendly_name: preset.friendly_name.to_string(),
                model_ctx_size: preset.model_ctx_size,
                inference_ctx_size: preset.inference_ctx_size,
            },
            provider: preset.provider,
            cost_per_m_in_tokens: preset.cost_per_m_in_tokens,
            cost_per_m_out_tokens: preset.cost_per_m_out_tokens,
            tokens_per_message: preset.tokens_per_message,
            tokens_per_name: preset.tokens_per_name,
        })
    }
}

impl ApiLlmProvider {
    #[cfg(feature = "model-tokenizers")]
    fn model_tokenizer(&self, model_id: &str) -> crate::Result<Arc<LlmTokenizer>> {
        match self {
            ApiLlmProvider::Anthropic => {
                tracing::warn!("Anthropic does not have a publicly available tokenizer. Using TikToken 'gpt-4' tokenizer for token counting estimation. See: https://github.com/javirandor/anthropic-tokenizer");
                // Use ? to propagate potential errors from new_tiktoken
                LlmTokenizer::new_tiktoken("gpt-4").map(Arc::new)
            }
            ApiLlmProvider::OpenAi => {
                // Use ? to propagate potential errors from new_tiktoken
                LlmTokenizer::new_tiktoken(model_id).map(Arc::new)
            }
            ApiLlmProvider::Perplexity => {
                tracing::warn!("Perplexity tokenizer details are not explicitly public. Using TikToken 'gpt-4' tokenizer for token counting estimation.");
                // Use ? to propagate potential errors from new_tiktoken
                LlmTokenizer::new_tiktoken("gpt-4").map(Arc::new)
            }
        }
    }
}
