//! Indexer CLI - Main entry point.
//!
//! A command-line tool for submitting URLs to Google Indexing API and IndexNow.

use clap::Parser;
use colored::Colorize;
use indexer_cli::cli::{handle_command, Cli};
use indexer_cli::types::IndexerError;
use std::process;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    // Parse command-line arguments
    let cli = Cli::parse();

    // Initialize logging based on verbose/quiet flags
    init_logging(&cli);

    // Handle the command and capture result
    let result = handle_command(cli.clone()).await;

    // Handle errors and exit appropriately
    match result {
        Ok(()) => {
            // Success - exit with code 0
            process::exit(0);
        }
        Err(err) => {
            // Error occurred - display and exit with appropriate code
            handle_error(err, &cli);
        }
    }
}

/// Initialize logging/tracing based on CLI flags.
fn init_logging(cli: &Cli) {
    let level = if cli.verbose {
        Level::DEBUG
    } else if cli.quiet {
        Level::ERROR
    } else {
        Level::INFO
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .without_time()
        .finish();

    // Ignore error if already initialized
    let _ = tracing::subscriber::set_global_default(subscriber);
}

/// Handle errors by displaying them and exiting with appropriate code.
fn handle_error(error: IndexerError, cli: &Cli) -> ! {
    if !cli.quiet {
        eprintln!();
        eprintln!("{} {}", "Error:".red().bold(), error);

        // Display additional context for specific error types
        if error.is_config_error() {
            eprintln!();
            eprintln!("{}", "Hint:".yellow().bold());
            eprintln!("  Run 'indexer init' to create a configuration file");
            eprintln!("  Or use 'indexer config --help' for configuration management");
        } else if error.is_api_error() {
            eprintln!();
            eprintln!("{}", "Hint:".yellow().bold());
            eprintln!("  Run 'indexer validate' to check your API configuration");
        }

        if cli.verbose {
            eprintln!();
            eprintln!("{}", "Debug information:".dimmed());
            eprintln!("{}", format!("  {:?}", error).dimmed());
        }

        eprintln!();
    }

    // Exit with appropriate error code
    let exit_code = error.exit_code();
    process::exit(exit_code);
}
