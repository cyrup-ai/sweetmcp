use crate::Error; // Import local Error type
use llm_prompt::PromptTokenizer;
use std::{
    fmt,
    path::{Path, PathBuf},
    sync::Arc, // Keep Arc import
};
use tiktoken_rs::{get_bpe_from_model, CoreBPE};
use tokenizers::Tokenizer as HFTokenizer;

#[derive(Clone)]
pub enum TokenizerBackend { // Consider boxing large variants if size becomes an issue
    HuggingFacesTokenizer(Arc<HFTokenizer>), // Wrap in Arc
    Tiktoken(Arc<CoreBPE>), // Wrap in Arc
}

impl fmt::Debug for TokenizerBackend { // Keep Debug impl
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenizerBackend::HuggingFacesTokenizer(_) => {
                write!(f, "TokenizerBackend::HuggingFacesTokenizer")
            }
            TokenizerBackend::Tiktoken(_) => {
                write!(f, "TokenizerBackend::Tiktoken")
            }
        }
    }
}

#[derive(Debug, Clone)] // Add Clone derive
pub struct LlmTokenizer {
    pub tokenizer: TokenizerBackend,
    pub tokenizer_path: Option<PathBuf>,
    pub with_special_tokens: bool,
    pub white_space_token_id: usize,
}

impl LlmTokenizer {
    pub fn new_tiktoken<T: AsRef<str>>(model_id: T) -> crate::Result<Self> { // Use crate::Result
        let tokenizer = get_bpe_from_model(model_id.as_ref())?;
        let white_space_token_id = usize::try_from(tokenizer.encode_ordinary(" ").remove(0))?;
        Ok(Self {
            tokenizer: TokenizerBackend::Tiktoken(Arc::new(tokenizer)), // Wrap in Arc
            tokenizer_path: None,
            with_special_tokens: false,
            white_space_token_id,
        })
    }

    pub fn new_from_tokenizer(tokenizer: HFTokenizer) -> crate::Result<Self> { // Use crate::Result
        let white_space_token_id = tokenizer.encode(" ", false).unwrap().get_ids()[0];
        Ok(Self {
            tokenizer: TokenizerBackend::HuggingFacesTokenizer(Arc::new(tokenizer)),
            tokenizer_path: None,
            with_special_tokens: false,
            white_space_token_id: white_space_token_id.try_into().unwrap(),
        })
    }

    pub fn new_from_tokenizer_json<T: AsRef<Path>>(local_path: T) -> crate::Result<Self> {
        let tokenizer = HFTokenizer::from_file(&local_path)
            .map_err(|e| Error::HfTokenizer(format!("Failed to load tokenizer from file {:?}: {}", local_path.as_ref(), e)))?;
        let white_space_token_id = tokenizer.encode(" ", false).map_err(|e| Error::HfTokenizer(format!("Failed to encode space: {}", e)))?.get_ids()[0];
        Ok(Self {
            tokenizer: TokenizerBackend::HuggingFacesTokenizer(Arc::new(tokenizer)),
            tokenizer_path: Some(local_path.as_ref().to_path_buf()),
            with_special_tokens: false,
            white_space_token_id: white_space_token_id.try_into().unwrap(), // Add comma here
        })
    }

    pub fn tokenize<T: AsRef<str>>(&self, str: T) -> Vec<usize> {
        self.encode(str.as_ref())
    }

    pub fn detokenize_one(&self, token: usize) -> crate::Result<String> { // Use crate::Result
        self.decode(&[token])
    }

    pub fn detokenize_many(&self, tokens: &[usize]) -> crate::Result<String> { // Use crate::Result
        self.decode(tokens)
    }

    pub fn count_tokens(&self, str: &str) -> usize {
        self.encode(str).len()
    }

    /// Attempts to decode a single token ID back into its original string representation,
    /// ensuring it represents exactly one logical token/word.
    pub fn try_from_single_token_id(&self, token_id: usize) -> crate::Result<String> {
        let decoded_string = self.decode(&[token_id])?;
        // Basic check: ensure the decoded string isn't empty and doesn't look like multiple words.
        // This might need refinement depending on the tokenizer's behavior (e.g., handling spaces).
        if decoded_string.is_empty() {
            return Err(Error::Tokenizer(format!(
                "Token ID {} decodes to an empty string",
                token_id
            )));
        }
        // A simple heuristic: check for internal whitespace which might indicate multiple words.
        if decoded_string.trim().contains(char::is_whitespace) {
             return Err(Error::Tokenizer(format!(
                 "Token ID {} decoded to '{}', which appears to be multiple words/tokens",
                 token_id, decoded_string
             )));
        }
        // Consider removing surrounding whitespace if the tokenizer adds it.
        Ok(decoded_string.trim().to_string())
    }

    /// Attempts to encode a string into a single token ID, returning an error if it
    /// encodes to zero or multiple tokens.
    pub fn try_into_single_token(&self, text: &str) -> crate::Result<usize> {
        let tokens = self.encode(text);
        match tokens.len() {
            0 => Err(Error::Tokenizer(format!(
                "Text '{}' encoded to zero tokens",
                text
            ))),
            1 => Ok(tokens[0]), // Use indexing instead of remove
            n => Err(Error::Tokenizer(format!(
                "Text '{}' encoded to multiple ({}) tokens: {:?}",
                text, n, tokens
            ))),
        }
    }

    /// Creates a window of text normalized to the specified token size in the center of the text.
    ///
    /// # Arguments
    ///
    /// * `text` - The input text to create a window from.
    /// * `target_token_size` - The desired number of tokens in the window.
    ///
    /// # Returns
    ///
    /// A new string that represents the normalized window of text, or the original
    /// text if its token count is less than or equal to `target_token_size`.
    /// Returns an empty string if decoding fails (e.g., invalid token sequence).
    pub fn create_text_window(&self, text: &str, target_token_size: usize) -> String {
        let tokens = self.encode(text); // Use encode directly
        if tokens.len() <= target_token_size {
            return text.to_string();
        }

        let start_token_index = (tokens.len() - target_token_size) / 2;
        let end_token_index = start_token_index + target_token_size;

        // Slice tokens directly
        let preserved_tokens = &tokens[start_token_index..end_token_index];

        // Decode the window, handling potential errors
        match self.decode(preserved_tokens) {
            Ok(window_text) => window_text,
            Err(e) => {
                tracing::error!("Failed to decode text window: {}", e);
                String::new() // Return empty string on error
            }
        }
    }


    /// Creates a range of text from the specified start and end token indices.
    ///
    /// # Arguments
    ///
    /// * `text` - The input text to create a window from.
    /// * `target_token_size` - The desired number of tokens in the window.
    ///
    /// # Returns
    ///
    /// A new string that represents the normalized window of text, or the original
    /// Returns an empty string if decoding fails or indices are invalid.
    pub fn create_text_range(
        &self,
        text: &str,
        start_token_index: usize,
        end_token_index: usize,
    ) -> String {
        let tokens = self.encode(text); // Use encode directly

        // Basic bounds check
        if start_token_index >= end_token_index || start_token_index >= tokens.len() {
            tracing::warn!(
                "Invalid token range [{}, {}) for text with {} tokens.",
                start_token_index,
                end_token_index,
                tokens.len()
            );
            return String::new();
        }

        // Adjust end_token_index if it exceeds bounds
        let end_token_index = end_token_index.min(tokens.len());

        let preserved_tokens = &tokens[start_token_index..end_token_index];

        // Decode the range, handling potential errors
        match self.decode(preserved_tokens) {
            Ok(range_text) => range_text,
            Err(e) => {
                tracing::error!("Failed to decode text range: {}", e);
                String::new() // Return empty string on error
            }
        }
    }


    fn encode_tiktoken(&self, tokenizer: &CoreBPE, str: &str) -> Vec<usize> {
        if self.with_special_tokens {
            tokenizer
                .encode_with_special_tokens(str)
                .iter()
                // Use expect for TryFromIntError conversion, as u32->usize should be safe
                .map(|&x| usize::try_from(x).expect("Tiktoken ID exceeds usize"))
                .collect()
        } else {
            tokenizer
                .encode_ordinary(str)
                .iter()
                .map(|&x| usize::try_from(x).expect("Tiktoken ID exceeds usize"))
                .collect()
        }
    }

    fn encode_hf(&self, tokenizer: &HFTokenizer, str: &str) -> Vec<usize> {
        let encoding_result = if self.with_special_tokens {
            tokenizer.encode(str, true)
        } else {
            tokenizer.encode(str, false)
        };

        match encoding_result {
            Ok(encoding) => encoding.get_ids().iter().map(|&x| x as usize).collect(),
            Err(e) => {
                tracing::error!("HuggingFace tokenizer failed to encode '{}': {}", str, e);
                Vec::new() // Return empty vec on error
            }
        }
    }

    fn encode(&self, str: &str) -> Vec<usize> {
        match &self.tokenizer {
            TokenizerBackend::HuggingFacesTokenizer(tokenizer) => self.encode_hf(&**tokenizer, str), // Explicitly deref Arc
            TokenizerBackend::Tiktoken(tokenizer) => self.encode_tiktoken(&**tokenizer, str), // Explicitly deref Arc
        }
    }

    fn decode_tiktoken(
        &self,
        tokenizer: &CoreBPE,
        tokens: &[usize],
    ) -> crate::Result<String> {
        // Convert usize to u32, handling potential overflow (though unlikely for token IDs)
        let u32_tokens: Vec<u32> = tokens
            .iter()
            .map(|&x| u32::try_from(x))
            .collect::<Result<Vec<u32>, _>>()
            .map_err(Error::TryFromInt)?;

        // Decode using Tiktoken, map error to crate::Error::Tiktoken
        tokenizer.decode(u32_tokens).map_err(|e| Error::Tiktoken(e.to_string()))
    }

    fn decode_hf(&self, tokenizer: &HFTokenizer, tokens: &[usize]) -> crate::Result<String> {
        // Convert usize to u32
        let u32_tokens: Vec<u32> = tokens
            .iter()
            .map(|&x| u32::try_from(x))
            .collect::<Result<Vec<u32>, _>>()
            .map_err(Error::TryFromInt)?;

        // Decode using HF Tokenizer, map error to crate::Error::HfTokenizer
        tokenizer
            .decode(&u32_tokens, true) // `skip_special_tokens = true` might be desired depending on use case
            .map_err(|e| Error::HfTokenizer(e.to_string()))
    }

    fn decode(&self, tokens: &[usize]) -> crate::Result<String> {
        match &self.tokenizer {
            TokenizerBackend::HuggingFacesTokenizer(tokenizer) => self.decode_hf(tokenizer, tokens),
            TokenizerBackend::Tiktoken(tokenizer) => self.decode_tiktoken(tokenizer, tokens),
        }
    }
}

impl PromptTokenizer for LlmTokenizer {
    fn tokenize(&self, input: &str) -> Vec<usize> {
        self.encode(input) // Use internal encode method
    }

    fn count_tokens(&self, str: &str) -> usize {
        self.encode(str).len() // Use internal encode method
    }
}
