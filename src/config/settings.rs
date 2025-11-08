// Configuration settings structures

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Google Indexing API configuration
    #[serde(default)]
    pub google: Option<GoogleConfig>,

    /// IndexNow API configuration
    #[serde(default)]
    pub indexnow: Option<IndexNowConfig>,

    /// Sitemap configuration
    #[serde(default)]
    pub sitemap: Option<SitemapConfig>,

    /// History tracking configuration
    #[serde(default)]
    pub history: HistoryConfig,

    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,

    /// Retry configuration
    #[serde(default)]
    pub retry: RetryConfig,

    /// Output configuration
    #[serde(default)]
    pub output: OutputConfig,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            google: None,
            indexnow: None,
            sitemap: None,
            history: HistoryConfig::default(),
            logging: LoggingConfig::default(),
            retry: RetryConfig::default(),
            output: OutputConfig::default(),
        }
    }
}

/// Google Indexing API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleConfig {
    /// Whether Google Indexing API is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Path to the service account JSON file
    pub service_account_file: PathBuf,

    /// API quota settings
    #[serde(default)]
    pub quota: QuotaConfig,

    /// Batch size for batch requests
    #[serde(default = "default_google_batch_size")]
    pub batch_size: usize,
}

impl Default for GoogleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            service_account_file: PathBuf::new(),
            quota: QuotaConfig::default(),
            batch_size: default_google_batch_size(),
        }
    }
}

/// API quota configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaConfig {
    /// Daily quota limit for publish requests
    #[serde(default = "default_daily_limit")]
    pub daily_limit: u32,

    /// Rate limit (requests per minute)
    #[serde(default = "default_rate_limit")]
    pub rate_limit: u32,
}

impl Default for QuotaConfig {
    fn default() -> Self {
        Self {
            daily_limit: default_daily_limit(),
            rate_limit: default_rate_limit(),
        }
    }
}

/// IndexNow API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexNowConfig {
    /// Whether IndexNow API is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// API key (8-128 characters)
    pub api_key: String,

    /// Key file location URL
    pub key_location: String,

    /// List of endpoints to submit to
    #[serde(default = "default_indexnow_endpoints")]
    pub endpoints: Vec<String>,

    /// Batch size for batch requests
    #[serde(default = "default_indexnow_batch_size")]
    pub batch_size: usize,
}

impl Default for IndexNowConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            api_key: String::new(),
            key_location: String::new(),
            endpoints: default_indexnow_endpoints(),
            batch_size: default_indexnow_batch_size(),
        }
    }
}

/// Sitemap configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitemapConfig {
    /// Sitemap URL
    pub url: String,

    /// Whether to follow sitemap index files
    #[serde(default = "default_true")]
    pub follow_index: bool,

    /// URL filters
    #[serde(default)]
    pub filters: SitemapFilters,
}

impl Default for SitemapConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            follow_index: true,
            filters: SitemapFilters::default(),
        }
    }
}

/// Sitemap filter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitemapFilters {
    /// URL pattern (regex)
    #[serde(default = "default_url_pattern")]
    pub url_pattern: String,

    /// Only include URLs modified after this date (ISO 8601)
    pub lastmod_after: Option<String>,

    /// Minimum priority threshold
    #[serde(default = "default_priority_min")]
    pub priority_min: f32,
}

impl Default for SitemapFilters {
    fn default() -> Self {
        Self {
            url_pattern: default_url_pattern(),
            lastmod_after: None,
            priority_min: default_priority_min(),
        }
    }
}

/// History tracking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryConfig {
    /// Whether history tracking is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Database file path
    #[serde(default = "default_database_path")]
    pub database_path: String,

    /// Number of days to retain history
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            database_path: default_database_path(),
            retention_days: default_retention_days(),
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Log file path
    #[serde(default = "default_log_file")]
    pub file: String,

    /// Maximum log file size in MB
    #[serde(default = "default_max_size_mb")]
    pub max_size_mb: u32,

    /// Maximum number of backup log files
    #[serde(default = "default_max_backups")]
    pub max_backups: u32,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            file: default_log_file(),
            max_size_mb: default_max_size_mb(),
            max_backups: default_max_backups(),
        }
    }
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Whether retry is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Maximum number of retry attempts
    #[serde(default = "default_max_attempts")]
    pub max_attempts: u32,

    /// Exponential backoff factor
    #[serde(default = "default_backoff_factor")]
    pub backoff_factor: u32,

    /// Maximum wait time in seconds
    #[serde(default = "default_max_wait_seconds")]
    pub max_wait_seconds: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_attempts: default_max_attempts(),
            backoff_factor: default_backoff_factor(),
            max_wait_seconds: default_max_wait_seconds(),
        }
    }
}

/// Output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Output format (text, json, csv)
    #[serde(default = "default_output_format")]
    pub format: String,

    /// Whether to use colored output
    #[serde(default = "default_true")]
    pub color: bool,

    /// Whether to show verbose output
    #[serde(default = "default_false")]
    pub verbose: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: default_output_format(),
            color: true,
            verbose: false,
        }
    }
}

// Default value functions

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_daily_limit() -> u32 {
    200
}

fn default_rate_limit() -> u32 {
    380
}

fn default_google_batch_size() -> usize {
    100
}

fn default_indexnow_batch_size() -> usize {
    10000
}

fn default_indexnow_endpoints() -> Vec<String> {
    vec![
        "https://api.indexnow.org/indexnow".to_string(),
        "https://www.bing.com/indexnow".to_string(),
        "https://yandex.com/indexnow".to_string(),
    ]
}

fn default_url_pattern() -> String {
    ".*".to_string()
}

fn default_priority_min() -> f32 {
    0.0
}

fn default_database_path() -> String {
    "~/.indexer-cli/history.db".to_string()
}

fn default_retention_days() -> u32 {
    365
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_file() -> String {
    "~/.indexer-cli/indexer.log".to_string()
}

fn default_max_size_mb() -> u32 {
    10
}

fn default_max_backups() -> u32 {
    5
}

fn default_max_attempts() -> u32 {
    3
}

fn default_backoff_factor() -> u32 {
    2
}

fn default_max_wait_seconds() -> u64 {
    60
}

fn default_output_format() -> String {
    "text".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert!(settings.google.is_none());
        assert!(settings.indexnow.is_none());
        assert!(settings.sitemap.is_none());
        assert!(settings.history.enabled);
        assert_eq!(settings.logging.level, "info");
        assert!(settings.retry.enabled);
        assert_eq!(settings.output.format, "text");
    }

    #[test]
    fn test_google_config_defaults() {
        let config = GoogleConfig::default();
        assert!(config.enabled);
        assert_eq!(config.quota.daily_limit, 200);
        assert_eq!(config.quota.rate_limit, 380);
        assert_eq!(config.batch_size, 100);
    }

    #[test]
    fn test_indexnow_config_defaults() {
        let config = IndexNowConfig::default();
        assert!(config.enabled);
        assert_eq!(config.batch_size, 10000);
        assert_eq!(config.endpoints.len(), 3);
    }

    #[test]
    fn test_retry_config_defaults() {
        let config = RetryConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.backoff_factor, 2);
        assert_eq!(config.max_wait_seconds, 60);
    }
}
