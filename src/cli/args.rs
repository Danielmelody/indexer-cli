//! Command line arguments definition using clap derive API.
//!
//! This module defines the complete CLI structure for the indexer-cli tool,
//! including all commands, subcommands, and their associated arguments and options.

use clap::{Parser, Subcommand, Args, ValueEnum};
use std::path::PathBuf;

/// Indexer CLI - A tool for submitting URLs to Google Indexing API and IndexNow
#[derive(Parser, Debug, Clone)]
#[command(name = "indexer")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// The command to execute
    #[command(subcommand)]
    pub command: Commands,

    /// Configuration file path
    #[arg(short, long, global = true, env = "INDEXER_CONFIG")]
    pub config: Option<PathBuf>,

    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Quiet mode (suppress non-error output)
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,
}

/// All available commands
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Initialize configuration with interactive wizard
    Init(InitArgs),

    /// Manage configuration settings
    Config(ConfigArgs),

    /// Google Indexing API operations
    Google(GoogleArgs),

    /// IndexNow API operations
    IndexNow(IndexNowArgs),

    /// Submit URLs to search engines (unified command)
    Submit(SubmitArgs),

    /// Sitemap operations
    Sitemap(SitemapArgs),

    /// View and manage submission history
    History(HistoryArgs),

    /// Watch sitemap for changes and auto-submit
    Watch(WatchArgs),

    /// Validate configuration and setup
    Validate(ValidateArgs),
}

// ============================================================================
// Init Command
// ============================================================================

/// Initialize configuration with interactive wizard
#[derive(Args, Debug, Clone)]
pub struct InitArgs {
    /// Create global configuration instead of project-local
    #[arg(short, long)]
    pub global: bool,

    /// Overwrite existing configuration
    #[arg(short, long)]
    pub force: bool,

    /// Skip interactive prompts and use defaults
    #[arg(long)]
    pub non_interactive: bool,
}

// ============================================================================
// Config Command
// ============================================================================

/// Configuration management command
#[derive(Args, Debug, Clone)]
pub struct ConfigArgs {
    /// Configuration subcommand
    #[command(subcommand)]
    pub command: ConfigCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ConfigCommand {
    /// List all configuration settings
    List,

    /// Set a configuration value
    Set(ConfigSetArgs),

    /// Get a configuration value
    Get(ConfigGetArgs),

    /// Validate configuration
    Validate,

    /// Show configuration file path
    Path,
}

#[derive(Args, Debug, Clone)]
pub struct ConfigSetArgs {
    /// Configuration key (e.g., google.enabled, indexnow.api_key)
    pub key: String,

    /// Configuration value
    pub value: String,

    /// Set in global configuration
    #[arg(short, long)]
    pub global: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ConfigGetArgs {
    /// Configuration key to retrieve
    pub key: String,
}

// ============================================================================
// Google Command
// ============================================================================

/// Google Indexing API operations
#[derive(Args, Debug, Clone)]
pub struct GoogleArgs {
    /// Google API subcommand
    #[command(subcommand)]
    pub command: GoogleCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum GoogleCommand {
    /// Setup Google service account
    Setup(GoogleSetupArgs),

    /// Submit URLs to Google Indexing API
    Submit(GoogleSubmitArgs),

    /// Check indexing status of URLs
    Status(GoogleStatusArgs),

    /// Show API quota usage
    Quota,

    /// Verify Google API configuration
    Verify,
}

#[derive(Args, Debug, Clone)]
pub struct GoogleSetupArgs {
    /// Path to Google service account JSON file
    #[arg(short, long)]
    pub service_account: PathBuf,

    /// Save to global configuration
    #[arg(short, long)]
    pub global: bool,
}

#[derive(Args, Debug, Clone)]
pub struct GoogleSubmitArgs {
    /// URLs to submit (can specify multiple)
    #[arg(required_unless_present_any = ["file", "sitemap"])]
    pub urls: Vec<String>,

    /// Read URLs from file (one per line)
    #[arg(short, long, conflicts_with = "urls")]
    pub file: Option<PathBuf>,

    /// Extract URLs from sitemap
    #[arg(short, long, conflicts_with_all = ["urls", "file"])]
    pub sitemap: Option<String>,

    /// Action type
    #[arg(short = 't', long, default_value = "url-updated")]
    pub action: GoogleAction,

    /// URL filter pattern (regex)
    #[arg(long)]
    pub filter: Option<String>,

    /// Only submit URLs modified since this date (YYYY-MM-DD)
    #[arg(long)]
    pub since: Option<String>,

    /// Batch size for submission
    #[arg(short, long)]
    pub batch_size: Option<usize>,

    /// Dry run (don't actually submit)
    #[arg(short, long)]
    pub dry_run: bool,

    /// Skip history check (submit even if already submitted)
    #[arg(long)]
    pub skip_history: bool,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum GoogleAction {
    /// Notify Google that URL was updated or added
    #[value(name = "url-updated")]
    UrlUpdated,

    /// Notify Google that URL was deleted
    #[value(name = "url-deleted")]
    UrlDeleted,
}

#[derive(Args, Debug, Clone)]
pub struct GoogleStatusArgs {
    /// URLs to check status (can specify multiple)
    #[arg(required_unless_present = "file")]
    pub urls: Vec<String>,

    /// Read URLs from file (one per line)
    #[arg(short, long)]
    pub file: Option<PathBuf>,

    /// Output format
    #[arg(short, long, default_value = "text")]
    pub format: OutputFormat,
}

// ============================================================================
// IndexNow Command
// ============================================================================

/// IndexNow API operations
#[derive(Args, Debug, Clone)]
pub struct IndexNowArgs {
    /// IndexNow subcommand
    #[command(subcommand)]
    pub command: IndexNowCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum IndexNowCommand {
    /// Setup IndexNow API key
    Setup(IndexNowSetupArgs),

    /// Generate a new IndexNow API key
    GenerateKey(IndexNowGenerateKeyArgs),

    /// Submit URLs to IndexNow
    Submit(IndexNowSubmitArgs),

    /// Verify IndexNow configuration and key file
    Verify,
}

#[derive(Args, Debug, Clone)]
pub struct IndexNowSetupArgs {
    /// IndexNow API key
    #[arg(short, long)]
    pub key: String,

    /// Key file location URL
    #[arg(short, long)]
    pub key_location: Option<String>,

    /// Save to global configuration
    #[arg(short, long)]
    pub global: bool,
}

#[derive(Args, Debug, Clone)]
pub struct IndexNowGenerateKeyArgs {
    /// Key length (8-128 characters, default: 32)
    #[arg(short, long, default_value = "32")]
    pub length: usize,

    /// Output directory for key file
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Save key to configuration
    #[arg(short, long)]
    pub save: bool,

    /// Save to global configuration
    #[arg(short, long)]
    pub global: bool,
}

#[derive(Args, Debug, Clone)]
pub struct IndexNowSubmitArgs {
    /// URLs to submit (can specify multiple)
    #[arg(required_unless_present_any = ["file", "sitemap"])]
    pub urls: Vec<String>,

    /// Read URLs from file (one per line)
    #[arg(short, long, conflicts_with = "urls")]
    pub file: Option<PathBuf>,

    /// Extract URLs from sitemap
    #[arg(short, long, conflicts_with_all = ["urls", "file"])]
    pub sitemap: Option<String>,

    /// URL filter pattern (regex)
    #[arg(long)]
    pub filter: Option<String>,

    /// Only submit URLs modified since this date (YYYY-MM-DD)
    #[arg(long)]
    pub since: Option<String>,

    /// Batch size for submission
    #[arg(short, long)]
    pub batch_size: Option<usize>,

    /// Search engine endpoint
    #[arg(short, long, default_value = "all")]
    pub endpoint: IndexNowEndpoint,

    /// Dry run (don't actually submit)
    #[arg(short, long)]
    pub dry_run: bool,

    /// Skip history check (submit even if already submitted)
    #[arg(long)]
    pub skip_history: bool,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum IndexNowEndpoint {
    /// Submit to all configured endpoints
    All,
    /// Microsoft Bing
    Bing,
    /// Yandex
    Yandex,
    /// Seznam.cz
    Seznam,
    /// Naver
    Naver,
    /// IndexNow.org
    IndexNow,
}

// ============================================================================
// Submit Command (Unified)
// ============================================================================

/// Submit URLs to search engines (unified command)
#[derive(Args, Debug, Clone)]
pub struct SubmitArgs {
    /// URLs to submit (can specify multiple)
    #[arg(required_unless_present_any = ["file", "sitemap"])]
    pub urls: Vec<String>,

    /// Read URLs from file (one per line)
    #[arg(short, long, conflicts_with = "urls")]
    pub file: Option<PathBuf>,

    /// Extract URLs from sitemap
    #[arg(short, long, conflicts_with_all = ["urls", "file"])]
    pub sitemap: Option<String>,

    /// Which APIs to use
    #[arg(short, long, default_value = "all")]
    pub api: ApiTarget,

    /// URL filter pattern (regex)
    #[arg(long)]
    pub filter: Option<String>,

    /// Only submit URLs modified since this date (YYYY-MM-DD)
    #[arg(long)]
    pub since: Option<String>,

    /// Google action type (for Google API only)
    #[arg(long, default_value = "url-updated")]
    pub google_action: GoogleAction,

    /// Batch size for submission
    #[arg(short, long)]
    pub batch_size: Option<usize>,

    /// Dry run (don't actually submit)
    #[arg(short, long)]
    pub dry_run: bool,

    /// Skip history check (submit even if already submitted)
    #[arg(long)]
    pub skip_history: bool,

    /// Output format for results
    #[arg(short = 'o', long, default_value = "text")]
    pub format: OutputFormat,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ApiTarget {
    /// Use all enabled APIs
    All,
    /// Only Google Indexing API
    Google,
    /// Only IndexNow API
    IndexNow,
}

// ============================================================================
// Sitemap Command
// ============================================================================

/// Sitemap operations
#[derive(Args, Debug, Clone)]
pub struct SitemapArgs {
    /// Sitemap subcommand
    #[command(subcommand)]
    pub command: SitemapCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum SitemapCommand {
    /// Parse and display sitemap contents
    Parse(SitemapParseArgs),

    /// List all URLs from sitemap
    List(SitemapListArgs),

    /// Export URLs to file
    Export(SitemapExportArgs),

    /// Show sitemap statistics
    Stats(SitemapStatsArgs),

    /// Validate sitemap format
    Validate(SitemapValidateArgs),
}

#[derive(Args, Debug, Clone)]
pub struct SitemapParseArgs {
    /// Sitemap URL or file path
    pub sitemap: String,

    /// Follow sitemap index files
    #[arg(short, long)]
    pub follow_index: bool,

    /// Output format
    #[arg(short = 'o', long, default_value = "text")]
    pub format: OutputFormat,
}

#[derive(Args, Debug, Clone)]
pub struct SitemapListArgs {
    /// Sitemap URL or file path
    pub sitemap: String,

    /// Follow sitemap index files
    #[arg(short, long)]
    pub follow_index: bool,

    /// URL filter pattern (regex)
    #[arg(long)]
    pub filter: Option<String>,

    /// Only show URLs modified since this date (YYYY-MM-DD)
    #[arg(long)]
    pub since: Option<String>,

    /// Limit number of URLs to display
    #[arg(short, long)]
    pub limit: Option<usize>,
}

#[derive(Args, Debug, Clone)]
pub struct SitemapExportArgs {
    /// Sitemap URL or file path
    pub sitemap: String,

    /// Output file path
    #[arg(short, long)]
    pub output: PathBuf,

    /// Follow sitemap index files
    #[arg(short, long)]
    pub follow_index: bool,

    /// URL filter pattern (regex)
    #[arg(long)]
    pub filter: Option<String>,

    /// Only export URLs modified since this date (YYYY-MM-DD)
    #[arg(long)]
    pub since: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct SitemapStatsArgs {
    /// Sitemap URL or file path
    pub sitemap: String,

    /// Follow sitemap index files
    #[arg(short, long)]
    pub follow_index: bool,

    /// Output format
    #[arg(short = 'o', long, default_value = "text")]
    pub format: OutputFormat,
}

#[derive(Args, Debug, Clone)]
pub struct SitemapValidateArgs {
    /// Sitemap URL or file path
    pub sitemap: String,

    /// Follow sitemap index files
    #[arg(short, long)]
    pub follow_index: bool,
}

// ============================================================================
// History Command
// ============================================================================

/// History management command
#[derive(Args, Debug, Clone)]
pub struct HistoryArgs {
    /// History subcommand
    #[command(subcommand)]
    pub command: HistoryCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum HistoryCommand {
    /// List recent submissions
    List(HistoryListArgs),

    /// Search submission history
    Search(HistorySearchArgs),

    /// Show submission statistics
    Stats(HistoryStatsArgs),

    /// Export history to file
    Export(HistoryExportArgs),

    /// Clean old history records
    Clean(HistoryCleanArgs),
}

#[derive(Args, Debug, Clone)]
pub struct HistoryListArgs {
    /// Number of recent records to show
    #[arg(short, long, default_value = "20")]
    pub limit: usize,

    /// Output format
    #[arg(short = 'o', long, default_value = "text")]
    pub format: OutputFormat,
}

#[derive(Args, Debug, Clone)]
pub struct HistorySearchArgs {
    /// Filter by URL pattern
    #[arg(short, long)]
    pub url: Option<String>,

    /// Filter by API (google, indexnow)
    #[arg(short, long)]
    pub api: Option<String>,

    /// Filter by status (success, failed)
    #[arg(short, long)]
    pub status: Option<String>,

    /// Start date (YYYY-MM-DD)
    #[arg(long)]
    pub since: Option<String>,

    /// End date (YYYY-MM-DD)
    #[arg(long)]
    pub until: Option<String>,

    /// Limit number of results
    #[arg(short, long, default_value = "100")]
    pub limit: usize,

    /// Output format
    #[arg(short = 'o', long, default_value = "text")]
    pub format: OutputFormat,
}

#[derive(Args, Debug, Clone)]
pub struct HistoryStatsArgs {
    /// Start date for statistics (YYYY-MM-DD)
    #[arg(long)]
    pub since: Option<String>,

    /// End date for statistics (YYYY-MM-DD)
    #[arg(long)]
    pub until: Option<String>,

    /// Output format
    #[arg(short = 'o', long, default_value = "text")]
    pub format: OutputFormat,
}

#[derive(Args, Debug, Clone)]
pub struct HistoryExportArgs {
    /// Output file path
    #[arg(short, long)]
    pub output: PathBuf,

    /// Export format
    #[arg(short, long, default_value = "csv")]
    pub format: ExportFormat,

    /// Start date (YYYY-MM-DD)
    #[arg(long)]
    pub since: Option<String>,

    /// End date (YYYY-MM-DD)
    #[arg(long)]
    pub until: Option<String>,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ExportFormat {
    /// CSV format
    Csv,
    /// JSON format
    Json,
}

#[derive(Args, Debug, Clone)]
pub struct HistoryCleanArgs {
    /// Delete records older than N days
    #[arg(short, long, conflicts_with = "all")]
    pub older_than: Option<u32>,

    /// Delete all records
    #[arg(short, long)]
    pub all: bool,

    /// Don't ask for confirmation
    #[arg(short = 'y', long)]
    pub yes: bool,
}

// ============================================================================
// Watch Command
// ============================================================================

/// Watch sitemap for changes and auto-submit
#[derive(Args, Debug, Clone)]
pub struct WatchArgs {
    /// Sitemap URL to monitor
    #[arg(short, long)]
    pub sitemap: String,

    /// Check interval in seconds
    #[arg(short, long, default_value = "3600")]
    pub interval: u64,

    /// Which APIs to use for submission
    #[arg(short, long, default_value = "all")]
    pub api: ApiTarget,

    /// Run in daemon mode (background)
    #[arg(short, long)]
    pub daemon: bool,

    /// PID file path (for daemon mode)
    #[arg(long)]
    pub pid_file: Option<PathBuf>,
}

// ============================================================================
// Validate Command
// ============================================================================

/// Validate configuration and setup
#[derive(Args, Debug, Clone)]
pub struct ValidateArgs {
    /// What to validate (if not specified, validates everything)
    #[arg(value_enum)]
    pub target: Option<ValidateTarget>,

    /// Check IndexNow key file accessibility
    #[arg(long)]
    pub check_key_file: bool,

    /// Output format
    #[arg(short = 'o', long, default_value = "text")]
    pub format: OutputFormat,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ValidateTarget {
    /// Validate all configuration
    All,
    /// Validate only Google configuration
    Google,
    /// Validate only IndexNow configuration
    IndexNow,
}

// ============================================================================
// Common Types
// ============================================================================

/// Output format for various commands
#[derive(ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    /// Human-readable text format
    Text,
    /// JSON format
    Json,
    /// CSV format (where applicable)
    Csv,
}

impl OutputFormat {
    pub fn is_text(&self) -> bool {
        matches!(self, OutputFormat::Text)
    }

    pub fn is_json(&self) -> bool {
        matches!(self, OutputFormat::Json)
    }

    pub fn is_csv(&self) -> bool {
        matches!(self, OutputFormat::Csv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        use clap::CommandFactory;
        let _ = Cli::command();
    }
}
