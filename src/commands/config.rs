//! Config command - Configuration management.
//!
//! This module provides commands for managing the indexer-cli configuration,
//! including listing, getting, setting, and validating configuration values.

use crate::cli::args::{Cli, ConfigArgs, ConfigCommand, ConfigGetArgs, ConfigSetArgs};
use crate::config::settings::Settings;
use crate::config::validation::{validate_config, ValidationReport};
use crate::config::{
    find_project_config, get_global_config_path, load_config, load_from_file, save_global_config,
    save_project_config,
};
use crate::types::{IndexerError, Result};
use anyhow::Context;
use colored::Colorize;
use std::path::PathBuf;

/// Run the config command
pub async fn run(args: ConfigArgs, cli: &Cli) -> Result<()> {
    match args.command {
        ConfigCommand::List => list_config(cli).await,
        ConfigCommand::Set(set_args) => set_config(set_args, cli).await,
        ConfigCommand::Get(get_args) => get_config(get_args, cli).await,
        ConfigCommand::Validate => validate_config_command(cli).await,
        ConfigCommand::Path => show_config_path(cli).await,
    }
}

/// List all configuration settings
async fn list_config(cli: &Cli) -> Result<()> {
    let settings = load_config_for_cli(cli)?;

    // Display configuration in YAML format
    println!("{}", "Configuration Settings".cyan().bold());
    println!("{}", "━".repeat(60).dimmed());
    println!();

    let yaml =
        serde_yaml::to_string(&settings).context("Failed to serialize configuration to YAML")?;

    // Print with syntax highlighting (simple coloring)
    for line in yaml.lines() {
        if line.ends_with(':') && !line.starts_with(' ') {
            // Top-level keys
            println!("{}", line.bright_blue().bold());
        } else if line.contains(':') {
            // Key-value pairs
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                print!("{}{}", parts[0].green(), ":".dimmed());
                println!("{}", parts[1]);
            } else {
                println!("{}", line);
            }
        } else {
            println!("{}", line);
        }
    }

    println!();
    show_config_sources(cli)?;

    Ok(())
}

/// Set a configuration value
async fn set_config(args: ConfigSetArgs, cli: &Cli) -> Result<()> {
    let mut settings = load_config_for_cli(cli)?;

    // Parse the key and set the value
    set_value_by_key(&mut settings, &args.key, &args.value)?;

    // Validate the new configuration
    if let Err(e) = validate_config(&settings) {
        return Err(IndexerError::ConfigValidationError {
            message: format!("Configuration validation failed after setting value: {}", e),
        });
    }

    // Save to the appropriate file
    let config_path = if args.global {
        save_global_config(&settings)?
    } else {
        save_project_config(&settings)?
    };

    println!(
        "{} Set {} = {} in {}",
        "✓".green().bold(),
        args.key.cyan(),
        args.value.yellow(),
        config_path.display().to_string().dimmed()
    );

    Ok(())
}

/// Get a configuration value
async fn get_config(args: ConfigGetArgs, cli: &Cli) -> Result<()> {
    let settings = load_config_for_cli(cli)?;

    // Get the value by key
    let value = get_value_by_key(&settings, &args.key)?;

    println!("{}", value);

    Ok(())
}

/// Validate the current configuration
async fn validate_config_command(cli: &Cli) -> Result<()> {
    println!("{}", "Validating configuration...".cyan().bold());
    println!("{}", "━".repeat(60).dimmed());

    let settings = load_config_for_cli(cli)?;

    match validate_config(&settings) {
        Ok(report) => {
            print_validation_report(&report);
            println!();
            println!("{}", "✓ Configuration is valid".green().bold());
            Ok(())
        }
        Err(e) => {
            // The error already contains the validation report
            eprintln!("{}", e.to_string().red());
            Err(IndexerError::ConfigValidationError {
                message: "Configuration validation failed".to_string(),
            })
        }
    }
}

/// Show configuration file paths
async fn show_config_path(cli: &Cli) -> Result<()> {
    println!("{}", "Configuration File Paths".cyan().bold());
    println!("{}", "━".repeat(60).dimmed());
    println!();

    // Global config path
    if let Some(global_path) = get_global_config_path() {
        let exists = global_path.exists();
        println!(
            "{} {}",
            "Global:".bright_blue().bold(),
            global_path.display()
        );
        println!(
            "  {}",
            if exists {
                "✓ File exists".green()
            } else {
                "✗ File does not exist".yellow()
            }
        );
        println!();
    }

    // Project config path
    if let Some(project_path) = find_project_config() {
        println!(
            "{} {}",
            "Project:".bright_blue().bold(),
            project_path.display()
        );
        println!("  {}", "✓ File exists".green());
        println!();
    } else {
        println!(
            "{} {}",
            "Project:".bright_blue().bold(),
            "None found".dimmed()
        );
        println!("  {} indexer.yaml", "Would use:".dimmed());
        println!();
    }

    // CLI override
    if let Some(ref config_path) = cli.config {
        println!(
            "{} {}",
            "CLI Override:".bright_blue().bold(),
            config_path.display()
        );
        let exists = config_path.exists();
        println!(
            "  {}",
            if exists {
                "✓ File exists".green()
            } else {
                "✗ File does not exist".red()
            }
        );
    }

    Ok(())
}

/// Load configuration based on CLI options
fn load_config_for_cli(cli: &Cli) -> Result<Settings> {
    if let Some(ref config_path) = cli.config {
        // Use explicit config file if provided
        load_from_file(config_path).map_err(|_e| IndexerError::ConfigFileNotFound {
            path: config_path.clone(),
        })
    } else {
        // Use standard config loading
        load_config().map_err(|e| IndexerError::Other(e))
    }
}

/// Show configuration sources
fn show_config_sources(cli: &Cli) -> Result<()> {
    println!("{}", "Configuration Sources (priority order):".dimmed());

    if cli.config.is_some() {
        println!("  {} CLI override (--config)", "1.".dimmed());
        println!("  {} Environment variables (INDEXER_*)", "2.".dimmed());
    } else {
        println!("  {} Environment variables (INDEXER_*)", "1.".dimmed());

        if find_project_config().is_some() {
            println!("  {} Project config (./indexer.yaml)", "2.".dimmed());
            println!(
                "  {} Global config (~/.indexer-cli/config.yaml)",
                "3.".dimmed()
            );
        } else {
            println!("  {} Project config (not found)", "2.".dimmed());
            println!(
                "  {} Global config (~/.indexer-cli/config.yaml)",
                "3.".dimmed()
            );
        }

        println!("  {} Default values", "4.".dimmed());
    }

    Ok(())
}

/// Print a validation report with colors
fn print_validation_report(report: &ValidationReport) {
    if !report.successes.is_empty() {
        println!();
        println!("{}", "Successes:".green().bold());
        for success in &report.successes {
            println!("  {} {}", "✓".green(), success);
        }
    }

    if !report.info.is_empty() {
        println!();
        println!("{}", "Information:".blue().bold());
        for info in &report.info {
            println!("  {} {}", "ℹ".blue(), info);
        }
    }

    if !report.warnings.is_empty() {
        println!();
        println!("{}", "Warnings:".yellow().bold());
        for warning in &report.warnings {
            println!("  {} {}", "⚠".yellow(), warning);
        }
    }

    if !report.errors.is_empty() {
        println!();
        println!("{}", "Errors:".red().bold());
        for error in &report.errors {
            println!("  {} {}", "✗".red(), error);
        }
    }
}

/// Set a configuration value by key path (e.g., "google.enabled")
fn set_value_by_key(settings: &mut Settings, key: &str, value: &str) -> Result<()> {
    let parts: Vec<&str> = key.split('.').collect();

    match parts.as_slice() {
        // Google configuration
        ["google", "enabled"] => {
            let enabled = parse_bool(value)?;
            if settings.google.is_none() {
                settings.google = Some(Default::default());
            }
            settings.google.as_mut().unwrap().enabled = enabled;
        }
        ["google", "service_account_file"] => {
            if settings.google.is_none() {
                settings.google = Some(Default::default());
            }
            settings.google.as_mut().unwrap().service_account_file = Some(PathBuf::from(value));
        }
        ["google", "batch_size"] => {
            let batch_size = value
                .parse::<usize>()
                .context("Batch size must be a positive integer")?;
            if settings.google.is_none() {
                settings.google = Some(Default::default());
            }
            settings.google.as_mut().unwrap().batch_size = batch_size;
        }
        ["google", "quota", "daily_limit"] => {
            let daily_limit = value
                .parse::<u32>()
                .context("Daily limit must be a positive integer")?;
            if settings.google.is_none() {
                settings.google = Some(Default::default());
            }
            settings.google.as_mut().unwrap().quota.daily_limit = daily_limit;
        }
        ["google", "quota", "rate_limit"] => {
            let rate_limit = value
                .parse::<u32>()
                .context("Rate limit must be a positive integer")?;
            if settings.google.is_none() {
                settings.google = Some(Default::default());
            }
            settings.google.as_mut().unwrap().quota.rate_limit = rate_limit;
        }

        // IndexNow configuration
        ["indexnow", "enabled"] => {
            let enabled = parse_bool(value)?;
            if settings.indexnow.is_none() {
                settings.indexnow = Some(Default::default());
            }
            settings.indexnow.as_mut().unwrap().enabled = enabled;
        }
        ["indexnow", "api_key"] => {
            if settings.indexnow.is_none() {
                settings.indexnow = Some(Default::default());
            }
            settings.indexnow.as_mut().unwrap().api_key = value.to_string();
        }
        ["indexnow", "key_location"] => {
            if settings.indexnow.is_none() {
                settings.indexnow = Some(Default::default());
            }
            settings.indexnow.as_mut().unwrap().key_location = value.to_string();
        }
        ["indexnow", "batch_size"] => {
            let batch_size = value
                .parse::<usize>()
                .context("Batch size must be a positive integer")?;
            if settings.indexnow.is_none() {
                settings.indexnow = Some(Default::default());
            }
            settings.indexnow.as_mut().unwrap().batch_size = batch_size;
        }

        // Sitemap configuration
        ["sitemap", "url"] => {
            if settings.sitemap.is_none() {
                settings.sitemap = Some(Default::default());
            }
            settings.sitemap.as_mut().unwrap().url = value.to_string();
        }
        ["sitemap", "follow_index"] => {
            let follow_index = parse_bool(value)?;
            if settings.sitemap.is_none() {
                settings.sitemap = Some(Default::default());
            }
            settings.sitemap.as_mut().unwrap().follow_index = follow_index;
        }

        // History configuration
        ["history", "enabled"] => {
            settings.history.enabled = parse_bool(value)?;
        }
        ["history", "database_path"] => {
            settings.history.database_path = value.to_string();
        }
        ["history", "retention_days"] => {
            settings.history.retention_days = value
                .parse::<u32>()
                .context("Retention days must be a positive integer")?;
        }

        // Logging configuration
        ["logging", "level"] => {
            settings.logging.level = value.to_string();
        }
        ["logging", "file"] => {
            settings.logging.file = value.to_string();
        }
        ["logging", "max_size_mb"] => {
            settings.logging.max_size_mb = value
                .parse::<u32>()
                .context("Max size must be a positive integer")?;
        }
        ["logging", "max_backups"] => {
            settings.logging.max_backups = value
                .parse::<u32>()
                .context("Max backups must be a positive integer")?;
        }

        // Retry configuration
        ["retry", "enabled"] => {
            settings.retry.enabled = parse_bool(value)?;
        }
        ["retry", "max_attempts"] => {
            settings.retry.max_attempts = value
                .parse::<u32>()
                .context("Max attempts must be a positive integer")?;
        }
        ["retry", "backoff_factor"] => {
            settings.retry.backoff_factor = value
                .parse::<u32>()
                .context("Backoff factor must be a positive integer")?;
        }
        ["retry", "max_wait_seconds"] => {
            settings.retry.max_wait_seconds = value
                .parse::<u64>()
                .context("Max wait seconds must be a positive integer")?;
        }

        // Output configuration
        ["output", "format"] => {
            settings.output.format = value.to_string();
        }
        ["output", "color"] => {
            settings.output.color = parse_bool(value)?;
        }
        ["output", "verbose"] => {
            settings.output.verbose = parse_bool(value)?;
        }

        _ => {
            return Err(IndexerError::ConfigInvalidValue {
                field: key.to_string(),
                message: "Unknown configuration key".to_string(),
            });
        }
    }

    Ok(())
}

/// Get a configuration value by key path
fn get_value_by_key(settings: &Settings, key: &str) -> Result<String> {
    let parts: Vec<&str> = key.split('.').collect();

    let value = match parts.as_slice() {
        // Google configuration
        ["google", "enabled"] => settings
            .google
            .as_ref()
            .map(|g| g.enabled.to_string())
            .unwrap_or_else(|| "null".to_string()),
        ["google", "service_account_file"] => settings
            .google
            .as_ref()
            .and_then(|g| g.service_account_file.as_ref())
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "null".to_string()),
        ["google", "batch_size"] => settings
            .google
            .as_ref()
            .map(|g| g.batch_size.to_string())
            .unwrap_or_else(|| "null".to_string()),
        ["google", "quota", "daily_limit"] => settings
            .google
            .as_ref()
            .map(|g| g.quota.daily_limit.to_string())
            .unwrap_or_else(|| "null".to_string()),
        ["google", "quota", "rate_limit"] => settings
            .google
            .as_ref()
            .map(|g| g.quota.rate_limit.to_string())
            .unwrap_or_else(|| "null".to_string()),

        // IndexNow configuration
        ["indexnow", "enabled"] => settings
            .indexnow
            .as_ref()
            .map(|i| i.enabled.to_string())
            .unwrap_or_else(|| "null".to_string()),
        ["indexnow", "api_key"] => settings
            .indexnow
            .as_ref()
            .map(|i| i.api_key.clone())
            .unwrap_or_else(|| "null".to_string()),
        ["indexnow", "key_location"] => settings
            .indexnow
            .as_ref()
            .map(|i| i.key_location.clone())
            .unwrap_or_else(|| "null".to_string()),
        ["indexnow", "batch_size"] => settings
            .indexnow
            .as_ref()
            .map(|i| i.batch_size.to_string())
            .unwrap_or_else(|| "null".to_string()),

        // Sitemap configuration
        ["sitemap", "url"] => settings
            .sitemap
            .as_ref()
            .map(|s| s.url.clone())
            .unwrap_or_else(|| "null".to_string()),
        ["sitemap", "follow_index"] => settings
            .sitemap
            .as_ref()
            .map(|s| s.follow_index.to_string())
            .unwrap_or_else(|| "null".to_string()),

        // History configuration
        ["history", "enabled"] => settings.history.enabled.to_string(),
        ["history", "database_path"] => settings.history.database_path.clone(),
        ["history", "retention_days"] => settings.history.retention_days.to_string(),

        // Logging configuration
        ["logging", "level"] => settings.logging.level.clone(),
        ["logging", "file"] => settings.logging.file.clone(),
        ["logging", "max_size_mb"] => settings.logging.max_size_mb.to_string(),
        ["logging", "max_backups"] => settings.logging.max_backups.to_string(),

        // Retry configuration
        ["retry", "enabled"] => settings.retry.enabled.to_string(),
        ["retry", "max_attempts"] => settings.retry.max_attempts.to_string(),
        ["retry", "backoff_factor"] => settings.retry.backoff_factor.to_string(),
        ["retry", "max_wait_seconds"] => settings.retry.max_wait_seconds.to_string(),

        // Output configuration
        ["output", "format"] => settings.output.format.clone(),
        ["output", "color"] => settings.output.color.to_string(),
        ["output", "verbose"] => settings.output.verbose.to_string(),

        _ => {
            return Err(IndexerError::ConfigInvalidValue {
                field: key.to_string(),
                message: "Unknown configuration key".to_string(),
            });
        }
    };

    Ok(value)
}

/// Parse a boolean value from a string
fn parse_bool(value: &str) -> Result<bool> {
    match value.to_lowercase().as_str() {
        "true" | "yes" | "1" | "on" | "enabled" => Ok(true),
        "false" | "no" | "0" | "off" | "disabled" => Ok(false),
        _ => Err(IndexerError::ConfigInvalidValue {
            field: "boolean".to_string(),
            message: format!("Invalid boolean value: {}. Use true/false, yes/no, 1/0, on/off, or enabled/disabled", value),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bool() {
        assert_eq!(parse_bool("true").unwrap(), true);
        assert_eq!(parse_bool("yes").unwrap(), true);
        assert_eq!(parse_bool("1").unwrap(), true);
        assert_eq!(parse_bool("on").unwrap(), true);
        assert_eq!(parse_bool("enabled").unwrap(), true);

        assert_eq!(parse_bool("false").unwrap(), false);
        assert_eq!(parse_bool("no").unwrap(), false);
        assert_eq!(parse_bool("0").unwrap(), false);
        assert_eq!(parse_bool("off").unwrap(), false);
        assert_eq!(parse_bool("disabled").unwrap(), false);

        assert!(parse_bool("invalid").is_err());
    }

    #[test]
    fn test_set_and_get_value() {
        let mut settings = Settings::default();

        // Test setting and getting a simple value
        set_value_by_key(&mut settings, "logging.level", "debug").unwrap();
        assert_eq!(
            get_value_by_key(&settings, "logging.level").unwrap(),
            "debug"
        );

        // Test setting and getting a boolean
        set_value_by_key(&mut settings, "history.enabled", "false").unwrap();
        assert_eq!(
            get_value_by_key(&settings, "history.enabled").unwrap(),
            "false"
        );

        // Test setting and getting a numeric value
        set_value_by_key(&mut settings, "history.retention_days", "180").unwrap();
        assert_eq!(
            get_value_by_key(&settings, "history.retention_days").unwrap(),
            "180"
        );
    }

    #[test]
    fn test_nested_config_setting() {
        let mut settings = Settings::default();

        // Test Google nested config
        set_value_by_key(&mut settings, "google.enabled", "true").unwrap();
        set_value_by_key(&mut settings, "google.batch_size", "50").unwrap();

        assert!(settings.google.is_some());
        let google = settings.google.as_ref().unwrap();
        assert!(google.enabled);
        assert_eq!(google.batch_size, 50);

        // Test IndexNow nested config
        set_value_by_key(&mut settings, "indexnow.api_key", "test123456").unwrap();

        assert!(settings.indexnow.is_some());
        assert_eq!(settings.indexnow.as_ref().unwrap().api_key, "test123456");
    }

    #[test]
    fn test_invalid_key() {
        let mut settings = Settings::default();
        assert!(set_value_by_key(&mut settings, "invalid.key", "value").is_err());
        assert!(get_value_by_key(&settings, "invalid.key").is_err());
    }
}
