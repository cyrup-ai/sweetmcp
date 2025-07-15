use crate::{local_models::{hf_loader::HuggingFaceLoader, metadata::LocalLlmMetadata, LocalLlmModel}, LlmModelBase, Error, LlmChatTemplate}; // Import Error
use crate::local_models::gguf::load_chat_template; // Import from gguf module
#[cfg(feature = "model-tokenizers")]
use crate::local_models::gguf::load_tokenizer; // Import feature-gated function
use tracing::error;

#[derive(Default, Clone)]
pub struct GgufHfLoader {
    pub hf_quant_file_url: Option<String>,
    pub hf_tokenizer_repo_id: Option<String>,
    pub model_id: Option<String>,
}

impl GgufHfLoader {
    pub fn load(&mut self, hf_loader: &HuggingFaceLoader) -> crate::Result<LocalLlmModel> {
        let hf_quant_file_url = self.hf_quant_file_url.as_ref().ok_or_else(|| {
            Error::Config("GgufHfLoader requires 'hf_quant_file_url' to be set".to_string())
        })?;

        let (model_id, repo_id, gguf_model_filename) =
            HuggingFaceLoader::parse_full_model_url(hf_quant_file_url)?;

        let local_model_path =
            hf_loader.load_file(&gguf_model_filename, &repo_id)?; // Load the GGUF file
        let local_model_path = HuggingFaceLoader::canonicalize_local_path(local_model_path)?; // Canonicalize path

        #[cfg(feature = "model-tokenizers")]
        let local_tokenizer_path = if let Some(hf_tokenizer_repo_id) = &self.hf_tokenizer_repo_id {
            match hf_loader.load_file("tokenizer.json", hf_tokenizer_repo_id.clone()) { // Clone repo_id if needed
                Ok(path) => Some(HuggingFaceLoader::canonicalize_local_path(path)?), // Canonicalize tokenizer path too
                Err(e) => {
                    error!("Failed to load tokenizer.json from repo '{}': {}. Will attempt to use tokenizer from GGUF.", hf_tokenizer_repo_id, e);
                    None
                }
            }
        } else {
            None
        };

        let model_metadata = LocalLlmMetadata::from_gguf_path(&local_model_path)?;

        Ok(LocalLlmModel {
            model_base: LlmModelBase {
                friendly_name: model_metadata
                    .general
                    .name
                    .clone()
                    .unwrap_or(model_id.clone()),
                model_id: model_id.clone(),
                model_ctx_size: model_metadata.context_length(),
                inference_ctx_size: model_metadata.context_length(),
                #[cfg(feature = "model-tokenizers")]
                tokenizer: load_tokenizer( // Use imported function
                    &local_tokenizer_path, // Pass &Option<PathBuf>
                    &model_metadata,
                )?,
            },
            chat_template: match load_chat_template(&model_metadata) {
                Ok(Some(template)) => template,
                _ => LlmChatTemplate::default(),
            }, // Use cleaner pattern
            model_metadata, // Use shorthand field init
            local_model_path,
        })
    }
}
