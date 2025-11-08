//! History command - Submission history management.

use crate::cli::args::{Cli, ExportFormat, HistoryArgs, HistoryCommand, OutputFormat};
use crate::config::loader::load_config;
use crate::database::models::{ApiType, SubmissionStatus};
use crate::database::queries::{
    count_submissions, delete_old_submissions, get_submissions_stats, list_submissions,
    SubmissionFilters,
};
use crate::database::schema::init_database;
use crate::types::{IndexerError, Result};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use colored::Colorize;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

/// Run the history command
pub async fn run(args: HistoryArgs, cli: &Cli) -> Result<()> {
    match args.command {
        HistoryCommand::List(list_args) => {
            let conn = get_database_connection(cli)?;

            // Build filters with limit
            let filters = SubmissionFilters::new().limit(list_args.limit);

            // Query submissions
            let submissions = list_submissions(&conn, &filters)?;

            if submissions.is_empty() {
                if !cli.quiet {
                    println!("{}", "No submissions found in history.".yellow());
                }
                return Ok(());
            }

            // Display based on format
            match list_args.format {
                OutputFormat::Text => display_as_table(&submissions, cli.quiet)?,
                OutputFormat::Json => display_as_json(&submissions)?,
                OutputFormat::Csv => display_as_csv(&submissions)?,
            }
        }
        HistoryCommand::Search(search_args) => {
            let conn = get_database_connection(cli)?;

            // Build filters from search arguments
            let mut filters = SubmissionFilters::new().limit(search_args.limit);

            if let Some(url) = search_args.url {
                filters = filters.url_pattern(if url.contains('%') {
                    url
                } else {
                    format!("%{}%", url)
                });
            }

            if let Some(api_str) = search_args.api {
                filters = filters.api(parse_api_filter(&api_str)?);
            }

            if let Some(status_str) = search_args.status {
                filters = filters.status(parse_status_filter(&status_str)?);
            }

            if let Some(since) = parse_date(search_args.since)? {
                filters = filters.after(since);
            }

            if let Some(until) = parse_date(search_args.until)? {
                filters = filters.before(until);
            }

            // Query submissions
            let submissions = list_submissions(&conn, &filters)?;

            if submissions.is_empty() {
                if !cli.quiet {
                    println!("{}", "No submissions found matching the filters.".yellow());
                }
                return Ok(());
            }

            if !cli.quiet {
                println!(
                    "{} {}",
                    "Found".green().bold(),
                    format!("{} submission(s)", submissions.len()).cyan()
                );
                println!();
            }

            // Display based on format
            match search_args.format {
                OutputFormat::Text => display_as_table(&submissions, cli.quiet)?,
                OutputFormat::Json => display_as_json(&submissions)?,
                OutputFormat::Csv => display_as_csv(&submissions)?,
            }
        }
        HistoryCommand::Stats(stats_args) => {
            let conn = get_database_connection(cli)?;

            // Get overall statistics
            let stats = get_submissions_stats(&conn)?;

            if stats.total == 0 {
                if !cli.quiet {
                    println!("{}", "No submissions found in history.".yellow());
                }
                return Ok(());
            }

            // Build filters for date range stats
            let mut filters = SubmissionFilters::new();
            let has_date_filter = stats_args.since.is_some() || stats_args.until.is_some();

            if let Some(since) = parse_date(stats_args.since.clone())? {
                filters = filters.after(since);
            }
            if let Some(until) = parse_date(stats_args.until.clone())? {
                filters = filters.before(until);
            }

            // Get filtered stats if date range specified
            let filtered_submissions = if has_date_filter {
                Some(list_submissions(&conn, &filters)?)
            } else {
                None
            };

            // Display based on format
            match stats_args.format {
                OutputFormat::Text => display_stats_text(&stats, filtered_submissions.as_deref())?,
                OutputFormat::Json => display_stats_json(&stats, filtered_submissions.as_deref())?,
                OutputFormat::Csv => {
                    return Err(IndexerError::UnsupportedOperation {
                        operation: "CSV format is not supported for stats command".to_string(),
                    });
                }
            }
        }
        HistoryCommand::Export(export_args) => {
            let conn = get_database_connection(cli)?;

            // Build filters for date range
            let mut filters = SubmissionFilters::new();
            if let Some(since) = parse_date(export_args.since)? {
                filters = filters.after(since);
            }
            if let Some(until) = parse_date(export_args.until)? {
                filters = filters.before(until);
            }

            // Query submissions
            let submissions = list_submissions(&conn, &filters)?;

            if submissions.is_empty() {
                if !cli.quiet {
                    println!("{}", "No submissions found to export.".yellow());
                }
                return Ok(());
            }

            if !cli.quiet {
                println!(
                    "{} {}",
                    "Exporting".green().bold(),
                    format!("{} submission(s)...", submissions.len()).cyan()
                );
            }

            // Export based on format
            match export_args.format {
                ExportFormat::Csv => export_to_csv(&submissions, &export_args.output)?,
                ExportFormat::Json => export_to_json(&submissions, &export_args.output)?,
            }

            if !cli.quiet {
                println!(
                    "{} Exported to: {}",
                    "✓".green().bold(),
                    export_args.output.display().to_string().cyan()
                );
            }
        }
        HistoryCommand::Clean(clean_args) => {
            let conn = get_database_connection(cli)?;

            // Determine cutoff date
            let cutoff = if let Some(days) = clean_args.older_than {
                Utc::now() - Duration::days(days as i64)
            } else if clean_args.all {
                // Set cutoff far in the future to delete everything
                Utc::now() + Duration::days(365 * 10)
            } else {
                return Err(IndexerError::MissingRequiredField {
                    field: "Must specify either --older-than or --all".to_string(),
                });
            };

            // Count records to be deleted
            let filters = if clean_args.all {
                SubmissionFilters::new()
            } else {
                SubmissionFilters::new().before(cutoff)
            };

            let count = count_submissions(&conn, &filters)?;

            if count == 0 {
                if !cli.quiet {
                    println!("{}", "No records to delete.".yellow());
                }
                return Ok(());
            }

            // Confirm before delete (unless --yes)
            if !clean_args.yes {
                let message = if clean_args.all {
                    format!("This will delete ALL {} record(s).", count)
                } else {
                    format!(
                        "This will delete {} record(s) older than {} days.",
                        count,
                        clean_args.older_than.unwrap()
                    )
                };

                println!("{}", message.yellow().bold());
                print!("{}", "Continue? (y/N): ".cyan());
                io::stdout().flush()?;

                let mut response = String::new();
                io::stdin().read_line(&mut response)?;
                let response = response.trim().to_lowercase();

                if response != "y" && response != "yes" {
                    if !cli.quiet {
                        println!("{}", "Cancelled.".yellow());
                    }
                    return Ok(());
                }
            }

            // Delete records
            let deleted = if clean_args.all {
                // Delete all by using a far future date
                delete_old_submissions(&conn, -365 * 10)?
            } else {
                delete_old_submissions(&conn, clean_args.older_than.unwrap() as i64)?
            };

            if !cli.quiet {
                println!(
                    "{} Deleted {} record(s)",
                    "✓".green().bold(),
                    deleted.to_string().cyan()
                );
            }
        }
    }
    Ok(())
}

/// Get database connection from config
fn get_database_connection(_cli: &Cli) -> Result<rusqlite::Connection> {
    // Load config to get database path
    let config = load_config()?;

    if !config.history.enabled {
        return Err(IndexerError::ConfigValidationError {
            message: "History tracking is disabled in configuration".to_string(),
        });
    }

    // Expand tilde in database path
    let db_path_str = &config.history.database_path;
    let db_path = if db_path_str.starts_with("~/") {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(db_path_str.replacen("~", &home, 1))
    } else {
        PathBuf::from(db_path_str)
    };

    // Initialize database
    init_database(&db_path)
}

/// Parse API filter from string
fn parse_api_filter(api: &str) -> Result<ApiType> {
    ApiType::from_str(api).map_err(|e| IndexerError::InvalidApiKey { message: e })
}

/// Parse status filter from string
fn parse_status_filter(status: &str) -> Result<SubmissionStatus> {
    SubmissionStatus::from_str(status).map_err(|e| IndexerError::ConfigInvalidValue {
        field: "status".to_string(),
        message: e,
    })
}

/// Parse date from string in YYYY-MM-DD format
fn parse_date(date_str: Option<String>) -> Result<Option<DateTime<Utc>>> {
    match date_str {
        Some(s) => {
            let date = NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(|_e| {
                IndexerError::InvalidDateFormat {
                    value: s,
                    expected: "YYYY-MM-DD".to_string(),
                }
            })?;
            Ok(Some(
                date.and_hms_opt(0, 0, 0)
                    .ok_or_else(|| IndexerError::InternalError {
                        message: "Invalid time components".to_string(),
                    })?
                    .and_utc(),
            ))
        }
        None => Ok(None),
    }
}

/// Format date for display
fn format_date(dt: DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Truncate string to max length
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len - 3])
    } else {
        s.to_string()
    }
}

/// Display submissions as a formatted table
fn display_as_table(
    submissions: &[crate::database::models::SubmissionRecord],
    quiet: bool,
) -> Result<()> {
    if !quiet {
        println!(
            "{:<6} {:<45} {:<10} {:<12} {:<8} {:<20}",
            "ID".bold(),
            "URL".bold(),
            "API".bold(),
            "Action".bold(),
            "Status".bold(),
            "Submitted".bold()
        );
        println!("{}", "─".repeat(110).dimmed());
    }

    for sub in submissions {
        let id = sub.id.map(|i| i.to_string()).unwrap_or_default();
        let url = truncate(&sub.url, 45);
        let api = sub.api.to_string();
        let action = sub.action.to_string();

        let status = match sub.status {
            SubmissionStatus::Success => sub.status.to_string().green(),
            SubmissionStatus::Failed => sub.status.to_string().red(),
        };

        let submitted = format_date(sub.submitted_at);

        if quiet {
            println!(
                "{:<6} {:<45} {:<10} {:<12} {:<8} {:<20}",
                id, url, api, action, status, submitted
            );
        } else {
            println!(
                "{:<6} {:<45} {:<10} {:<12} {} {:<20}",
                id.dimmed(),
                url.cyan(),
                api.yellow(),
                action.blue(),
                status,
                submitted.dimmed()
            );
        }
    }

    if !quiet {
        println!();
        println!(
            "{}",
            format!("Total: {} record(s)", submissions.len())
                .dimmed()
                .italic()
        );
    }

    Ok(())
}

/// Display submissions as JSON
fn display_as_json(submissions: &[crate::database::models::SubmissionRecord]) -> Result<()> {
    let json = serde_json::to_string_pretty(submissions).map_err(|e| {
        IndexerError::JsonSerializationError {
            message: format!("Failed to serialize to JSON: {}", e),
        }
    })?;
    println!("{}", json);
    Ok(())
}

/// Display submissions as CSV
fn display_as_csv(submissions: &[crate::database::models::SubmissionRecord]) -> Result<()> {
    let mut wtr = csv::Writer::from_writer(io::stdout());

    // Write header
    wtr.write_record(&[
        "id",
        "url",
        "api",
        "action",
        "status",
        "response_code",
        "response_message",
        "submitted_at",
    ])
    .map_err(|e| IndexerError::InternalError {
        message: format!("Failed to write CSV header: {}", e),
    })?;

    // Write records
    for sub in submissions {
        wtr.write_record(&[
            sub.id.map(|i| i.to_string()).unwrap_or_default(),
            sub.url.clone(),
            sub.api.to_string(),
            sub.action.to_string(),
            sub.status.to_string(),
            sub.response_code
                .map(|c| c.to_string())
                .unwrap_or_default(),
            sub.response_message.clone().unwrap_or_default(),
            sub.submitted_at.to_rfc3339(),
        ])
        .map_err(|e| IndexerError::InternalError {
            message: format!("Failed to write CSV record: {}", e),
        })?;
    }

    wtr.flush().map_err(|e| IndexerError::InternalError {
        message: format!("Failed to flush CSV output: {}", e),
    })?;
    Ok(())
}

/// Display statistics in text format
fn display_stats_text(
    stats: &crate::database::queries::SubmissionStats,
    filtered: Option<&[crate::database::models::SubmissionRecord]>,
) -> Result<()> {
    println!("{}", "Submission Statistics".cyan().bold());
    println!("{}", "═".repeat(60).dimmed());
    println!();

    if let Some(filtered_subs) = filtered {
        // Display filtered statistics
        let total = filtered_subs.len();
        let success = filtered_subs
            .iter()
            .filter(|s| s.status == SubmissionStatus::Success)
            .count();
        let failed = filtered_subs
            .iter()
            .filter(|s| s.status == SubmissionStatus::Failed)
            .count();
        let google = filtered_subs
            .iter()
            .filter(|s| s.api == ApiType::Google)
            .count();
        let indexnow = filtered_subs
            .iter()
            .filter(|s| s.api == ApiType::IndexNow)
            .count();

        println!(
            "{:<25} {}",
            "Total submissions:".bold(),
            total.to_string().cyan()
        );
        println!(
            "{:<25} {} ({}%)",
            "Successful:".bold(),
            success.to_string().green(),
            if total > 0 {
                format!("{:.1}", (success as f64 / total as f64) * 100.0)
            } else {
                "0.0".to_string()
            }
        );
        println!(
            "{:<25} {} ({}%)",
            "Failed:".bold(),
            failed.to_string().red(),
            if total > 0 {
                format!("{:.1}", (failed as f64 / total as f64) * 100.0)
            } else {
                "0.0".to_string()
            }
        );
        println!();
        println!("{}", "By API:".bold());
        println!("  {:<20} {}", "Google:", google.to_string().yellow());
        println!("  {:<20} {}", "IndexNow:", indexnow.to_string().yellow());
    } else {
        // Display overall statistics
        println!(
            "{:<25} {}",
            "Total submissions:".bold(),
            stats.total.to_string().cyan()
        );
        println!(
            "{:<25} {} ({}%)",
            "Successful:".bold(),
            stats.success.to_string().green(),
            if stats.total > 0 {
                format!("{:.1}", (stats.success as f64 / stats.total as f64) * 100.0)
            } else {
                "0.0".to_string()
            }
        );
        println!(
            "{:<25} {} ({}%)",
            "Failed:".bold(),
            stats.failed.to_string().red(),
            if stats.total > 0 {
                format!("{:.1}", (stats.failed as f64 / stats.total as f64) * 100.0)
            } else {
                "0.0".to_string()
            }
        );
        println!();
        println!("{}", "By API:".bold());
        println!("  {:<20} {}", "Google:", stats.google.to_string().yellow());
        println!(
            "  {:<20} {}",
            "IndexNow:",
            stats.indexnow.to_string().yellow()
        );
        println!();
        if let Some(last) = stats.last_submission {
            println!(
                "{:<25} {}",
                "Last submission:".bold(),
                format_date(last).dimmed()
            );
        }
    }

    println!();
    Ok(())
}

/// Display statistics in JSON format
fn display_stats_json(
    stats: &crate::database::queries::SubmissionStats,
    filtered: Option<&[crate::database::models::SubmissionRecord]>,
) -> Result<()> {
    let stats_json = if let Some(filtered_subs) = filtered {
        let total = filtered_subs.len();
        let success = filtered_subs
            .iter()
            .filter(|s| s.status == SubmissionStatus::Success)
            .count();
        let failed = filtered_subs
            .iter()
            .filter(|s| s.status == SubmissionStatus::Failed)
            .count();
        let google = filtered_subs
            .iter()
            .filter(|s| s.api == ApiType::Google)
            .count();
        let indexnow = filtered_subs
            .iter()
            .filter(|s| s.api == ApiType::IndexNow)
            .count();

        serde_json::json!({
            "total": total,
            "success": success,
            "failed": failed,
            "success_rate": if total > 0 { (success as f64 / total as f64) * 100.0 } else { 0.0 },
            "by_api": {
                "google": google,
                "indexnow": indexnow
            }
        })
    } else {
        serde_json::json!({
            "total": stats.total,
            "success": stats.success,
            "failed": stats.failed,
            "success_rate": if stats.total > 0 { (stats.success as f64 / stats.total as f64) * 100.0 } else { 0.0 },
            "by_api": {
                "google": stats.google,
                "indexnow": stats.indexnow
            },
            "last_submission": stats.last_submission.map(|dt| dt.to_rfc3339())
        })
    };

    println!("{}", serde_json::to_string_pretty(&stats_json).unwrap());
    Ok(())
}

/// Export submissions to CSV file
fn export_to_csv(
    submissions: &[crate::database::models::SubmissionRecord],
    output: &PathBuf,
) -> Result<()> {
    let file = File::create(output).map_err(|e| IndexerError::FileWriteError {
        path: output.clone(),
        message: e.to_string(),
    })?;

    let mut wtr = csv::Writer::from_writer(file);

    // Write header
    wtr.write_record(&[
        "id",
        "url",
        "api",
        "action",
        "status",
        "response_code",
        "response_message",
        "submitted_at",
    ])
    .map_err(|e| IndexerError::FileWriteError {
        path: output.clone(),
        message: format!("Failed to write CSV header: {}", e),
    })?;

    // Write records
    for sub in submissions {
        wtr.write_record(&[
            sub.id.map(|i| i.to_string()).unwrap_or_default(),
            sub.url.clone(),
            sub.api.to_string(),
            sub.action.to_string(),
            sub.status.to_string(),
            sub.response_code
                .map(|c| c.to_string())
                .unwrap_or_default(),
            sub.response_message.clone().unwrap_or_default(),
            sub.submitted_at.to_rfc3339(),
        ])
        .map_err(|e| IndexerError::FileWriteError {
            path: output.clone(),
            message: format!("Failed to write CSV record: {}", e),
        })?;
    }

    wtr.flush().map_err(|e| IndexerError::FileWriteError {
        path: output.clone(),
        message: format!("Failed to flush CSV: {}", e),
    })?;

    Ok(())
}

/// Export submissions to JSON file
fn export_to_json(
    submissions: &[crate::database::models::SubmissionRecord],
    output: &PathBuf,
) -> Result<()> {
    let file = File::create(output).map_err(|e| IndexerError::FileWriteError {
        path: output.clone(),
        message: e.to_string(),
    })?;

    serde_json::to_writer_pretty(file, submissions).map_err(|e| IndexerError::FileWriteError {
        path: output.clone(),
        message: format!("Failed to write JSON: {}", e),
    })?;

    Ok(())
}
