use crate::LlmClient;
use llm_devices::{LoggingConfig, LoggingConfigTrait};
use llm_interface::llms::{LlmBackend, api::{
    ApiConfig, LlmApiConfigTrait, 
    openai::{OpenAiBackend, OpenAiConfig}
}};
use llm_models::{
    api_models::{ApiLlmModel, OpenAiModelTrait},
    ApiLlmPreset,
};

pub struct OpenAiBackendBuilder {
    pub config: OpenAiConfig,
    pub model: ApiLlmModel,
}

impl Default for OpenAiBackendBuilder {
    fn default() -> Self {
        // Handle potential error during default model creation
        let model = ApiLlmModel::from_preset(ApiLlmPreset::GPT_4O_MINI)
            .expect("Failed to create default OpenAI model from preset");
            
        Self {
            config: Default::default(),
            model,
        }
    }
}

impl OpenAiBackendBuilder {
    pub fn init(self) -> crate::Result<LlmClient> {
        Ok(LlmClient::new(std::sync::Arc::new(LlmBackend::OpenAi(
            OpenAiBackend::new(self.config, self.model)?,
        ))))
    }
}

impl LlmApiConfigTrait for OpenAiBackendBuilder {
    fn api_base_config_mut(&mut self) -> &mut ApiConfig {
        &mut self.config.api_config
    }

    fn api_config(&self) -> &ApiConfig {
        &self.config.api_config
    }
}

impl OpenAiModelTrait for OpenAiBackendBuilder {
    fn model(&mut self) -> &mut ApiLlmModel {
        &mut self.model
    }
}

impl LoggingConfigTrait for OpenAiBackendBuilder {
    fn logging_config_mut(&mut self) -> &mut LoggingConfig {
        &mut self.config.logging_config
    }
}
