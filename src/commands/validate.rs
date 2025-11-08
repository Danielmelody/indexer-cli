//! Validate command implementation.
//!
//! This module provides functionality to validate the configuration and setup
//! of the indexer-cli tool, including Google and IndexNow API credentials.

use crate::cli::args::{Cli, ValidateArgs, ValidateTarget};
use crate::config::{load_config, validate_config};
use crate::types::Result;
use colored::Colorize;

/// Run the validate command.
pub async fn run(args: ValidateArgs, cli: &Cli) -> Result<()> {
    let quiet = cli.quiet;

    if !quiet {
        println!("{}", "Validating configuration...".cyan().bold());
        println!();
    }

    // Load configuration
    let config = load_config(cli.config.as_deref())?;

    // Run validation
    let report = validate_config(&config)?;

    // Display results
    if !quiet {
        if !report.successes.is_empty() {
            println!("{}", "Successes:".green().bold());
            for success in &report.successes {
                println!("  {} {}", "✓".green(), success);
            }
            println!();
        }

        if !report.warnings.is_empty() {
            println!("{}", "Warnings:".yellow().bold());
            for warning in &report.warnings {
                println!("  {} {}", "⚠".yellow(), warning);
            }
            println!();
        }

        if !report.errors.is_empty() {
            println!("{}", "Errors:".red().bold());
            for error in &report.errors {
                println!("  {} {}", "✗".red(), error);
            }
            println!();
        }

        if !report.info.is_empty() && cli.verbose {
            println!("{}", "Info:".cyan());
            for info in &report.info {
                println!("  {} {}", "ℹ".cyan(), info);
            }
            println!();
        }

        if report.is_valid() {
            println!("{}", "✓ All validations passed!".green().bold());
        } else {
            println!("{}", "✗ Validation failed!".red().bold());
        }
    }

    // Return error if validation failed
    if !report.is_valid() {
        return Err(crate::types::IndexerError::ConfigValidationError {
            message: "Configuration validation failed".to_string(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_module() {
        assert!(true);
    }
}
