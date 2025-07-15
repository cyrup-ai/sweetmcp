use super::*; // Keep wildcard import if needed for traits/structs in parent
use crate::Error; // Import local Error type

#[derive(Debug, Clone)]
pub enum ApiLlmProvider {
    Anthropic,
    OpenAi,
    Perplexity,
}
impl ApiLlmProvider {
    pub fn all_providers() -> Vec<Self> {
        vec![Self::Anthropic, Self::OpenAi, Self::Perplexity]
    }
    pub fn all_provider_presets(&self) -> Vec<ApiLlmPreset> {
        match self {
            Self::Anthropic => {
                vec![
                    ApiLlmPreset::CLAUDE_3_OPUS,
                    ApiLlmPreset::CLAUDE_3_SONNET,
                    ApiLlmPreset::CLAUDE_3_HAIKU,
                    ApiLlmPreset::CLAUDE_3_5_SONNET,
                    ApiLlmPreset::CLAUDE_3_5_HAIKU,
                ]
            }
            Self::OpenAi => {
                vec![
                    ApiLlmPreset::GPT_4,
                    ApiLlmPreset::GPT_3_5_TURBO,
                    ApiLlmPreset::GPT_4_32K,
                    ApiLlmPreset::GPT_4_TURBO,
                    ApiLlmPreset::GPT_4O,
                    ApiLlmPreset::GPT_4O_MINI,
                    ApiLlmPreset::O1,
                    ApiLlmPreset::O1_MINI,
                    ApiLlmPreset::O3_MINI,
                ]
            }
            Self::Perplexity => {
                vec![
                    ApiLlmPreset::SONAR_REASONING_PRO,
                    ApiLlmPreset::SONAR_REASONING,
                    ApiLlmPreset::SONAR_PRO,
                    ApiLlmPreset::SONAR,
                ]
            }
        }
    }
    pub fn preset_from_model_id(&self, model_id: &str) -> Result<ApiLlmPreset, crate::Error> {
        let model_id = model_id.to_lowercase();
        let presets = self.all_provider_presets();
        for preset in &presets {
            if preset.model_id.to_lowercase() == model_id {
                return Ok(preset.to_owned());
            }
        }
        for preset in presets {
            if model_id.contains(&preset.model_id.to_lowercase())
                || preset.model_id.contains(&model_id.to_lowercase())
                || preset.friendly_name.to_lowercase().contains(&model_id)
            {
                return Ok(preset);
            }
        }
        Err(Error::Config(format!(
            "Model ID '{}' not found for provider {}",
            model_id,
            self.friendly_name()
        )))
    }
    pub fn friendly_name(&self) -> &str {
        match self {
            Self::Anthropic => "Anthropic",
            Self::OpenAi => "OpenAI",
            Self::Perplexity => "Perplexity",
        }
    }
}
pub trait AnthropicModelTrait {
    fn model(&mut self) -> &mut ApiLlmModel;
    fn model_id_str(mut self, model_id: &str) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let preset = ApiLlmProvider::Anthropic.preset_from_model_id(model_id)?;
        *self.model() = ApiLlmModel::from_preset(preset)?;
        Ok(self)
    }
    fn claude_3_opus(mut self) -> Self
    where
        Self: Sized,
    {
        *self.model() = ApiLlmModel::from_preset(ApiLlmPreset::CLAUDE_3_OPUS).unwrap();
        self
    }
    fn claude_3_sonnet(mut self) -> Self
    where
        Self: Sized,
    {
        *self.model() = ApiLlmModel::from_preset(ApiLlmPreset::CLAUDE_3_SONNET).unwrap();
        self
    }
    fn claude_3_haiku(mut self) -> Self
    where
        Self: Sized,
    {
        *self.model() = ApiLlmModel::from_preset(ApiLlmPreset::CLAUDE_3_HAIKU).unwrap();
        self
    }
    fn claude_3_5_sonnet(mut self) -> Self
    where
        Self: Sized,
    {
        *self.model() = ApiLlmModel::from_preset(ApiLlmPreset::CLAUDE_3_5_SONNET).unwrap();
        self
    }
    fn claude_3_5_haiku(mut self) -> Self
    where
        Self: Sized,
    {
        *self.model() = ApiLlmModel::from_preset(ApiLlmPreset::CLAUDE_3_5_HAIKU).unwrap();
        self
    }
}
pub trait OpenAiModelTrait {
    fn model(&mut self) -> &mut ApiLlmModel;
    fn model_id_str(mut self, model_id: &str) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let preset = ApiLlmProvider::OpenAi.preset_from_model_id(model_id)?;
        *self.model() = ApiLlmModel::from_preset(preset)?;
        Ok(self)
    }
    fn gpt_4(mut self) -> Self
    where
        Self: Sized,
    {
        *self.model() = ApiLlmModel::from_preset(ApiLlmPreset::GPT_4).unwrap();
        self
    }
    fn gpt_3_5_turbo(mut self) -> Self
    where
        Self: Sized,
    {
        *self.model() = ApiLlmModel::from_preset(ApiLlmPreset::GPT_3_5_TURBO).unwrap();
        self
    }
    fn gpt_4_32k(mut self) -> Self
    where
        Self: Sized,
    {
        *self.model() = ApiLlmModel::from_preset(ApiLlmPreset::GPT_4_32K).unwrap();
        self
    }
    fn gpt_4_turbo(mut self) -> Self
    where
        Self: Sized,
    {
        *self.model() = ApiLlmModel::from_preset(ApiLlmPreset::GPT_4_TURBO).unwrap();
        self
    }
    fn gpt_4o(mut self) -> Self
    where
        Self: Sized,
    {
        *self.model() = ApiLlmModel::from_preset(ApiLlmPreset::GPT_4O).unwrap();
        self
    }
    fn gpt_4o_mini(mut self) -> Self
    where
        Self: Sized,
    {
        *self.model() = ApiLlmModel::from_preset(ApiLlmPreset::GPT_4O_MINI).unwrap();
        self
    }
    fn o1(mut self) -> Self
    where
        Self: Sized,
    {
        *self.model() = ApiLlmModel::from_preset(ApiLlmPreset::O1).unwrap();
        self
    }
    fn o1_mini(mut self) -> Self
    where
        Self: Sized,
    {
        *self.model() = ApiLlmModel::from_preset(ApiLlmPreset::O1_MINI).unwrap();
        self
    }
    fn o3_mini(mut self) -> Self
    where
        Self: Sized,
    {
        *self.model() = ApiLlmModel::from_preset(ApiLlmPreset::O3_MINI).unwrap();
        self
    }
}
pub trait PerplexityModelTrait {
    fn model(&mut self) -> &mut ApiLlmModel;
    fn model_id_str(mut self, model_id: &str) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let preset = ApiLlmProvider::Perplexity.preset_from_model_id(model_id)?;
        *self.model() = ApiLlmModel::from_preset(preset)?;
        Ok(self)
    }
    fn sonar_reasoning_pro(mut self) -> Self
    where
        Self: Sized,
    {
        *self.model() = ApiLlmModel::from_preset(ApiLlmPreset::SONAR_REASONING_PRO).unwrap();
        self
    }
    fn sonar_reasoning(mut self) -> Self
    where
        Self: Sized,
    {
        *self.model() = ApiLlmModel::from_preset(ApiLlmPreset::SONAR_REASONING).unwrap();
        self
    }
    fn sonar_pro(mut self) -> Self
    where
        Self: Sized,
    {
        *self.model() = ApiLlmModel::from_preset(ApiLlmPreset::SONAR_PRO).unwrap();
        self
    }
    fn sonar(mut self) -> Self
    where
        Self: Sized,
    {
        *self.model() = ApiLlmModel::from_preset(ApiLlmPreset::SONAR).unwrap();
        self
    }
}
