//! Types module - Error handling and Result types.
//!
//! This module provides the core type definitions used throughout the indexer-cli
//! application, including comprehensive error types and convenient Result type aliases.

pub mod error;
pub mod result;

// Re-export commonly used types for convenience
pub use error::IndexerError;
pub use result::{BoolResult, OptionResult, Result, StringResult, UnitResult, VecResult};
