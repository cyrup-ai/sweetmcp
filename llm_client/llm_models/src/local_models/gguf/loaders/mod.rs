pub mod hf;
pub mod local;
// Functions load_tokenizer and load_chat_template are defined in gguf/mod.rs

// Re-export loaders
pub use hf::GgufHfLoader;
pub use local::GgufLocalLoader;
pub use crate::gguf_presets::loader::GgufPresetLoader;
