//! Command handler - Routes CLI commands to their implementations.
//!
//! This module provides the main command dispatcher that routes incoming
//! CLI commands to their respective implementation functions.

use crate::cli::args::{Cli, Commands};
use crate::commands;
use crate::types::Result;
use colored::Colorize;

/// Handle the CLI command by dispatching to appropriate command handlers.
///
/// This is the main entry point for command execution. It takes the parsed
/// CLI arguments and routes them to the appropriate command implementation.
///
/// # Arguments
///
/// * `cli` - The parsed CLI arguments
///
/// # Returns
///
/// Returns `Ok(())` if the command executed successfully, or an error if
/// something went wrong.
///
/// # Example
///
/// ```no_run
/// use indexer_cli::cli::{Cli, handle_command};
/// use clap::Parser;
///
/// #[tokio::main]
/// async fn main() {
///     let cli = Cli::parse();
///     if let Err(e) = handle_command(cli).await {
///         eprintln!("Error: {}", e);
///         std::process::exit(1);
///     }
/// }
/// ```
pub async fn handle_command(cli: Cli) -> Result<()> {
    // Set up colored output based on flags
    if cli.no_color {
        colored::control::set_override(false);
    }

    // Dispatch to the appropriate command handler
    match cli.command {
        Commands::Init(args) => {
            if cli.verbose {
                println!("{}", "Running init command...".cyan());
            }
            commands::init::run(args, &cli).await
        }

        Commands::Config(args) => {
            if cli.verbose {
                println!("{}", "Running config command...".cyan());
            }
            commands::config::run(args, &cli).await
        }

        Commands::Google(args) => {
            if cli.verbose {
                println!("{}", "Running google command...".cyan());
            }
            commands::google::run(args, &cli).await
        }

        Commands::IndexNow(args) => {
            if cli.verbose {
                println!("{}", "Running indexnow command...".cyan());
            }
            commands::indexnow::run(args, &cli).await
        }

        Commands::Submit(args) => {
            if cli.verbose {
                println!("{}", "Running submit command...".cyan());
            }
            commands::submit::run(args, &cli).await
        }

        Commands::Sitemap(args) => {
            if cli.verbose {
                println!("{}", "Running sitemap command...".cyan());
            }
            commands::sitemap::run(args, &cli).await
        }

        Commands::History(args) => {
            if cli.verbose {
                println!("{}", "Running history command...".cyan());
            }
            commands::history::run(args, &cli).await
        }

        Commands::Watch(args) => {
            if cli.verbose {
                println!("{}", "Running watch command...".cyan());
            }
            commands::watch::run(args, &cli).await
        }

        Commands::Validate(args) => {
            if cli.verbose {
                println!("{}", "Running validate command...".cyan());
            }
            commands::validate::run(args, &cli).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_module() {
        // Basic module test
        assert!(true);
    }
}
