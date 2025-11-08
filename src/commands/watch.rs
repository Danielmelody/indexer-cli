//! Watch command - Monitor sitemap and auto-submit changes.
//!
//! This module implements a sitemap monitoring service that:
//! - Periodically checks a sitemap URL for new or modified URLs
//! - Automatically submits changes to configured indexing APIs
//! - Tracks sitemap state to detect additions and modifications
//! - Supports graceful shutdown with Ctrl+C
//! - Can run in daemon mode (background process)
//!
//! # Features
//!
//! - Configurable check interval
//! - Change detection based on lastmod timestamps
//! - Automatic submission to Google Indexing API and/or IndexNow
//! - History tracking to avoid duplicate submissions
//! - Clean signal handling for graceful shutdown
//!
//! # Example Usage
//!
//! ```bash
//! # Watch sitemap with default 1-hour interval
//! indexer-cli watch --sitemap https://example.com/sitemap.xml
//!
//! # Custom interval (5 minutes)
//! indexer-cli watch -s https://example.com/sitemap.xml -i 300
//!
//! # Use only IndexNow API
//! indexer-cli watch -s https://example.com/sitemap.xml -a indexnow
//!
//! # Run in daemon mode
//! indexer-cli watch -s https://example.com/sitemap.xml -d
//! ```

use crate::api::google_indexing::{GoogleIndexingClient, NotificationType};
use crate::api::indexnow::IndexNowClient;
use crate::cli::args::{ApiTarget, Cli, WatchArgs};
use crate::config::load_config;
use crate::database::init_database;
use crate::services::batch_submitter::{BatchConfig, BatchSubmitter, HistoryManager};
use crate::services::sitemap_parser::{SitemapParser, SitemapUrl};
use crate::types::Result;
use chrono::{DateTime, Utc};
use colored::Colorize;
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

/// Sitemap state tracker - stores URL locations and their last modification times
#[derive(Debug, Clone)]
struct SitemapState {
    /// Map of URL location to last modification timestamp
    urls: HashMap<String, Option<DateTime<Utc>>>,
    /// Timestamp of the last check
    last_check: DateTime<Utc>,
}

impl SitemapState {
    /// Create an empty sitemap state
    fn new() -> Self {
        Self {
            urls: HashMap::new(),
            last_check: Utc::now(),
        }
    }

    /// Create sitemap state from a list of URLs
    fn from_urls(urls: Vec<SitemapUrl>) -> Self {
        let url_map = urls
            .into_iter()
            .map(|u| (u.loc, u.lastmod))
            .collect();

        Self {
            urls: url_map,
            last_check: Utc::now(),
        }
    }

    /// Find new or modified URLs compared to previous state
    ///
    /// A URL is considered changed if:
    /// - It's new (not in previous state)
    /// - Its lastmod timestamp is newer than before
    /// - It previously had no lastmod but now has one
    fn find_changes(&self, new_state: &SitemapState) -> Vec<String> {
        let mut changed = Vec::new();

        for (url, new_lastmod) in &new_state.urls {
            match self.urls.get(url) {
                None => {
                    // New URL not seen before
                    changed.push(url.clone());
                }
                Some(old_lastmod) => {
                    // Check if lastmod timestamp changed
                    match (old_lastmod, new_lastmod) {
                        (Some(old), Some(new)) if new > old => {
                            // URL was modified
                            changed.push(url.clone());
                        }
                        (None, Some(_)) => {
                            // URL now has lastmod when it didn't before
                            changed.push(url.clone());
                        }
                        _ => {
                            // No change detected
                        }
                    }
                }
            }
        }

        changed
    }

    /// Count total URLs in this state
    fn url_count(&self) -> usize {
        self.urls.len()
    }
}

/// Execute the watch command
pub async fn run(args: WatchArgs, cli: &Cli) -> Result<()> {
    let config = load_config()?;

    if !cli.quiet {
        println!("{}", "Starting sitemap watcher...".cyan().bold());
        println!("  Sitemap: {}", args.sitemap);
        println!("  Interval: {}s ({})", args.interval, format_duration(args.interval));
        println!("  APIs: {:?}", args.api);
        if args.daemon {
            println!("  Mode: daemon (background)");
        }
        println!();
    }

    // Daemon mode is not yet implemented - warn the user
    if args.daemon {
        if !cli.quiet {
            println!(
                "{}",
                "⚠ Daemon mode is not yet implemented. Running in foreground mode.".yellow()
            );
            println!();
        }
    }

    // Setup signal handler for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        println!("\n{}", "Received shutdown signal, stopping...".yellow());
        r.store(false, Ordering::SeqCst);
    })
    .map_err(|e| crate::types::error::IndexerError::ConfigValidationError {
        message: format!("Failed to set up signal handler: {}", e),
    })?;

    // Initialize API clients based on configuration
    let google_client = if should_use_google(&args.api, &config) {
        match config.google.as_ref() {
            Some(google_config) if google_config.enabled => {
                if !cli.quiet {
                    println!("{}", "Initializing Google Indexing API...".cyan());
                }
                match GoogleIndexingClient::new(google_config.service_account_file.clone()).await {
                    Ok(client) => {
                        if !cli.quiet {
                            println!("  {} Google API client ready", "✓".green());
                        }
                        Some(Arc::new(client))
                    }
                    Err(e) => {
                        eprintln!(
                            "  {} Failed to initialize Google client: {}",
                            "✗".red(),
                            e
                        );
                        return Err(e);
                    }
                }
            }
            _ => {
                if !cli.quiet {
                    println!(
                        "  {} Google API not configured, skipping",
                        "!".yellow()
                    );
                }
                None
            }
        }
    } else {
        None
    };

    let indexnow_client = if should_use_indexnow(&args.api, &config) {
        match config.indexnow.as_ref() {
            Some(indexnow_config) if indexnow_config.enabled => {
                if !cli.quiet {
                    println!("{}", "Initializing IndexNow API...".cyan());
                }
                let endpoints = if indexnow_config.endpoints.is_empty() {
                    crate::constants::INDEXNOW_ENDPOINTS
                        .iter()
                        .map(|s| s.to_string())
                        .collect()
                } else {
                    indexnow_config.endpoints.clone()
                };

                match IndexNowClient::new(
                    indexnow_config.api_key.clone(),
                    indexnow_config.key_location.clone(),
                    endpoints,
                ) {
                    Ok(client) => {
                        if !cli.quiet {
                            println!("  {} IndexNow API client ready", "✓".green());
                        }
                        Some(Arc::new(client))
                    }
                    Err(e) => {
                        eprintln!(
                            "  {} Failed to initialize IndexNow client: {}",
                            "✗".red(),
                            e
                        );
                        return Err(e);
                    }
                }
            }
            _ => {
                if !cli.quiet {
                    println!(
                        "  {} IndexNow API not configured, skipping",
                        "!".yellow()
                    );
                }
                None
            }
        }
    } else {
        None
    };

    // Check that at least one API is configured
    if google_client.is_none() && indexnow_client.is_none() {
        return Err(crate::types::error::IndexerError::ConfigValidationError {
            message: "No API clients configured. Run 'indexer-cli init' to configure APIs."
                .to_string(),
        });
    }

    // Initialize database and history manager
    let db_path = Path::new(&config.history.database_path);
    let db_conn = init_database(db_path)?;
    let history = Arc::new(HistoryManager::new(db_conn));

    // Configure batch submitter with history checking enabled
    let batch_config = BatchConfig::new()
        .with_check_history(true)
        .with_progress_bar(!cli.quiet);

    let submitter = BatchSubmitter::new(
        google_client,
        indexnow_client,
        history,
        batch_config,
    );

    // Parse initial sitemap state
    if !cli.quiet {
        println!();
        println!("{}", "Fetching initial sitemap state...".cyan());
    }

    let parser = SitemapParser::new()?;
    let initial_result = parser.parse_sitemap(&args.sitemap, None).await?;
    let mut current_state = SitemapState::from_urls(initial_result.urls);

    if !cli.quiet {
        println!(
            "  {} Initial sitemap parsed: {} URLs",
            "✓".green(),
            current_state.url_count()
        );
        println!();
        println!(
            "{}",
            "Watching for changes... (Press Ctrl+C to stop)".green().bold()
        );
        println!();
    }

    // Main watch loop
    let mut iteration = 0;
    while running.load(Ordering::SeqCst) {
        // Sleep for the configured interval
        sleep(Duration::from_secs(args.interval)).await;

        // Check if still running (might have been interrupted during sleep)
        if !running.load(Ordering::SeqCst) {
            break;
        }

        iteration += 1;
        let check_time = Utc::now();

        if !cli.quiet {
            println!(
                "[{}] {} Checking sitemap...",
                check_time.format("%Y-%m-%d %H:%M:%S"),
                "→".cyan()
            );
        }

        // Parse sitemap again
        let new_result = match parser.parse_sitemap(&args.sitemap, None).await {
            Ok(result) => result,
            Err(e) => {
                eprintln!("  {} Error parsing sitemap: {}", "✗".red(), e);
                if !cli.quiet {
                    println!("  Retrying on next interval...");
                }
                continue;
            }
        };

        let new_state = SitemapState::from_urls(new_result.urls);

        if !cli.quiet {
            println!("  {} Parsed {} URLs", "✓".green(), new_state.url_count());
        }

        // Detect changes
        let changed_urls = current_state.find_changes(&new_state);

        if changed_urls.is_empty() {
            if !cli.quiet {
                println!("  {} No changes detected", "→".cyan());
            }
        } else {
            if !cli.quiet {
                println!(
                    "  {} {} new/modified URLs detected",
                    "!".yellow(),
                    changed_urls.len()
                );

                // Show first few changed URLs
                let display_count = changed_urls.len().min(5);
                for url in changed_urls.iter().take(display_count) {
                    println!("    • {}", url);
                }
                if changed_urls.len() > display_count {
                    println!("    ... and {} more", changed_urls.len() - display_count);
                }
            }

            // Submit changes to APIs
            if !cli.quiet {
                println!("  {} Submitting changes...", "→".cyan());
            }

            match submitter
                .submit_to_all(changed_urls.clone(), NotificationType::UrlUpdated)
                .await
            {
                Ok(result) => {
                    if !cli.quiet {
                        println!(
                            "  {} Submitted: {} successful, {} failed, {} skipped",
                            "✓".green(),
                            result.total_successful(),
                            result.total_failed(),
                            result.skipped
                        );
                    }
                }
                Err(e) => {
                    eprintln!("  {} Submission failed: {}", "✗".red(), e);
                }
            }
        }

        // Update current state for next iteration
        current_state = new_state;
        current_state.last_check = check_time;

        if !cli.quiet {
            println!();
        }
    }

    if !cli.quiet {
        println!();
        println!("{}", "Watch stopped.".cyan().bold());
        println!("Total checks performed: {}", iteration);
    }

    Ok(())
}

/// Check if Google API should be used based on args and config
fn should_use_google(api_target: &ApiTarget, config: &crate::config::Settings) -> bool {
    // Check if explicitly requested or using "all"
    let requested = matches!(api_target, ApiTarget::Google | ApiTarget::All);

    // Check if Google config exists and is enabled
    let enabled = config
        .google
        .as_ref()
        .map(|g| g.enabled)
        .unwrap_or(false);

    // Check if service account is configured
    let configured = config
        .google
        .as_ref()
        .map(|g| !g.service_account_file.as_os_str().is_empty())
        .unwrap_or(false);

    requested && enabled && configured
}

/// Check if IndexNow API should be used based on args and config
fn should_use_indexnow(api_target: &ApiTarget, config: &crate::config::Settings) -> bool {
    // Check if explicitly requested or using "all"
    let requested = matches!(api_target, ApiTarget::IndexNow | ApiTarget::All);

    // Check if IndexNow config exists and is enabled
    let enabled = config
        .indexnow
        .as_ref()
        .map(|i| i.enabled)
        .unwrap_or(false);

    // Check if API key is configured
    let configured = config
        .indexnow
        .as_ref()
        .map(|i| !i.api_key.is_empty())
        .unwrap_or(false);

    requested && enabled && configured
}

/// Format duration in seconds to human-readable format
fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        let minutes = seconds / 60;
        let secs = seconds % 60;
        if secs == 0 {
            format!("{}m", minutes)
        } else {
            format!("{}m {}s", minutes, secs)
        }
    } else {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        if minutes == 0 {
            format!("{}h", hours)
        } else {
            format!("{}h {}m", hours, minutes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sitemap_state_new_urls() {
        let base_time = Utc::now();

        let old_urls = vec![SitemapUrl {
            loc: "https://example.com/page1".to_string(),
            lastmod: Some(base_time),
            changefreq: None,
            priority: None,
        }];

        let new_urls = vec![
            SitemapUrl {
                loc: "https://example.com/page1".to_string(),
                lastmod: Some(base_time),
                changefreq: None,
                priority: None,
            },
            SitemapUrl {
                loc: "https://example.com/page2".to_string(),
                lastmod: None,
                changefreq: None,
                priority: None,
            },
        ];

        let old_state = SitemapState::from_urls(old_urls);
        let new_state = SitemapState::from_urls(new_urls);

        let changes = old_state.find_changes(&new_state);
        assert_eq!(changes.len(), 1);
        assert!(changes.contains(&"https://example.com/page2".to_string()));
    }

    #[test]
    fn test_sitemap_state_modified_urls() {
        use std::ops::Add;

        let base_time = Utc::now();
        let later_time = base_time.add(chrono::Duration::hours(1));

        let old_urls = vec![SitemapUrl {
            loc: "https://example.com/page1".to_string(),
            lastmod: Some(base_time),
            changefreq: None,
            priority: None,
        }];

        let new_urls = vec![SitemapUrl {
            loc: "https://example.com/page1".to_string(),
            lastmod: Some(later_time),
            changefreq: None,
            priority: None,
        }];

        let old_state = SitemapState::from_urls(old_urls);
        let new_state = SitemapState::from_urls(new_urls);

        let changes = old_state.find_changes(&new_state);
        assert_eq!(changes.len(), 1);
        assert!(changes.contains(&"https://example.com/page1".to_string()));
    }

    #[test]
    fn test_sitemap_state_no_changes() {
        let base_time = Utc::now();

        let old_urls = vec![SitemapUrl {
            loc: "https://example.com/page1".to_string(),
            lastmod: Some(base_time),
            changefreq: None,
            priority: None,
        }];

        let new_urls = vec![SitemapUrl {
            loc: "https://example.com/page1".to_string(),
            lastmod: Some(base_time),
            changefreq: None,
            priority: None,
        }];

        let old_state = SitemapState::from_urls(old_urls);
        let new_state = SitemapState::from_urls(new_urls);

        let changes = old_state.find_changes(&new_state);
        assert_eq!(changes.len(), 0);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(60), "1m");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3600), "1h");
        assert_eq!(format_duration(3660), "1h 1m");
        assert_eq!(format_duration(7200), "2h");
        assert_eq!(format_duration(7320), "2h 2m");
    }

    #[test]
    fn test_sitemap_state_url_count() {
        let urls = vec![
            SitemapUrl {
                loc: "https://example.com/page1".to_string(),
                lastmod: None,
                changefreq: None,
                priority: None,
            },
            SitemapUrl {
                loc: "https://example.com/page2".to_string(),
                lastmod: None,
                changefreq: None,
                priority: None,
            },
        ];

        let state = SitemapState::from_urls(urls);
        assert_eq!(state.url_count(), 2);
    }
}
