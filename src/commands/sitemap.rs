//! Sitemap command - Sitemap parsing and operations.

use crate::cli::args::{
    Cli, OutputFormat, SitemapArgs, SitemapCommand, SitemapExportArgs, SitemapListArgs,
    SitemapParseArgs, SitemapStatsArgs, SitemapValidateArgs,
};
use crate::services::sitemap_parser::{SitemapFilters, SitemapParser, SitemapUrl};
use crate::types::{error::IndexerError, Result};
use crate::utils::validators::validate_date;
use chrono::{DateTime, Utc};
use colored::Colorize;
use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tracing::{debug, info};

pub async fn run(args: SitemapArgs, cli: &Cli) -> Result<()> {
    match args.command {
        SitemapCommand::Parse(parse_args) => handle_parse(parse_args, cli).await,
        SitemapCommand::List(list_args) => handle_list(list_args, cli).await,
        SitemapCommand::Export(export_args) => handle_export(export_args, cli).await,
        SitemapCommand::Stats(stats_args) => handle_stats(stats_args, cli).await,
        SitemapCommand::Validate(validate_args) => handle_validate(validate_args, cli).await,
    }
}

/// Handle the parse subcommand
async fn handle_parse(args: SitemapParseArgs, cli: &Cli) -> Result<()> {
    if !cli.quiet {
        println!("{}", "Parsing sitemap...".cyan().bold());
        println!("{}", "─".repeat(60).dimmed());
    }

    // Create parser
    let parser = SitemapParser::new()?;

    // Parse sitemap
    let result = parser
        .parse_sitemap(&args.sitemap, None)
        .await
        .map_err(|e| {
            if !cli.quiet {
                eprintln!("{} {}", "Error:".red().bold(), e);
            }
            e
        })?;

    if !cli.quiet {
        println!(
            "{} Found {} URLs in sitemap",
            "✓".green().bold(),
            result.urls.len()
        );
    }

    // Output based on format
    match args.format {
        OutputFormat::Json => {
            let output = serde_json::to_string_pretty(&SitemapParseOutput {
                sitemap: args.sitemap.clone(),
                total_urls: result.total_count,
                filtered_urls: result.filtered_count,
                urls: result
                    .urls
                    .iter()
                    .map(|u| UrlOutput {
                        loc: u.loc.clone(),
                        lastmod: u.lastmod.map(|dt| dt.to_rfc3339()),
                        changefreq: u.changefreq.clone(),
                        priority: u.priority,
                    })
                    .collect(),
            })
            .map_err(|e| IndexerError::JsonSerializationError {
                message: e.to_string(),
            })?;
            println!("{}", output);
        }
        OutputFormat::Text => {
            if !args.follow_index && !cli.quiet {
                println!();
                println!("{}", "Sitemap Structure:".cyan().bold());
                println!("{}", "─".repeat(60).dimmed());
            }

            for (i, url) in result.urls.iter().enumerate() {
                if !cli.quiet {
                    println!("{:4}. {}", i + 1, url.loc);
                    if let Some(lastmod) = &url.lastmod {
                        println!(
                            "      Last Modified: {}",
                            lastmod.format("%Y-%m-%d %H:%M:%S UTC")
                        );
                    }
                    if let Some(changefreq) = &url.changefreq {
                        println!("      Change Frequency: {}", changefreq);
                    }
                    if let Some(priority) = url.priority {
                        println!("      Priority: {:.1}", priority);
                    }
                } else {
                    println!("{}", url.loc);
                }
            }
        }
        OutputFormat::Csv => {
            println!("url,lastmod,changefreq,priority");
            for url in result.urls.iter() {
                println!(
                    "{},{},{},{}",
                    url.loc,
                    url.lastmod.map(|dt| dt.to_rfc3339()).unwrap_or_default(),
                    url.changefreq.as_deref().unwrap_or(""),
                    url.priority.map(|p| p.to_string()).unwrap_or_default()
                );
            }
        }
    }

    Ok(())
}

/// Handle the list subcommand
async fn handle_list(args: SitemapListArgs, cli: &Cli) -> Result<()> {
    if !cli.quiet {
        println!("{}", "Listing URLs from sitemap...".cyan().bold());
        println!("{}", "─".repeat(60).dimmed());
    }

    // Build filters
    let filters = build_filters(args.filter.as_deref(), args.since.as_deref())?;

    // Create parser
    let parser = SitemapParser::new()?;

    // Parse sitemap with filters
    let result = parser
        .parse_sitemap(&args.sitemap, filters.as_ref())
        .await?;

    if !cli.quiet {
        println!(
            "{} Found {} URLs (filtered from {} total)",
            "✓".green().bold(),
            result.filtered_count,
            result.total_count
        );
        println!();
    }

    // Apply limit
    let urls_to_show = if let Some(limit) = args.limit {
        result.urls.iter().take(limit).collect::<Vec<_>>()
    } else {
        result.urls.iter().collect::<Vec<_>>()
    };

    // Display URLs
    for url in urls_to_show {
        println!("{}", url.loc);
    }

    if let Some(limit) = args.limit {
        if result.urls.len() > limit && !cli.quiet {
            println!();
            println!(
                "{}",
                format!(
                    "(Showing first {} of {} URLs. Use --limit to show more)",
                    limit,
                    result.urls.len()
                )
                .dimmed()
            );
        }
    }

    Ok(())
}

/// Handle the export subcommand
async fn handle_export(args: SitemapExportArgs, cli: &Cli) -> Result<()> {
    if !cli.quiet {
        println!("{}", "Exporting sitemap URLs...".cyan().bold());
        println!("{}", "─".repeat(60).dimmed());
    }

    // Build filters
    let filters = build_filters(args.filter.as_deref(), args.since.as_deref())?;

    // Create parser
    let parser = SitemapParser::new()?;

    // Parse sitemap with filters
    let result = parser
        .parse_sitemap(&args.sitemap, filters.as_ref())
        .await?;

    if !cli.quiet {
        println!(
            "{} Found {} URLs to export",
            "✓".green().bold(),
            result.filtered_count
        );
    }

    // Determine format from file extension
    let export_format = determine_export_format(&args.output)?;

    // Export to file
    export_urls(&result.urls, &args.output, export_format, cli)?;

    if !cli.quiet {
        println!(
            "{} Exported {} URLs to {}",
            "✓".green().bold(),
            result.filtered_count,
            args.output.display()
        );
    }

    Ok(())
}

/// Handle the stats subcommand
async fn handle_stats(args: SitemapStatsArgs, cli: &Cli) -> Result<()> {
    if !cli.quiet {
        println!("{}", "Analyzing sitemap...".cyan().bold());
        println!("{}", "─".repeat(60).dimmed());
    }

    // Create parser
    let parser = SitemapParser::new()?;

    // Parse sitemap
    let result = parser.parse_sitemap(&args.sitemap, None).await?;

    // Calculate statistics
    let stats = calculate_statistics(&result.urls);

    // Output based on format
    match args.format {
        OutputFormat::Json => {
            let output = serde_json::to_string_pretty(&stats).map_err(|e| {
                IndexerError::JsonSerializationError {
                    message: e.to_string(),
                }
            })?;
            println!("{}", output);
        }
        OutputFormat::Text => {
            display_statistics(&stats, cli);
        }
        OutputFormat::Csv => {
            // CSV format for stats
            println!("metric,value");
            println!("total_urls,{}", stats.total_urls);
            println!("urls_with_lastmod,{}", stats.urls_with_lastmod);
            println!("urls_with_changefreq,{}", stats.urls_with_changefreq);
            println!("urls_with_priority,{}", stats.urls_with_priority);
            println!("average_priority,{:.2}", stats.average_priority);
            if let Some(oldest) = &stats.oldest_modification {
                println!("oldest_modification,{}", oldest);
            }
            if let Some(newest) = &stats.newest_modification {
                println!("newest_modification,{}", newest);
            }
        }
    }

    Ok(())
}

/// Handle the validate subcommand
async fn handle_validate(args: SitemapValidateArgs, cli: &Cli) -> Result<()> {
    if !cli.quiet {
        println!("{}", "Validating sitemap...".cyan().bold());
        println!("{}", "─".repeat(60).dimmed());
    }

    // Create parser
    let parser = SitemapParser::new()?;

    // Parse sitemap (this validates it in the process)
    let result = match parser.parse_sitemap(&args.sitemap, None).await {
        Ok(r) => r,
        Err(e) => {
            if !cli.quiet {
                println!("{} Sitemap validation failed!", "✗".red().bold());
                println!();
                println!("{} {}", "Error:".red().bold(), e);
            }
            return Err(e);
        }
    };

    // Perform additional validation checks
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    // Check URL count
    if result.total_count > 50_000 {
        warnings.push(format!(
            "Sitemap contains {} URLs, which exceeds the recommended limit of 50,000",
            result.total_count
        ));
    }

    // Check for duplicate URLs
    let mut url_set = std::collections::HashSet::new();
    for url in &result.urls {
        if !url_set.insert(&url.loc) {
            warnings.push(format!("Duplicate URL found: {}", url.loc));
        }
    }

    // Validate URLs
    for url in &result.urls {
        if let Err(e) = crate::utils::validators::validate_url(&url.loc) {
            errors.push(format!("Invalid URL '{}': {}", url.loc, e));
        }
    }

    // Display validation results
    if !cli.quiet {
        if errors.is_empty() && warnings.is_empty() {
            println!("{} Sitemap is valid!", "✓".green().bold());
            println!();
            println!("  Total URLs: {}", result.total_count);
            println!("  Unique URLs: {}", url_set.len());
            if result.urls.iter().any(|u| u.lastmod.is_some()) {
                println!(
                    "  URLs with lastmod: {}",
                    result.urls.iter().filter(|u| u.lastmod.is_some()).count()
                );
            }
        } else {
            if !errors.is_empty() {
                println!("{} Sitemap validation failed!", "✗".red().bold());
                println!();
                for error in &errors {
                    println!("  {} {}", "✗".red(), error);
                }
            } else {
                println!("{} Sitemap is valid with warnings", "⚠".yellow().bold());
            }

            if !warnings.is_empty() {
                println!();
                println!("{}", "Warnings:".yellow().bold());
                for warning in &warnings {
                    println!("  {} {}", "⚠".yellow(), warning);
                }
            }

            if !errors.is_empty() {
                return Err(IndexerError::SitemapParseError {
                    message: format!("Found {} validation errors", errors.len()),
                });
            }
        }
    }

    Ok(())
}

// Helper functions

/// Build filters from command line arguments
fn build_filters(
    filter_pattern: Option<&str>,
    since_date: Option<&str>,
) -> Result<Option<SitemapFilters>> {
    let mut has_filters = false;
    let mut filters = SitemapFilters::default();

    if let Some(pattern) = filter_pattern {
        debug!("Applying URL filter pattern: {}", pattern);
        let regex = Regex::new(pattern).map_err(|e| IndexerError::InvalidRegexPattern {
            pattern: pattern.to_string(),
            message: e.to_string(),
        })?;
        filters.url_pattern = Some(regex);
        has_filters = true;
    }

    if let Some(date_str) = since_date {
        debug!("Applying date filter: {}", date_str);
        let date = validate_date(date_str)?;
        let datetime = date
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| IndexerError::InvalidDateFormat {
                value: date_str.to_string(),
                expected: "YYYY-MM-DD".to_string(),
            })?
            .and_local_timezone(Utc)
            .single()
            .ok_or_else(|| IndexerError::InvalidDateFormat {
                value: date_str.to_string(),
                expected: "YYYY-MM-DD".to_string(),
            })?;
        filters.lastmod_after = Some(datetime);
        has_filters = true;
    }

    if has_filters {
        Ok(Some(filters))
    } else {
        Ok(None)
    }
}

/// Determine export format from file extension
fn determine_export_format(output_path: &Path) -> Result<ExportFormat> {
    let extension = output_path
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| IndexerError::InvalidFileFormat {
            path: output_path.to_path_buf(),
            expected: "csv or json".to_string(),
            actual: "unknown".to_string(),
        })?;

    match extension.to_lowercase().as_str() {
        "json" => Ok(ExportFormat::Json),
        "csv" => Ok(ExportFormat::Csv),
        _ => Err(IndexerError::InvalidFileFormat {
            path: output_path.to_path_buf(),
            expected: "csv or json".to_string(),
            actual: extension.to_string(),
        }),
    }
}

/// Export URLs to a file
fn export_urls(
    urls: &[SitemapUrl],
    output_path: &Path,
    format: ExportFormat,
    cli: &Cli,
) -> Result<()> {
    // Create parent directories if they don't exist
    if let Some(parent) = output_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| IndexerError::DirectoryCreationFailed {
                path: parent.to_path_buf(),
                message: e.to_string(),
            })?;
        }
    }

    let mut file = File::create(output_path).map_err(|e| IndexerError::FileWriteError {
        path: output_path.to_path_buf(),
        message: e.to_string(),
    })?;

    match format {
        ExportFormat::Json => {
            let json_urls: Vec<UrlOutput> = urls
                .iter()
                .map(|u| UrlOutput {
                    loc: u.loc.clone(),
                    lastmod: u.lastmod.map(|dt| dt.to_rfc3339()),
                    changefreq: u.changefreq.clone(),
                    priority: u.priority,
                })
                .collect();

            let json = serde_json::to_string_pretty(&json_urls).map_err(|e| {
                IndexerError::JsonSerializationError {
                    message: e.to_string(),
                }
            })?;

            file.write_all(json.as_bytes())
                .map_err(|e| IndexerError::FileWriteError {
                    path: output_path.to_path_buf(),
                    message: e.to_string(),
                })?;
        }
        ExportFormat::Csv => {
            // Write CSV header
            writeln!(file, "url,lastmod,changefreq,priority").map_err(|e| {
                IndexerError::FileWriteError {
                    path: output_path.to_path_buf(),
                    message: e.to_string(),
                }
            })?;

            // Write CSV rows
            for url in urls {
                writeln!(
                    file,
                    "{},{},{},{}",
                    url.loc,
                    url.lastmod.map(|dt| dt.to_rfc3339()).unwrap_or_default(),
                    url.changefreq.as_deref().unwrap_or(""),
                    url.priority.map(|p| p.to_string()).unwrap_or_default()
                )
                .map_err(|e| IndexerError::FileWriteError {
                    path: output_path.to_path_buf(),
                    message: e.to_string(),
                })?;
            }
        }
    }

    if !cli.quiet {
        info!("Exported {} URLs to {}", urls.len(), output_path.display());
    }

    Ok(())
}

/// Calculate statistics from URLs
fn calculate_statistics(urls: &[SitemapUrl]) -> SitemapStatistics {
    let total_urls = urls.len();
    let urls_with_lastmod = urls.iter().filter(|u| u.lastmod.is_some()).count();
    let urls_with_changefreq = urls.iter().filter(|u| u.changefreq.is_some()).count();
    let urls_with_priority = urls.iter().filter(|u| u.priority.is_some()).count();

    // Calculate average priority
    let priorities: Vec<f32> = urls.iter().filter_map(|u| u.priority).collect();
    let average_priority = if !priorities.is_empty() {
        priorities.iter().sum::<f32>() / priorities.len() as f32
    } else {
        0.0
    };

    // Find oldest and newest modification dates
    let dates: Vec<DateTime<Utc>> = urls.iter().filter_map(|u| u.lastmod).collect();
    let oldest_modification = dates.iter().min().map(|dt| dt.to_rfc3339());
    let newest_modification = dates.iter().max().map(|dt| dt.to_rfc3339());

    // Count changefreq values
    let mut changefreq_distribution: HashMap<String, usize> = HashMap::new();
    for url in urls {
        if let Some(changefreq) = &url.changefreq {
            *changefreq_distribution
                .entry(changefreq.clone())
                .or_insert(0) += 1;
        }
    }

    SitemapStatistics {
        total_urls,
        urls_with_lastmod,
        urls_with_changefreq,
        urls_with_priority,
        average_priority,
        oldest_modification,
        newest_modification,
        changefreq_distribution,
    }
}

/// Display statistics in text format
fn display_statistics(stats: &SitemapStatistics, cli: &Cli) {
    if cli.quiet {
        return;
    }

    println!("{}", "Sitemap Statistics".cyan().bold());
    println!("{}", "━".repeat(60).dimmed());
    println!();

    println!("{}", "General:".cyan().bold());
    println!("  Total URLs: {}", stats.total_urls);
    println!("  URLs with lastmod: {}", stats.urls_with_lastmod);
    println!("  URLs with changefreq: {}", stats.urls_with_changefreq);
    println!("  URLs with priority: {}", stats.urls_with_priority);
    println!();

    if stats.urls_with_priority > 0 {
        println!("{}", "Priority:".cyan().bold());
        println!("  Average priority: {:.2}", stats.average_priority);
        println!();
    }

    if stats.oldest_modification.is_some() || stats.newest_modification.is_some() {
        println!("{}", "Modification Dates:".cyan().bold());
        if let Some(oldest) = &stats.oldest_modification {
            println!("  Oldest: {}", oldest);
        }
        if let Some(newest) = &stats.newest_modification {
            println!("  Newest: {}", newest);
        }
        println!();
    }

    if !stats.changefreq_distribution.is_empty() {
        println!("{}", "Change Frequency Distribution:".cyan().bold());
        let mut sorted: Vec<_> = stats.changefreq_distribution.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        for (freq, count) in sorted {
            println!("  {}: {}", freq, count);
        }
    }
}

// Data structures

#[derive(Debug)]
enum ExportFormat {
    Json,
    Csv,
}

#[derive(Debug, Serialize)]
struct SitemapParseOutput {
    sitemap: String,
    total_urls: usize,
    filtered_urls: usize,
    urls: Vec<UrlOutput>,
}

#[derive(Debug, Serialize)]
struct UrlOutput {
    loc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    lastmod: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    changefreq: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<f32>,
}

#[derive(Debug, Serialize)]
struct SitemapStatistics {
    total_urls: usize,
    urls_with_lastmod: usize,
    urls_with_changefreq: usize,
    urls_with_priority: usize,
    average_priority: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    oldest_modification: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    newest_modification: Option<String>,
    changefreq_distribution: HashMap<String, usize>,
}
