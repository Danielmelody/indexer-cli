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

    // Extract verbose flag before matching
    let verbose = cli.verbose;

    // Dispatch to the appropriate command handler
    match cli.command {
        Commands::Init(ref args) => {
            if verbose {
                println!("{}", "Running init command...".cyan());
            }
            commands::init::run(args.clone(), &cli).await
        }

        Commands::Config(ref args) => {
            if verbose {
                println!("{}", "Running config command...".cyan());
            }
            commands::config::run(args.clone(), &cli).await
        }

        Commands::Google(ref args) => {
            if verbose {
                println!("{}", "Running google command...".cyan());
            }
            commands::google::run(args.clone(), &cli).await
        }

        Commands::IndexNow(ref args) => {
            if verbose {
                println!("{}", "Running indexnow command...".cyan());
            }
            commands::indexnow::run(args.clone(), &cli).await
        }

        Commands::Submit(ref args) => {
            if verbose {
                println!("{}", "Running submit command...".cyan());
            }
            commands::submit::run(args.clone(), &cli).await
        }

        Commands::Sitemap(ref args) => {
            if verbose {
                println!("{}", "Running sitemap command...".cyan());
            }
            commands::sitemap::run(args.clone(), &cli).await
        }

        Commands::History(ref args) => {
            if verbose {
                println!("{}", "Running history command...".cyan());
            }
            commands::history::run(args.clone(), &cli).await
        }

        Commands::Watch(ref args) => {
            if verbose {
                println!("{}", "Running watch command...".cyan());
            }
            commands::watch::run(args.clone(), &cli).await
        }

        Commands::Validate(ref args) => {
            if verbose {
                println!("{}", "Running validate command...".cyan());
            }
            commands::validate::run(args.clone(), &cli).await
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
