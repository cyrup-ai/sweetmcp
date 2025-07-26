//! Memory type enumerations extracted from memory_type.rs

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::utils::{Result, error::Error};

/// Memory type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryTypeEnum {
    /// Semantic memory (knowledge graph)
    Semantic,
    /// Episodic memory (events and experiences)
    Episodic,
    /// Procedural memory (skills and procedures)
    Procedural,
    /// Working memory (temporary storage)
    Working,
    /// Long-term memory (permanent storage)
    LongTerm,
    /// Fact memory (factual information)
    Fact,
    /// Custom memory type
    Custom(u8),
}

impl fmt::Display for MemoryTypeEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryTypeEnum::Semantic => write!(f, "semantic"),
            MemoryTypeEnum::Episodic => write!(f, "episodic"),
            MemoryTypeEnum::Procedural => write!(f, "procedural"),
            MemoryTypeEnum::Working => write!(f, "working"),
            MemoryTypeEnum::LongTerm => write!(f, "longterm"),
            MemoryTypeEnum::Fact => write!(f, "fact"),
            MemoryTypeEnum::Custom(id) => write!(f, "custom_{}", id),
        }
    }
}

impl MemoryTypeEnum {
    /// Convert from string with zero-allocation parsing
    pub fn from_string(s: &str) -> Result<Self> {
        match s.len() {
            8 if s.eq_ignore_ascii_case("semantic") => Ok(MemoryTypeEnum::Semantic),
            8 if s.eq_ignore_ascii_case("episodic") => Ok(MemoryTypeEnum::Episodic),
            10 if s.eq_ignore_ascii_case("procedural") => Ok(MemoryTypeEnum::Procedural),
            7 if s.eq_ignore_ascii_case("working") => Ok(MemoryTypeEnum::Working),
            8 if s.eq_ignore_ascii_case("longterm") => Ok(MemoryTypeEnum::LongTerm),
            4 if s.eq_ignore_ascii_case("fact") => Ok(MemoryTypeEnum::Fact),
            _ => {
                if s.len() > 7 && s.as_bytes().starts_with(b"custom_") {
                    let id_bytes = &s.as_bytes()[7..];
                    let id_str = std::str::from_utf8(id_bytes).map_err(|_| {
                        Error::ConversionError(format!("Invalid UTF-8 in custom memory type: {}", s))
                    })?;
                    let id = id_str.parse::<u8>().map_err(|_| {
                        Error::ConversionError(format!("Invalid custom memory type ID: {}", id_str))
                    })?;
                    Ok(MemoryTypeEnum::Custom(id))
                } else {
                    Err(Error::ConversionError(format!("Unknown memory type: {}", s)))
                }
            }
        }
    }

    /// Get the canonical string representation
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            MemoryTypeEnum::Semantic => "semantic",
            MemoryTypeEnum::Episodic => "episodic", 
            MemoryTypeEnum::Procedural => "procedural",
            MemoryTypeEnum::Working => "working",
            MemoryTypeEnum::LongTerm => "longterm",
            MemoryTypeEnum::Fact => "fact",
            MemoryTypeEnum::Custom(_) => "custom", // Base name, ID appended in Display
        }
    }

    /// Check if this is a built-in memory type
    #[inline]
    pub const fn is_builtin(&self) -> bool {
        !matches!(self, MemoryTypeEnum::Custom(_))
    }

    /// Get the numeric priority for ordering (lower = higher priority)
    #[inline]
    pub const fn priority(&self) -> u8 {
        match self {
            MemoryTypeEnum::Working => 0,    // Highest priority
            MemoryTypeEnum::Episodic => 1,
            MemoryTypeEnum::Semantic => 2,
            MemoryTypeEnum::Fact => 3,
            MemoryTypeEnum::Procedural => 4,
            MemoryTypeEnum::LongTerm => 5,
            MemoryTypeEnum::Custom(id) => 128u8.saturating_add(*id), // Custom types have lower priority
        }
    }
}

/// Memory content type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryContentType {
    /// Text content
    Text,
    /// JSON content
    Json,
    /// Binary content
    Binary,
}

impl fmt::Display for MemoryContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl MemoryContentType {
    /// Convert from string with optimized parsing
    pub fn from_string(s: &str) -> Result<Self> {
        match s.len() {
            4 if s.eq_ignore_ascii_case("text") => Ok(MemoryContentType::Text),
            4 if s.eq_ignore_ascii_case("json") => Ok(MemoryContentType::Json),
            6 if s.eq_ignore_ascii_case("binary") => Ok(MemoryContentType::Binary),
            _ => Err(Error::ConversionError(format!("Unknown content type: {}", s))),
        }
    }

    /// Get the canonical string representation
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            MemoryContentType::Text => "text",
            MemoryContentType::Json => "json",
            MemoryContentType::Binary => "binary",
        }
    }

    /// Check if content type supports embedding
    #[inline]
    pub const fn supports_embeddings(&self) -> bool {
        matches!(self, MemoryContentType::Text | MemoryContentType::Json)
    }

    /// Get the MIME type for HTTP responses
    #[inline]
    pub const fn mime_type(&self) -> &'static str {
        match self {
            MemoryContentType::Text => "text/plain",
            MemoryContentType::Json => "application/json",
            MemoryContentType::Binary => "application/octet-stream",
        }
    }
}