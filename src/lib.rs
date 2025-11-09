//! Indexer CLI Library
//!
//! A comprehensive library for managing URL submissions to Google Indexing API
//! and IndexNow API. This library provides:
//!
//! - Configuration management
//! - API clients for Google and IndexNow
//! - Sitemap parsing and processing
//! - Batch submission handling
//! - Submission history tracking
//! - CLI interface and commands
//!
//! # Examples
//!
//! ```no_run
//! use indexer_cli::config::load_config;
//! use indexer_cli::types::Result;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Load configuration
//!     let config = load_config(None)?;
//!
//!     // Use the library...
//!
//!     Ok(())
//! }
//! ```

// Core modules
pub mod api;
pub mod auth;
pub mod cli;
pub mod commands;
pub mod config;
pub mod constants;
pub mod database;
pub mod services;
pub mod types;
pub mod utils;

// Re-export commonly used types
pub use types::{IndexerError, Result};

// Re-export configuration
pub use config::{load_config, Settings};

// Re-export CLI
pub use cli::Cli;
