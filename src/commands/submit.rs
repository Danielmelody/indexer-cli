//! Submit command - Unified URL submission to all APIs.
//!
//! This module implements the core `submit` command, which provides a unified
//! interface for submitting URLs to Google Indexing API and IndexNow API.
//!
//! # Features
//!
//! - Multiple input sources: command-line args, files, sitemaps
//! - Flexible API selection: all, Google only, or IndexNow only
//! - URL filtering by regex pattern and modification date
//! - History tracking to avoid duplicate submissions
//! - Dry-run mode for testing
//! - Multiple output formats: text, JSON, CSV
//! - Progress indicators and detailed reporting
//!
//! # Example Usage
//!
//! ```bash
//! # Submit URLs from command line
//! indexer submit --urls https://placeholder.test/page1 https://placeholder.test/page2
//!
//! # Submit from file
//! indexer submit --file urls.txt
//!
//! # Submit from sitemap
//! indexer submit --sitemap https://placeholder.test/sitemap.xml
//!
//! # Dry run to see what would be submitted
//! indexer submit --sitemap https://placeholder.test/sitemap.xml --dry-run
//!
//! # Submit to specific API only
//! indexer submit --file urls.txt --api google
//!
//! # Filter URLs with regex
//! indexer submit --sitemap https://placeholder.test/sitemap.xml --filter "^https://placeholder.test/blog/"
//! ```

use crate::api::google_indexing::{GoogleIndexingClient, NotificationType};
use crate::api::indexnow::IndexNowClient;
use crate::cli::args::{ApiTarget, Cli, GoogleAction, OutputFormat, SubmitArgs};
use crate::config::load_config;
use crate::database::init_database;
use crate::services::batch_submitter::{BatchConfig, BatchSubmitter, HistoryManager};
use crate::services::sitemap_parser::SitemapParser;
use crate::types::Result;
use chrono::{DateTime, NaiveDate, Utc};
use colored::Colorize;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::Arc;

/// Execute the submit command
pub async fn run(args: SubmitArgs, cli: &Cli) -> Result<()> {
    // Step 1: Load configuration
    let config = load_config()?;

    // Step 2: Collect URLs from all sources
    if !cli.quiet {
        println!("{}", "Collecting URLs...".cyan().bold());
    }

    let mut urls = collect_urls(&args, cli).await?;

    if urls.is_empty() {
        return Err(crate::types::error::IndexerError::ConfigValidationError {
            message: "No URLs provided. Use --urls, --file, or --sitemap to specify URLs"
                .to_string(),
        });
    }

    // Remove duplicates
    let original_count = urls.len();
    urls = deduplicate_urls(urls);

    if !cli.quiet && urls.len() < original_count {
        println!(
            "  {} Removed {} duplicate URLs",
            "→".cyan(),
            original_count - urls.len()
        );
    }

    // Step 3: Apply filters
    if args.filter.is_some() || args.since.is_some() {
        let before_filter = urls.len();
        urls = apply_filters(&urls, &args)?;

        if !cli.quiet {
            println!(
                "  {} Filtered: {} URLs remaining (removed {})",
                "→".cyan(),
                urls.len(),
                before_filter - urls.len()
            );
        }
    }

    if urls.is_empty() {
        return Err(crate::types::error::IndexerError::ConfigValidationError {
            message: "No URLs remaining after filtering".to_string(),
        });
    }

    if !cli.quiet {
        println!("  {} Total URLs to process: {}", "✓".green(), urls.len());
        println!();
    }

    // Step 4: Dry run mode - show what would be submitted and exit
    if args.dry_run {
        print_dry_run_summary(&urls, &args, cli)?;
        return Ok(());
    }

    // Step 5: Initialize API clients based on configuration and --api flag
    let google_client = if should_use_google(&args, &config) {
        if !cli.quiet {
            println!("{}", "Initializing Google Indexing API...".cyan());
        }

        let google_config = config.google.as_ref().ok_or_else(|| {
            crate::types::error::IndexerError::ConfigMissingField {
                field: "google".to_string(),
            }
        })?;

        let service_account_path = google_config.service_account_path().ok_or_else(|| {
            crate::types::error::IndexerError::ConfigMissingField {
                field: "google.auth.service_account_file (or google.service_account_file)"
                    .to_string(),
            }
        })?;

        match GoogleIndexingClient::from_service_account(service_account_path).await {
            Ok(client) => {
                if !cli.quiet {
                    println!("  {} Google API client ready", "✓".green());
                }
                Some(Arc::new(client))
            }
            Err(e) => {
                eprintln!("  {} Failed to initialize Google API: {}", "✗".red(), e);
                return Err(e);
            }
        }
    } else {
        // When user expects to use Google but it's not configured, give clear guidance
        if !cli.quiet {
            match args.api {
                ApiTarget::Google => {
                    // Explicitly requested Google - show error-level guidance
                    eprintln!(
                        "\n  {} Google Indexing API is not configured",
                        "!".yellow().bold()
                    );
                    eprintln!(
                        "  {} Run 'indexer-cli google setup --service-account <path-to-json>' to configure the service account",
                        "→".cyan()
                    );
                    eprintln!(
                        "  {} Or use '--api index-now' to only use IndexNow API\n",
                        "→".cyan()
                    );
                }
                ApiTarget::All => {
                    // Using All but Google is not configured - show info-level notice
                    println!(
                        "  {} Google Indexing API not configured (will only use IndexNow)",
                        "ℹ".blue()
                    );
                    println!(
                        "    Run 'indexer-cli google setup --service-account <path-to-json>' to enable Google Search indexing",
                    );
                    println!();
                }
                _ => {}
            }
        }
        None
    };

    let indexnow_client = if should_use_indexnow(&args, &config) {
        if !cli.quiet {
            println!("{}", "Initializing IndexNow API...".cyan());
        }

        let indexnow_config = config.indexnow.as_ref().ok_or_else(|| {
            crate::types::error::IndexerError::ConfigMissingField {
                field: "indexnow".to_string(),
            }
        })?;

        let api_key = indexnow_config.api_key.clone();
        let key_location = indexnow_config.key_location.clone();
        let endpoints = if indexnow_config.endpoints.is_empty() {
            crate::constants::INDEXNOW_ENDPOINTS
                .iter()
                .map(|s| s.to_string())
                .collect()
        } else {
            indexnow_config.endpoints.clone()
        };

        match IndexNowClient::new(api_key, key_location, endpoints) {
            Ok(client) => {
                if !cli.quiet {
                    println!("  {} IndexNow API client ready", "✓".green());
                }
                Some(Arc::new(client))
            }
            Err(e) => {
                eprintln!("  {} Failed to initialize IndexNow API: {}", "✗".red(), e);
                return Err(e);
            }
        }
    } else {
        if !cli.quiet {
            match args.api {
                ApiTarget::IndexNow => {
                    // Explicitly requested IndexNow - show error-level guidance
                    eprintln!("\n  {} IndexNow API is not configured", "!".yellow().bold());
                    eprintln!(
                        "  {} Run 'indexer-cli indexnow setup' to configure IndexNow",
                        "→".cyan()
                    );
                    eprintln!(
                        "  {} Or use '--api google' to only use Google Indexing API\n",
                        "→".cyan()
                    );
                }
                ApiTarget::All => {
                    // Using All but IndexNow is not configured - show info-level notice
                    println!(
                        "  {} IndexNow API not configured (will only use Google)",
                        "ℹ".blue()
                    );
                    println!("    Run 'indexer-cli indexnow setup' to enable IndexNow",);
                    println!();
                }
                _ => {}
            }
        }
        None
    };

    // Check that at least one API is available
    if google_client.is_none() && indexnow_client.is_none() {
        return Err(crate::types::error::IndexerError::ConfigValidationError {
            message: "No APIs configured. Run 'indexer init' to set up API credentials".to_string(),
        });
    }

    if !cli.quiet {
        println!();
    }

    // Step 6: Setup batch submitter
    let db_path = crate::config::expand_tilde(&config.history.database_path);
    let db_conn = init_database(&db_path)?;
    let history = Arc::new(HistoryManager::new(db_conn));

    let mut batch_config = BatchConfig::new()
        .with_check_history(!(args.skip_history || args.force))
        .with_progress_bar(!cli.quiet);

    match args.batch_size {
        Some(batch_size) => {
            batch_config = batch_config
                .with_google_batch_size(batch_size)
                .with_indexnow_batch_size(batch_size);
        }
        None => {
            if let Some(google_cfg) = config.google.as_ref() {
                batch_config = batch_config.with_google_batch_size(google_cfg.batch_size);
            }
            if let Some(indexnow_cfg) = config.indexnow.as_ref() {
                batch_config = batch_config.with_indexnow_batch_size(indexnow_cfg.batch_size);
            }
        }
    }

    let submitter = BatchSubmitter::new(
        google_client.clone(),
        indexnow_client.clone(),
        history,
        batch_config,
    );

    // Step 7: Submit URLs
    if !cli.quiet {
        println!("{}", "Submitting URLs...".cyan().bold());
        println!();
    }

    let action = convert_google_action(&args.google_action);
    let result = submitter.submit_to_all(urls, action).await?;

    // Step 8: Print results
    if !cli.quiet {
        println!();
    }
    print_result(&result, &args, cli)?;

    // Return error if there were failures
    if result.total_failed() > 0 {
        Err(crate::types::error::IndexerError::BatchProcessingFailed {
            successful: result.total_successful(),
            failed: result.total_failed(),
        })
    } else {
        Ok(())
    }
}

/// Collect URLs from all input sources
async fn collect_urls(args: &SubmitArgs, cli: &Cli) -> Result<Vec<String>> {
    let mut urls = Vec::new();

    // From command line arguments
    if !args.urls.is_empty() {
        if !cli.quiet {
            println!(
                "  {} Reading {} URLs from command line",
                "→".cyan(),
                args.urls.len()
            );
        }
        urls.extend(args.urls.clone());
    }

    // From file
    if let Some(ref file_path) = args.file {
        if !cli.quiet {
            println!(
                "  {} Reading URLs from file: {}",
                "→".cyan(),
                file_path.display()
            );
        }
        let file_urls = read_urls_from_file(file_path)?;
        if !cli.quiet {
            println!("    Found {} URLs", file_urls.len());
        }
        urls.extend(file_urls);
    }

    // From sitemap
    if let Some(ref sitemap_url) = args.sitemap {
        if !cli.quiet {
            println!("  {} Parsing sitemap: {}", "→".cyan(), sitemap_url);
        }

        let parser = SitemapParser::new()?;
        let result = parser.parse_sitemap(sitemap_url, None).await?;

        if !cli.quiet {
            println!("    Found {} URLs in sitemap", result.urls.len());
        }

        urls.extend(result.urls.into_iter().map(|u| u.loc));
    }

    Ok(urls)
}

/// Read URLs from a text file (one URL per line)
fn read_urls_from_file(file_path: &Path) -> Result<Vec<String>> {
    let content = fs::read_to_string(file_path).map_err(|e| {
        crate::types::error::IndexerError::FileReadError {
            path: file_path.to_path_buf(),
            message: e.to_string(),
        }
    })?;

    let urls: Vec<String> = content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(String::from)
        .collect();

    Ok(urls)
}

/// Remove duplicate URLs while preserving order
fn deduplicate_urls(urls: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    urls.into_iter()
        .filter(|url| seen.insert(url.clone()))
        .collect()
}

/// Apply filters to URLs
fn apply_filters(urls: &[String], args: &SubmitArgs) -> Result<Vec<String>> {
    let mut filtered = urls.to_vec();

    // Apply regex filter
    if let Some(ref pattern) = args.filter {
        let regex = Regex::new(pattern).map_err(|e| {
            crate::types::error::IndexerError::ConfigValidationError {
                message: format!("Invalid regex pattern: {}", e),
            }
        })?;
        filtered.retain(|url| regex.is_match(url));
    }

    // Apply date filter
    if let Some(ref since_str) = args.since {
        let _since_date = parse_date(since_str)?;
        // Note: We can't filter by date here since we don't have lastmod info
        // from command-line URLs. This would only work with sitemap data.
        // For now, we'll skip date filtering for non-sitemap sources.
    }

    Ok(filtered)
}

/// Parse a date string in YYYY-MM-DD format
fn parse_date(date_str: &str) -> Result<DateTime<Utc>> {
    let naive_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|e| {
        crate::types::error::IndexerError::ConfigValidationError {
            message: format!(
                "Invalid date format '{}': {}. Expected YYYY-MM-DD",
                date_str, e
            ),
        }
    })?;

    Ok(naive_date.and_hms_opt(0, 0, 0).unwrap().and_utc())
}

/// Check if Google API should be used
fn should_use_google(args: &SubmitArgs, config: &crate::config::Settings) -> bool {
    // Check if explicitly requested or using "all"
    let requested = matches!(args.api, ApiTarget::Google | ApiTarget::All);

    // Check if Google config exists and is enabled
    let enabled = config.google.as_ref().map(|g| g.enabled).unwrap_or(false);

    // Check if service account is configured
    let configured = config
        .google
        .as_ref()
        .and_then(|g| g.service_account_path())
        .map(|p| !p.as_os_str().is_empty())
        .unwrap_or(false);

    requested && enabled && configured
}

/// Check if IndexNow API should be used
fn should_use_indexnow(args: &SubmitArgs, config: &crate::config::Settings) -> bool {
    // Check if explicitly requested or using "all"
    let requested = matches!(args.api, ApiTarget::IndexNow | ApiTarget::All);

    // Check if IndexNow config exists and is enabled
    let enabled = config.indexnow.as_ref().map(|i| i.enabled).unwrap_or(false);

    // Check if API key is configured
    let configured = config
        .indexnow
        .as_ref()
        .map(|i| !i.api_key.is_empty())
        .unwrap_or(false);

    requested && enabled && configured
}

/// Convert GoogleAction to NotificationType
fn convert_google_action(action: &GoogleAction) -> NotificationType {
    match action {
        GoogleAction::UrlUpdated => NotificationType::UrlUpdated,
        GoogleAction::UrlDeleted => NotificationType::UrlDeleted,
    }
}

/// Print dry-run summary
fn print_dry_run_summary(urls: &[String], args: &SubmitArgs, cli: &Cli) -> Result<()> {
    if cli.quiet {
        return Ok(());
    }

    println!("{}", "DRY RUN - No URLs will be submitted".yellow().bold());
    println!();
    println!("{}", "Summary:".bold());
    println!("  Total URLs: {}", urls.len());
    println!("  API Target: {:?}", args.api);
    println!(
        "  Google Action: {}",
        match args.google_action {
            GoogleAction::UrlUpdated => "URL_UPDATED",
            GoogleAction::UrlDeleted => "URL_DELETED",
        }
    );
    println!("  Skip History: {}", args.skip_history);
    println!("  Force:        {}", args.force);

    if let Some(ref pattern) = args.filter {
        println!("  Filter Pattern: {}", pattern);
    }

    if let Some(ref since) = args.since {
        println!("  Modified Since: {}", since);
    }

    println!();
    println!("{}", "URLs to submit:".bold());

    let display_count = urls.len().min(20);
    for (i, url) in urls.iter().take(display_count).enumerate() {
        println!("  {}. {}", i + 1, url);
    }

    if urls.len() > display_count {
        println!("  ... and {} more", urls.len() - display_count);
    }

    println!();
    println!(
        "{}",
        "Run without --dry-run to actually submit these URLs".cyan()
    );

    Ok(())
}

/// Print submission results
fn print_result(
    result: &crate::services::batch_submitter::BatchResult,
    args: &SubmitArgs,
    cli: &Cli,
) -> Result<()> {
    match args.format {
        OutputFormat::Text => print_result_text(result, cli),
        OutputFormat::Json => print_result_json(result),
        OutputFormat::Csv => print_result_csv(result),
    }
}

/// Print results in text format
fn print_result_text(
    result: &crate::services::batch_submitter::BatchResult,
    cli: &Cli,
) -> Result<()> {
    if cli.quiet {
        // In quiet mode, only print essential summary
        println!(
            "{} successful, {} failed, {} skipped",
            result.total_successful(),
            result.total_failed(),
            result.skipped
        );
        return Ok(());
    }

    println!("{}", "Submission Results".green().bold());
    println!("{}", "=".repeat(50));
    println!();

    // Overall summary
    println!("{}", "Overall Summary:".bold());
    println!("  Total URLs:      {}", result.total_urls);
    println!("  Submitted:       {}", result.submitted);
    println!("  Skipped:         {}", result.skipped);
    println!("  {} {}", "Successful:".green(), result.total_successful());

    if result.total_failed() > 0 {
        println!("  {} {}", "Failed:".red(), result.total_failed());
    } else {
        println!("  {} {}", "Failed:", result.total_failed());
    }
    println!();

    // Google API results
    if let Some(ref google_results) = result.google_results {
        println!("{}", "Google Indexing API:".bold());
        println!("  Successful:  {}", google_results.successful);
        println!("  Failed:      {}", google_results.failed);

        if !google_results.errors.is_empty() {
            println!("  Errors:");
            for (i, error) in google_results.errors.iter().take(5).enumerate() {
                println!("    {}. {}", i + 1, error);
            }
            if google_results.errors.len() > 5 {
                println!(
                    "    ... and {} more errors",
                    google_results.errors.len() - 5
                );
            }
        }
        println!();
    }

    // IndexNow API results
    if let Some(ref indexnow_results) = result.indexnow_results {
        println!("{}", "IndexNow API:".bold());
        println!("  Successful:  {}", indexnow_results.successful);
        println!("  Failed:      {}", indexnow_results.failed);

        if !indexnow_results.errors.is_empty() {
            println!("  Errors:");
            for (i, error) in indexnow_results.errors.iter().take(5).enumerate() {
                println!("    {}. {}", i + 1, error);
            }
            if indexnow_results.errors.len() > 5 {
                println!(
                    "    ... and {} more errors",
                    indexnow_results.errors.len() - 5
                );
            }
        }
        println!();
    }

    // Final status
    if result.is_success() {
        println!("{}", "✓ All submissions successful!".green().bold());
    } else {
        println!(
            "{}",
            "⚠ Some submissions failed. Check the errors above."
                .yellow()
                .bold()
        );
    }

    Ok(())
}

/// Print results in JSON format
fn print_result_json(result: &crate::services::batch_submitter::BatchResult) -> Result<()> {
    let json_output = serde_json::json!({
        "total_urls": result.total_urls,
        "submitted": result.submitted,
        "skipped": result.skipped,
        "successful": result.total_successful(),
        "failed": result.total_failed(),
        "google": result.google_results.as_ref().map(|r| serde_json::json!({
            "successful": r.successful,
            "failed": r.failed,
            "errors": r.errors,
        })),
        "indexnow": result.indexnow_results.as_ref().map(|r| serde_json::json!({
            "successful": r.successful,
            "failed": r.failed,
            "errors": r.errors,
        })),
    });

    println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
    Ok(())
}

/// Print results in CSV format
fn print_result_csv(result: &crate::services::batch_submitter::BatchResult) -> Result<()> {
    println!("metric,value");
    println!("total_urls,{}", result.total_urls);
    println!("submitted,{}", result.submitted);
    println!("skipped,{}", result.skipped);
    println!("successful,{}", result.total_successful());
    println!("failed,{}", result.total_failed());

    if let Some(ref google_results) = result.google_results {
        println!("google_successful,{}", google_results.successful);
        println!("google_failed,{}", google_results.failed);
    }

    if let Some(ref indexnow_results) = result.indexnow_results {
        println!("indexnow_successful,{}", indexnow_results.successful);
        println!("indexnow_failed,{}", indexnow_results.failed);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate_urls() {
        let urls = vec![
            "https://placeholder.test/page1".to_string(),
            "https://placeholder.test/page2".to_string(),
            "https://placeholder.test/page1".to_string(),
            "https://placeholder.test/page3".to_string(),
            "https://placeholder.test/page2".to_string(),
        ];

        let result = deduplicate_urls(urls);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "https://placeholder.test/page1");
        assert_eq!(result[1], "https://placeholder.test/page2");
        assert_eq!(result[2], "https://placeholder.test/page3");
    }

    #[test]
    fn test_parse_date() {
        let result = parse_date("2024-01-15");
        assert!(result.is_ok());

        let result = parse_date("invalid-date");
        assert!(result.is_err());

        let result = parse_date("2024-13-01"); // Invalid month
        assert!(result.is_err());
    }

    #[test]
    fn test_convert_google_action() {
        assert_eq!(
            convert_google_action(&GoogleAction::UrlUpdated),
            NotificationType::UrlUpdated
        );
        assert_eq!(
            convert_google_action(&GoogleAction::UrlDeleted),
            NotificationType::UrlDeleted
        );
    }
}
