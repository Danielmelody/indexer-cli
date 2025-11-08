//! Google command - Google Indexing API operations.

use crate::api::google_indexing::{GoogleIndexingClient, MetadataResponse};
use crate::cli::args::{Cli, GoogleArgs, GoogleCommand, GoogleAction, GoogleSetupArgs, GoogleSubmitArgs, GoogleStatusArgs, SubmitArgs, ApiTarget, OutputFormat};
use crate::config::loader::{load_config, save_global_config, save_project_config};
use crate::config::settings::{Settings, GoogleConfig, QuotaConfig};
use crate::types::error::IndexerError;
use crate::types::Result;
use colored::Colorize;
use dialoguer::{Input, Confirm};
use std::path::PathBuf;

pub async fn run(args: GoogleArgs, cli: &Cli) -> Result<()> {
    match args.command {
        GoogleCommand::Setup(setup_args) => setup(setup_args, cli).await,
        GoogleCommand::Submit(submit_args) => submit(submit_args, cli).await,
        GoogleCommand::Status(status_args) => status(status_args, cli).await,
        GoogleCommand::Quota => quota(cli).await,
        GoogleCommand::Verify => verify(cli).await,
    }
}

/// Interactive setup for Google service account
pub async fn setup(args: GoogleSetupArgs, _cli: &Cli) -> Result<()> {
    println!("{}", "Google Indexing API Setup".cyan().bold());
    println!("{}", "=".repeat(60).dimmed());
    println!();

    // Get service account path
    let default_path = args.service_account.display().to_string();
    let sa_path_str: String = Input::new()
        .with_prompt("Service account JSON file path")
        .default(default_path)
        .interact()
        .map_err(|e| IndexerError::InternalError {
            message: format!("Failed to read input: {}", e),
        })?;

    // Validate file exists
    let path = PathBuf::from(&sa_path_str);
    if !path.exists() {
        return Err(IndexerError::GoogleServiceAccountNotFound { path });
    }

    println!();
    println!("{}", "Testing authentication...".dimmed());

    // Test authentication
    let _client = GoogleIndexingClient::new(path.clone()).await?;

    println!("{}", "✓ Authentication successful!".green());
    println!();

    // Ask for global or project config
    let use_global = if args.global {
        true
    } else {
        Confirm::new()
            .with_prompt("Save to global configuration? (No = project config)")
            .default(false)
            .interact()
            .map_err(|e| IndexerError::InternalError {
                message: format!("Failed to read input: {}", e),
            })?
    };

    // Load existing config or create new
    let mut config = load_config().unwrap_or_default();

    // Update Google config
    config.google = Some(GoogleConfig {
        enabled: true,
        service_account_file: path,
        quota: QuotaConfig::default(),
        batch_size: 100,
    });

    // Save config
    let config_path = if use_global {
        save_global_config(&config)?
    } else {
        save_project_config(&config)?
    };

    println!("{}", "✓ Configuration saved!".green());
    println!("  Location: {}", config_path.display().to_string().dimmed());
    println!();
    println!("{}", "Setup complete! You can now use Google Indexing API.".green().bold());
    println!();
    println!("Next steps:");
    println!("  • Run 'indexer google verify' to verify the setup");
    println!("  • Run 'indexer google submit <url>' to submit URLs");
    println!("  • Run 'indexer google quota' to check your quota");

    Ok(())
}

/// Submit URLs to Google (wrapper around main submit)
pub async fn submit(args: GoogleSubmitArgs, cli: &Cli) -> Result<()> {
    // Convert GoogleAction to the main GoogleAction type
    let google_action = match args.action {
        GoogleAction::UrlUpdated => crate::cli::args::GoogleAction::UrlUpdated,
        GoogleAction::UrlDeleted => crate::cli::args::GoogleAction::UrlDeleted,
    };

    // Convert to main SubmitArgs
    let submit_args = SubmitArgs {
        urls: args.urls,
        file: args.file,
        sitemap: args.sitemap,
        api: ApiTarget::Google, // Google only
        filter: args.filter,
        since: args.since,
        google_action,
        batch_size: args.batch_size,
        dry_run: args.dry_run,
        skip_history: args.skip_history,
        format: OutputFormat::Text,
    };

    // Delegate to main submit command
    crate::commands::submit::run(submit_args, cli).await
}

/// Check indexing status of URLs
pub async fn status(args: GoogleStatusArgs, _cli: &Cli) -> Result<()> {
    println!("{}", "Google Indexing Status".cyan().bold());
    println!("{}", "=".repeat(60).dimmed());
    println!();

    // Load config
    let config = load_config().map_err(|_| IndexerError::ConfigMissingField {
        field: "configuration".to_string(),
    })?;

    let google_config = require_google_config(&config)?;

    // Create client
    let client = GoogleIndexingClient::new(google_config.service_account_file.clone()).await?;

    // Collect URLs
    let urls = collect_urls(&args.urls, &args.file)?;

    if urls.is_empty() {
        println!("{}", "No URLs to check".yellow());
        return Ok(());
    }

    println!("Checking status for {} URL(s)...", urls.len());
    println!();

    // Query status for each URL
    let mut results = Vec::new();
    for url in &urls {
        match client.get_metadata(url).await {
            Ok(metadata) => results.push((url, Some(metadata))),
            Err(_) => results.push((url, None)),
        }
    }

    // Display results
    match args.format {
        OutputFormat::Text => display_status_table(&results),
        OutputFormat::Json => display_status_json(&results)?,
        OutputFormat::Csv => display_status_csv(&results)?,
    }

    Ok(())
}

/// Show quota usage
pub async fn quota(_cli: &Cli) -> Result<()> {
    println!("{}", "Google Indexing API Quota".cyan().bold());
    println!("{}", "=".repeat(60).dimmed());
    println!();

    // Load config
    let config = load_config().map_err(|_| IndexerError::ConfigMissingField {
        field: "configuration".to_string(),
    })?;

    let google_config = require_google_config(&config)?;

    // Create client
    let client = GoogleIndexingClient::new(google_config.service_account_file.clone()).await?;

    // Get quota info
    let quota_info = client.check_quota().await?;

    // Calculate remaining
    let remaining = quota_info.daily_publish_limit.saturating_sub(quota_info.daily_publish_used);

    println!("{:<20} {}", "Daily Limit:", google_config.quota.daily_limit.to_string().cyan());
    println!("{:<20} {}", "Used Today:", quota_info.daily_publish_used.to_string().yellow());
    println!("{:<20} {}", "Remaining:", remaining.to_string().green());
    println!("{:<20} {} req/min", "Rate Limit:", google_config.quota.rate_limit.to_string().cyan());
    println!();

    if remaining == 0 {
        println!("{}", "⚠ Quota exhausted! Quota resets at midnight Pacific Time.".yellow().bold());
    } else {
        let percent_used = (quota_info.daily_publish_used as f64 / quota_info.daily_publish_limit as f64) * 100.0;
        if percent_used >= 90.0 {
            println!("{}", format!("⚠ Warning: {:.1}% of daily quota used", percent_used).yellow());
        } else {
            println!("{}", format!("✓ {:.1}% of daily quota used", percent_used).green());
        }
    }

    Ok(())
}

/// Verify configuration and connectivity
pub async fn verify(_cli: &Cli) -> Result<()> {
    println!("{}", "Verifying Google Indexing API Configuration".cyan().bold());
    println!("{}", "=".repeat(60).dimmed());
    println!();

    // Check config exists
    let config = match load_config() {
        Ok(c) => {
            println!("{}", "✓ Configuration found".green());
            c
        }
        Err(_) => {
            println!("{}", "✗ Configuration not found".red());
            println!("  {}", "Run 'indexer google setup' to configure".dimmed());
            return Err(IndexerError::ConfigMissingField {
                field: "configuration".to_string(),
            });
        }
    };

    // Check Google config
    let google_config = match config.google {
        Some(ref g) if g.enabled => {
            println!("{}", "✓ Google configuration enabled".green());
            g
        }
        _ => {
            println!("{}", "✗ Google not configured or disabled".red());
            println!("  {}", "Run 'indexer google setup' to configure".dimmed());
            return Err(IndexerError::ConfigMissingField {
                field: "google".to_string(),
            });
        }
    };

    // Check service account file
    if !google_config.service_account_file.exists() {
        println!("{}", "✗ Service account file not found".red());
        println!("  Expected: {}", google_config.service_account_file.display().to_string().dimmed());
        return Err(IndexerError::GoogleServiceAccountNotFound {
            path: google_config.service_account_file.clone(),
        });
    }
    println!("{}", "✓ Service account file exists".green());

    // Test authentication
    print!("{}", "  Testing authentication... ".dimmed());
    let client = GoogleIndexingClient::new(google_config.service_account_file.clone()).await?;
    println!("{}", "✓".green());

    // Test API connectivity (try to get metadata for a test URL)
    print!("{}", "  Testing API connectivity... ".dimmed());
    match client.get_metadata("https://example.com").await {
        Ok(_) => {
            println!("{}", "✓".green());
        }
        Err(IndexerError::GoogleApiError { status_code: 404, .. }) => {
            // 404 is expected for non-existent URLs, but proves API connectivity
            println!("{}", "✓".green());
        }
        Err(IndexerError::GooglePermissionDenied { .. }) => {
            // Permission denied means auth works but URL not owned
            println!("{}", "✓".green());
            println!();
            println!("{}", "⚠ Note: Permission denied for example.com (expected)".yellow());
            println!("  {}", "Make sure to add the service account as an owner in Search Console".dimmed());
        }
        Err(e) => {
            println!("{}", "✗".red());
            println!();
            println!("{}", format!("Error: {}", e).red());
            return Err(e);
        }
    }

    println!();
    println!("{}", "✓ All checks passed!".green().bold());
    println!("  {}", "Google Indexing API is ready to use".green());
    println!();
    println!("Configuration details:");
    println!("  Service Account: {}", google_config.service_account_file.display().to_string().dimmed());
    println!("  Daily Limit: {}", google_config.quota.daily_limit.to_string().dimmed());
    println!("  Rate Limit: {} req/min", google_config.quota.rate_limit.to_string().dimmed());
    println!("  Batch Size: {}", google_config.batch_size.to_string().dimmed());

    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Require Google config or return error
fn require_google_config(config: &Settings) -> Result<&GoogleConfig> {
    config
        .google
        .as_ref()
        .filter(|g| g.enabled)
        .ok_or_else(|| IndexerError::ConfigMissingField {
            field: "google".to_string(),
        })
}

/// Collect URLs from command line args and file
fn collect_urls(urls: &[String], file: &Option<PathBuf>) -> Result<Vec<String>> {
    let mut all_urls = urls.to_vec();

    if let Some(path) = file {
        if !path.exists() {
            return Err(IndexerError::FileNotFound {
                path: path.clone(),
            });
        }

        let file_content = std::fs::read_to_string(path).map_err(|e| IndexerError::FileReadError {
            path: path.clone(),
            message: e.to_string(),
        })?;

        for line in file_content.lines() {
            let url = line.trim();
            // Skip empty lines and comments
            if !url.is_empty() && !url.starts_with('#') {
                all_urls.push(url.to_string());
            }
        }
    }

    Ok(all_urls)
}

/// Display status results in table format
fn display_status_table(results: &[(&String, Option<MetadataResponse>)]) {
    println!("{:<50} {:<15} {:<20}", "URL".bold(), "Status".bold(), "Last Updated".bold());
    println!("{}", "―".repeat(85).dimmed());

    for (url, metadata) in results {
        match metadata {
            Some(m) => {
                let status = "Indexed".green();
                let last_update = m
                    .url_notification_metadata
                    .latest_update
                    .as_ref()
                    .and_then(|u| u.notify_time.clone())
                    .unwrap_or_else(|| "Unknown".to_string());
                let last_update_display = if last_update == "Unknown" {
                    last_update.dimmed().to_string()
                } else {
                    last_update.cyan().to_string()
                };
                println!(
                    "{:<50} {:<15} {:<20}",
                    truncate(url, 50),
                    status,
                    last_update_display
                );
            }
            None => {
                println!(
                    "{:<50} {:<15} {:<20}",
                    truncate(url, 50),
                    "Not found".yellow(),
                    "-".dimmed()
                );
            }
        }
    }
}

/// Display status results in JSON format
fn display_status_json(results: &[(&String, Option<MetadataResponse>)]) -> Result<()> {
    let json_results: Vec<serde_json::Value> = results
        .iter()
        .map(|(url, metadata)| {
            match metadata {
                Some(m) => serde_json::json!({
                    "url": url,
                    "status": "indexed",
                    "last_updated": m.url_notification_metadata.latest_update.as_ref()
                        .and_then(|u| u.notify_time.clone())
                        .unwrap_or_else(|| "unknown".to_string()),
                    "notification_type": m.url_notification_metadata.latest_update.as_ref()
                        .map(|u| u.notification_type.clone())
                        .unwrap_or_else(|| "unknown".to_string()),
                }),
                None => serde_json::json!({
                    "url": url,
                    "status": "not_found",
                    "last_updated": null,
                    "notification_type": null,
                }),
            }
        })
        .collect();

    let json_output = serde_json::json!({
        "results": json_results,
        "total": results.len(),
        "indexed": results.iter().filter(|(_, m)| m.is_some()).count(),
        "not_found": results.iter().filter(|(_, m)| m.is_none()).count(),
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&json_output).map_err(|e| {
            IndexerError::JsonSerializationError {
                message: e.to_string(),
            }
        })?
    );

    Ok(())
}

/// Display status results in CSV format
fn display_status_csv(results: &[(&String, Option<MetadataResponse>)]) -> Result<()> {
    use std::io;

    let mut wtr = csv::Writer::from_writer(io::stdout());

    // Write header
    wtr.write_record(&["url", "status", "last_updated", "notification_type"])
        .map_err(|e| IndexerError::InternalError {
            message: format!("CSV write error: {}", e),
        })?;

    // Write data
    for (url, metadata) in results {
        match metadata {
            Some(m) => {
                let last_update = m
                    .url_notification_metadata
                    .latest_update
                    .as_ref()
                    .and_then(|u| u.notify_time.clone())
                    .unwrap_or_else(|| "unknown".to_string());
                let notification_type = m
                    .url_notification_metadata
                    .latest_update
                    .as_ref()
                    .map(|u| u.notification_type.clone())
                    .unwrap_or_else(|| "unknown".to_string());

                wtr.write_record(&[url, "indexed", &last_update, &notification_type])
                    .map_err(|e| IndexerError::InternalError {
                        message: format!("CSV write error: {}", e),
                    })?;
            }
            None => {
                wtr.write_record(&[url, "not_found", "", ""])
                    .map_err(|e| IndexerError::InternalError {
                        message: format!("CSV write error: {}", e),
                    })?;
            }
        }
    }

    wtr.flush().map_err(|e| IndexerError::InternalError {
        message: format!("CSV flush error: {}", e),
    })?;

    Ok(())
}

/// Truncate string to specified length with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a very long string", 10), "this is...");
    }

    #[test]
    fn test_collect_urls() {
        let urls = vec!["https://example.com".to_string()];
        let result = collect_urls(&urls, &None).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "https://example.com");
    }
}
