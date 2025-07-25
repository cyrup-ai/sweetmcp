//! DEPRECATED: Basic operations for stabilizer quantum error correction
//!
//! This module is deprecated. All functionality has been moved to `stabilizer_operations.rs`.
//! This file is kept for backward compatibility but will be removed in a future version.

#![allow(deprecated)]
#![allow(unused_imports)]

// Re-export everything from stabilizer_operations
pub use super::stabilizer_operations::{
    StabilizerCode, 
    StabilizerGenerator, 
    PauliOp, 
    PauliMatrix, 
    LogicalOperator, 
    LogicalOpType,
    ErrorPattern, 
    SyndromeResult,
    StabilizerCodeParameters,
    DecoderType,
    StabilizerCodeExt,
};

// Add deprecation warnings for any remaining public items
#[deprecated(
    since = "0.1.0",
    note = "This module is deprecated. Use `stabilizer_operations` instead."
)]
pub mod deprecated {
    // This module is intentionally left empty as all functionality has been moved
    // to stabilizer_operations.rs
}
