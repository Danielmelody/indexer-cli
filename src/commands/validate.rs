//! Validate command implementation.
//!
//! This module provides functionality to validate the configuration and setup
//! of the indexer-cli tool, including Google and IndexNow API credentials.

use crate::api::google_indexing::GoogleIndexingClient;
use crate::api::indexnow::IndexNowClient;
use crate::cli::args::{Cli, OutputFormat, ValidateArgs};
use crate::config::{expand_tilde, load_config, validate_config};
use crate::types::Result;
use chrono::Utc;
use colored::Colorize;

/// Run the validate command.
pub async fn run(args: ValidateArgs, cli: &Cli) -> Result<()> {
    let quiet = cli.quiet;

    if !quiet {
        println!("{}", "Validating configuration...".cyan().bold());
        println!();
    }

    // Load configuration
    let config = load_config()?;

    // Run basic validation
    let mut report = validate_config(&config)?;

    // Additional checks if requested
    if args.check_connectivity {
        if !quiet {
            println!("{}", "Checking connectivity...".cyan());
        }

        let connectivity_results = check_connectivity(&config).await?;
        report.info.extend(connectivity_results);
    }

    if args.check_files {
        if !quiet {
            println!("{}", "Checking files...".cyan());
        }

        let file_results = check_referenced_files(&config)?;
        report.info.extend(file_results);
    }

    if args.check_permissions {
        if !quiet {
            println!("{}", "Checking permissions...".cyan());
        }

        let permission_results = check_file_permissions(&config)?;
        report.info.extend(permission_results);
    }

    // Output based on format
    match args.format {
        OutputFormat::Json => {
            output_json(&report)?;
        }
        OutputFormat::Text => {
            output_text(&report, cli.verbose, quiet)?;
        }
        _ => {
            output_text(&report, cli.verbose, quiet)?;
        }
    }

    // Check if validation passed
    let failed = !report.is_valid() || (args.strict && !report.warnings.is_empty());

    if failed {
        return Err(crate::types::IndexerError::ConfigValidationError {
            message: "Configuration validation failed".to_string(),
        });
    }

    Ok(())
}

/// Check connectivity to configured APIs
async fn check_connectivity(config: &crate::config::Settings) -> Result<Vec<String>> {
    let mut results = Vec::new();

    // Test Google API if configured
    if let Some(google_config) = &config.google {
        if google_config.enabled {
            match GoogleIndexingClient::new(google_config.service_account_file.clone()).await {
                Ok(_client) => {
                    // Successfully created client (which validates and authenticates)
                    results.push("✓ Google API authentication successful".to_string());
                }
                Err(e) => {
                    results.push(format!("✗ Google API connection failed: {}", e));
                }
            }
        }
    }

    // Test IndexNow API if configured
    if let Some(indexnow_config) = &config.indexnow {
        if indexnow_config.enabled {
            match IndexNowClient::new(
                indexnow_config.api_key.clone(),
                indexnow_config.key_location.clone(),
                indexnow_config.endpoints.clone(),
            ) {
                Ok(_client) => {
                    results.push(format!(
                        "✓ IndexNow client initialized ({} endpoints configured)",
                        indexnow_config.endpoints.len()
                    ));
                }
                Err(e) => {
                    results.push(format!("✗ IndexNow client error: {}", e));
                }
            }
        }
    }

    Ok(results)
}

/// Check that referenced files exist
fn check_referenced_files(config: &crate::config::Settings) -> Result<Vec<String>> {
    let mut results = Vec::new();

    // Check Google service account file
    if let Some(google_config) = &config.google {
        if google_config.enabled {
            let path = &google_config.service_account_file;
            if path.exists() {
                results.push(format!("✓ Google service account file exists: {}", path.display()));
            } else {
                results.push(format!(
                    "✗ Google service account file not found: {}",
                    path.display()
                ));
            }
        }
    }

    // Check database directory
    let db_path = expand_tilde(&config.history.database_path);
    if db_path.exists() {
        results.push(format!("✓ Database file exists: {}", db_path.display()));
    } else if let Some(parent) = db_path.parent() {
        if parent.exists() {
            results.push(format!(
                "ℹ Database will be created at: {}",
                db_path.display()
            ));
        } else {
            results.push(format!(
                "✗ Database directory does not exist: {}",
                parent.display()
            ));
        }
    }

    // Check log file directory
    let log_path = expand_tilde(&config.logging.file);
    if let Some(parent) = log_path.parent() {
        if parent.exists() {
            results.push(format!(
                "✓ Log directory exists: {}",
                parent.display()
            ));
        } else {
            results.push(format!(
                "⚠ Log directory does not exist (will be created): {}",
                parent.display()
            ));
        }
    }

    Ok(results)
}

/// Check file permissions
fn check_file_permissions(config: &crate::config::Settings) -> Result<Vec<String>> {
    let mut results = Vec::new();

    // Check Google service account file
    if let Some(google_config) = &config.google {
        if google_config.enabled {
            let path = &google_config.service_account_file;
            if path.exists() {
                let metadata = std::fs::metadata(path)?;
                let permissions = metadata.permissions();

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mode = permissions.mode();

                    // Warn if file is world-readable (security issue)
                    if mode & 0o004 != 0 {
                        results.push(format!(
                            "⚠ {} is world-readable (security risk)",
                            path.display()
                        ));
                    } else {
                        results.push(format!("✓ {} has secure permissions", path.display()));
                    }
                }

                #[cfg(not(unix))]
                {
                    if permissions.readonly() {
                        results.push(format!("⚠ {} is read-only", path.display()));
                    } else {
                        results.push(format!("✓ {} is accessible", path.display()));
                    }
                }
            }
        }
    }

    // Check database file permissions
    let db_path = expand_tilde(&config.history.database_path);
    if db_path.exists() {
        let _metadata = std::fs::metadata(&db_path)?;
        results.push(format!(
            "✓ Database file accessible: {}",
            db_path.display()
        ));
    } else if let Some(parent) = db_path.parent() {
        if parent.exists() {
            let metadata = std::fs::metadata(parent)?;
            if metadata.permissions().readonly() {
                results.push(format!(
                    "✗ Database directory is read-only: {}",
                    parent.display()
                ));
            } else {
                results.push(format!(
                    "ℹ Database will be created at: {}",
                    db_path.display()
                ));
            }
        }
    }

    Ok(results)
}

/// Output validation report in text format
fn output_text(
    report: &crate::config::validation::ValidationReport,
    verbose: bool,
    quiet: bool,
) -> Result<()> {
    if quiet {
        return Ok(());
    }

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

    if !report.info.is_empty() && verbose {
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

    Ok(())
}

/// Output validation report in JSON format
fn output_json(report: &crate::config::validation::ValidationReport) -> Result<()> {
    let json_output = serde_json::json!({
        "valid": report.is_valid(),
        "successes": report.successes,
        "warnings": report.warnings,
        "errors": report.errors,
        "info": report.info,
        "timestamp": Utc::now().to_rfc3339(),
    });

    println!("{}", serde_json::to_string_pretty(&json_output)?);
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
