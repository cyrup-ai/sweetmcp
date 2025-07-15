use super::metadata::tokenizer::TokenizerMetadata;
use crate::{bail, Error, Result};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Clone, PartialEq)]
pub struct LlmChatTemplate {
    pub chat_template: String,
    pub bos_token: Option<String>,
    pub eos_token: String,
    pub unk_token: Option<String>,
    pub base_generation_prefix: Option<String>,
}

impl LlmChatTemplate {
    pub fn from_local_path(
        tokenizer_config_local_path: &std::path::Path,
    ) -> Result<Self> {
        let file = std::fs::File::open(tokenizer_config_local_path)?;
        let reader = std::io::BufReader::new(file);
        let mut chat_template: LlmChatTemplate = serde_json::from_reader(reader)?;
        chat_template.set_generation_prefix()?;
        Ok(chat_template)
    }

    pub fn from_gguf_tokenizer(tokenizer: &TokenizerMetadata) -> crate::Result<Self> {
        let chat_template = if let Some(chat_template) = &tokenizer.chat_template {
            chat_template
        } else {
            bail!("chat_template not found in GGUF tokenizer metadata.");
        };
        let ggml = if let Some(ggml) = &tokenizer.ggml {
            ggml
        } else {
            bail!("GGML tokenizer model not found in GGUF tokenizer metadata.");
        };

        let bos_token = ggml
            .tokens
            .get(ggml.bos_token_id as usize)
            .map(ToString::to_string);
            // .ok_or_else(|| Error::Tokenizer(format!("BOS Token not found for ID: {}", ggml.bos_token_id)))?; // BOS is optional in some templates

        let eos_token = ggml
            .tokens
            .get(ggml.eos_token_id as usize)
            .map(ToString::to_string)
            .ok_or_else(|| Error::Tokenizer(format!("EOS Token not found for ID: {}", ggml.eos_token_id)))?; // EOS is required

        let unk_token = if let Some(unk_token_id) = ggml.unknown_token_id {
            Some(
                ggml.tokens // UNK is optional
                    .get(unk_token_id as usize)
                    .map(ToString::to_string)
                    .ok_or_else(|| Error::Tokenizer(format!("UNK Token not found for ID: {}", unk_token_id)))?,
            )
        } else {
            None
        };

        let mut chat_template = LlmChatTemplate {
            chat_template: chat_template.to_owned(),
            bos_token, // Keep as Option<String>
            eos_token,
            unk_token,
            base_generation_prefix: None,
        };
        chat_template.set_generation_prefix()?;
        Ok(chat_template)
    }

    fn set_generation_prefix(&mut self) -> Result<()> {
        let user_message_1 = HashMap::from([
            ("role".to_string(), "user".to_string()),
            ("content".to_string(), "test_user_message_1".to_string()),
        ]);
        let assistant_message_1 = HashMap::from([
            ("role".to_string(), "assistant".to_string()),
            (
                "content".to_string(),
                "test_assistant_message_1".to_string(),
            ),
        ]);

        let message_1 = llm_prompt::apply_chat_template( // Assuming this returns String
            &vec![user_message_1.clone()],
            &self.chat_template,
            self.bos_token.as_deref(),
            &self.eos_token,
            self.unk_token.as_deref(),
        );
        let message_1 = message_1
            .trim_end_matches(self.eos_token.as_str())
            .to_owned();
        let message_2 = llm_prompt::apply_chat_template( // Assuming this returns String
            &vec![user_message_1, assistant_message_1],
            &self.chat_template,
            self.bos_token.as_deref(),
            &self.eos_token,
            self.unk_token.as_deref(),
        );

        // Find the point where the outputs start to differ
        let diff_index = message_1
            .chars()
            .zip(message_2.chars())
            .position(|(a, b)| a != b)
            .unwrap_or(message_1.len());

        // Extract the differing part
        let diff_part = &message_2[diff_index..];

        // Find the start of the assistant content
        if let Some(content_index) = diff_part.find("test_assistant_message_1") {
            // The prefix is everything before the content
            self.base_generation_prefix = Some(
                diff_part[..content_index]
                    .trim_start_matches(self.eos_token.as_str())
                    .to_string(),
            );
        } else {
            bail!("Error finding base_generation_prefix: assistant content 'test_assistant_message_1' not found in rendered template difference.");
        }
        Ok(())
    }
}

impl Default for LlmChatTemplate {
    fn default() -> Self {
        Self {
            chat_template: String::from("{% if messages[0].role == 'system' %}{% set loop_messages = messages[1:] %}{% set system_message = messages[0].content %}{% else %}{% set loop_messages = messages %}{% set system_message = '' %}{% endif %}{% for message in loop_messages %}{% if message.role == 'user' %}{{ message.content }}{% elif message.role == 'assistant' %}{{ message.content }}{% endif %}{% endfor %}"),
            bos_token: None,
            eos_token: String::from("</s>"),
            unk_token: None,
            base_generation_prefix: None,
        }
    }
}

impl std::fmt::Debug for LlmChatTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug_struct = f.debug_struct("LlmChatTemplate");
        debug_struct.field("chat_template", &"string too long to print nicely");
        debug_struct.field("bos_token", &self.bos_token);
        debug_struct.field("eos_token", &self.eos_token);
        debug_struct.field("unk_token", &self.unk_token);
        debug_struct.finish()
    }
}
