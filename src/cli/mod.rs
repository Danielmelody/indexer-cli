//! CLI module - Command-line interface handling.
//!
//! This module provides the command-line interface components including
//! argument parsing and command dispatching.

pub mod args;
pub mod handler;

// Re-export the main CLI structures
pub use args::Cli;
pub use handler::handle_command;
