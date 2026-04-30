//! Global constants for the indexer-cli application
//!
//! This module defines all constants used throughout the application, including:
//! - API endpoints
//! - Quotas and limits
//! - Default configuration paths
//! - Timeout and retry settings
//! - Other application-wide constants

use std::time::Duration;

// =============================================================================
// Application Information
// =============================================================================

/// Application version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name
pub const APP_NAME: &str = env!("CARGO_PKG_NAME");

/// User agent string used for HTTP requests
pub const USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " (Rust)"
);

// =============================================================================
// API Endpoints
// =============================================================================

/// Google Indexing API endpoint
///
/// This is the base URL for the Google Indexing API v3.
/// Used for submitting URL updates and retrieving metadata.
pub const GOOGLE_INDEXING_API_ENDPOINT: &str = "https://indexing.googleapis.com/v3";

/// Google Indexing API batch endpoint
///
/// Used for batch requests to the Google Indexing API.
pub const GOOGLE_INDEXING_API_BATCH_ENDPOINT: &str = "https://indexing.googleapis.com/batch/v3";

/// IndexNow API endpoint (api.indexnow.org)
///
/// Primary IndexNow endpoint that distributes to multiple search engines.
pub const INDEXNOW_API_ENDPOINT: &str = "https://api.indexnow.org";

/// IndexNow Bing endpoint
///
/// Direct endpoint for submitting URLs to Bing via IndexNow protocol.
pub const INDEXNOW_BING_ENDPOINT: &str = "https://www.bing.com/indexnow";

/// IndexNow Yandex endpoint
///
/// Direct endpoint for submitting URLs to Yandex via IndexNow protocol.
pub const INDEXNOW_YANDEX_ENDPOINT: &str = "https://yandex.com/indexnow";

/// IndexNow DuckDuckGo endpoint
///
/// Direct endpoint for submitting URLs to DuckDuckGo via IndexNow protocol.
pub const INDEXNOW_DUCKDUCKGO_ENDPOINT: &str = "https://www.duckduckgo.com/indexnow";

/// List of all supported IndexNow endpoints
pub const INDEXNOW_ENDPOINTS: &[&str] = &[
    INDEXNOW_API_ENDPOINT,
    INDEXNOW_BING_ENDPOINT,
    INDEXNOW_YANDEX_ENDPOINT,
    INDEXNOW_DUCKDUCKGO_ENDPOINT,
];

// =============================================================================
// Google API Quotas and Limits
// =============================================================================

/// Maximum number of publish/update requests per day for Google Indexing API
///
/// Google allows up to 200 URL_UPDATED or URL_DELETED notifications per day.
pub const GOOGLE_QUOTA_PUBLISH_PER_DAY: u32 = 200;

/// Maximum number of getMetadata requests per minute
///
/// Rate limit for retrieving URL metadata from Google Indexing API.
pub const GOOGLE_QUOTA_GET_METADATA_PER_MINUTE: u32 = 180;

/// Maximum total requests per minute across all API methods
///
/// Combined rate limit for all Google Indexing API requests.
pub const GOOGLE_QUOTA_TOTAL_PER_MINUTE: u32 = 380;

/// Default batch size for Google API requests
///
/// Number of URLs to process in a single batch operation.
pub const GOOGLE_DEFAULT_BATCH_SIZE: usize = 100;

// =============================================================================
// IndexNow Limits
// =============================================================================

/// Maximum number of URLs that can be submitted in a single IndexNow request
///
/// IndexNow protocol allows up to 10,000 URLs per request.
pub const INDEXNOW_MAX_URLS_PER_REQUEST: usize = 10_000;

/// Default batch size for IndexNow submissions
///
/// Conservative batch size for optimal performance.
pub const INDEXNOW_DEFAULT_BATCH_SIZE: usize = 1_000;

/// Maximum size of IndexNow request body in bytes (10MB)
pub const INDEXNOW_MAX_REQUEST_SIZE_BYTES: usize = 10_485_760;

// =============================================================================
// Sitemap Limits
// =============================================================================

/// Maximum number of URLs allowed in a single sitemap file
///
/// Standard sitemap limit as per sitemaps.org protocol.
pub const SITEMAP_MAX_URLS: usize = 50_000;

/// Maximum sitemap file size in bytes (50MB)
///
/// Sitemap files larger than this should be split into multiple files.
pub const SITEMAP_MAX_SIZE_BYTES: usize = 52_428_800;

/// Maximum number of sitemap index files that can reference other sitemaps
pub const SITEMAP_INDEX_MAX_SITEMAPS: usize = 50_000;

// =============================================================================
// Default File Paths
// =============================================================================

/// Default directory name for application configuration
pub const DEFAULT_CONFIG_DIR_NAME: &str = ".indexer-cli";

/// Default configuration file name
pub const DEFAULT_CONFIG_FILE_NAME: &str = "config.yaml";

/// Default database file name
pub const DEFAULT_DATABASE_FILE_NAME: &str = "history.db";

/// Default log file name
pub const DEFAULT_LOG_FILE_NAME: &str = "indexer.log";

/// Default Google service account credentials file name
pub const DEFAULT_GOOGLE_CREDENTIALS_FILE_NAME: &str = "service-account.json";

// =============================================================================
// HTTP Timeout Settings
// =============================================================================

/// Default HTTP request timeout
///
/// Maximum time to wait for a complete HTTP request/response cycle.
pub const DEFAULT_HTTP_TIMEOUT_SECS: u64 = 30;

/// Default HTTP request timeout as Duration
pub const DEFAULT_HTTP_TIMEOUT: Duration = Duration::from_secs(DEFAULT_HTTP_TIMEOUT_SECS);

/// HTTP connection timeout
///
/// Maximum time to wait for establishing a connection.
pub const HTTP_CONNECT_TIMEOUT_SECS: u64 = 10;

/// HTTP connection timeout as Duration
pub const HTTP_CONNECT_TIMEOUT: Duration = Duration::from_secs(HTTP_CONNECT_TIMEOUT_SECS);

/// HTTP read timeout for long-running operations
pub const HTTP_READ_TIMEOUT_SECS: u64 = 60;

/// HTTP read timeout as Duration
pub const HTTP_READ_TIMEOUT: Duration = Duration::from_secs(HTTP_READ_TIMEOUT_SECS);

// =============================================================================
// Retry Configuration
// =============================================================================

/// Default maximum number of retry attempts for failed requests
///
/// Includes the initial request, so 3 means: 1 initial + 2 retries.
pub const DEFAULT_MAX_RETRIES: u32 = 3;

/// Backoff factor for exponential retry delays
///
/// Delay between retries is calculated as: `base_delay * backoff_factor^attempt`
pub const DEFAULT_BACKOFF_FACTOR: f64 = 2.0;

/// Initial delay in milliseconds before first retry
pub const DEFAULT_INITIAL_RETRY_DELAY_MS: u64 = 1000;

/// Initial retry delay as Duration
pub const DEFAULT_INITIAL_RETRY_DELAY: Duration =
    Duration::from_millis(DEFAULT_INITIAL_RETRY_DELAY_MS);

/// Maximum retry delay in seconds
///
/// Caps the exponential backoff to prevent excessively long waits.
pub const MAX_RETRY_DELAY_SECS: u64 = 60;

/// Maximum retry delay as Duration
pub const MAX_RETRY_DELAY: Duration = Duration::from_secs(MAX_RETRY_DELAY_SECS);

// =============================================================================
// IndexNow Key Configuration
// =============================================================================

/// Minimum length for IndexNow API key
///
/// Keys must be at least 8 characters according to IndexNow specification.
pub const INDEXNOW_KEY_MIN_LENGTH: usize = 8;

/// Maximum length for IndexNow API key
///
/// Keys can be up to 128 characters according to IndexNow specification.
pub const INDEXNOW_KEY_MAX_LENGTH: usize = 128;

/// Recommended length for IndexNow API key
///
/// 32 characters provides a good balance of security and usability.
pub const INDEXNOW_KEY_RECOMMENDED_LENGTH: usize = 32;

/// Valid characters for IndexNow API key
///
/// Keys should contain only alphanumeric characters and hyphens.
pub const INDEXNOW_KEY_VALID_CHARS: &str =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-";

// =============================================================================
// Rate Limiting
// =============================================================================

/// Default delay between requests in milliseconds
///
/// Used to avoid overwhelming APIs with rapid requests.
pub const DEFAULT_REQUEST_DELAY_MS: u64 = 100;

/// Default request delay as Duration
pub const DEFAULT_REQUEST_DELAY: Duration = Duration::from_millis(DEFAULT_REQUEST_DELAY_MS);

/// Minimum delay between requests in milliseconds
pub const MIN_REQUEST_DELAY_MS: u64 = 10;

/// Minimum request delay as Duration
pub const MIN_REQUEST_DELAY: Duration = Duration::from_millis(MIN_REQUEST_DELAY_MS);

/// Default concurrent request limit
///
/// Maximum number of simultaneous HTTP requests.
pub const DEFAULT_CONCURRENT_REQUESTS: usize = 10;

// =============================================================================
// Database Configuration
// =============================================================================

/// SQLite database connection pool size
pub const DATABASE_POOL_SIZE: u32 = 5;

/// Database busy timeout in milliseconds
///
/// How long to wait if database is locked before failing.
pub const DATABASE_BUSY_TIMEOUT_MS: u32 = 5000;

/// Maximum number of records to return in a single query
pub const DATABASE_MAX_QUERY_LIMIT: usize = 1000;

// =============================================================================
// Logging Configuration
// =============================================================================

/// Default log level
pub const DEFAULT_LOG_LEVEL: &str = "info";

/// Maximum log file size in bytes before rotation (10MB)
pub const LOG_MAX_FILE_SIZE_BYTES: u64 = 10_485_760;

/// Maximum number of rotated log files to keep
pub const LOG_MAX_BACKUP_FILES: usize = 5;

// =============================================================================
// URL Validation
// =============================================================================

/// Maximum URL length
///
/// Most browsers and servers support URLs up to 2048 characters.
pub const MAX_URL_LENGTH: usize = 2048;

/// Supported URL schemes
pub const SUPPORTED_URL_SCHEMES: &[&str] = &["http", "https"];

// =============================================================================
// Progress Display
// =============================================================================

/// Progress bar update interval in milliseconds
pub const PROGRESS_UPDATE_INTERVAL_MS: u64 = 100;

/// Progress bar template for displaying submission progress
pub const PROGRESS_BAR_TEMPLATE: &str =
    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})";

// =============================================================================
// Cache Configuration
// =============================================================================

/// Default cache TTL (time-to-live) in seconds
///
/// How long to keep cached data before considering it stale.
pub const DEFAULT_CACHE_TTL_SECS: u64 = 3600; // 1 hour

/// Default cache TTL as Duration
pub const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(DEFAULT_CACHE_TTL_SECS);

/// Maximum cache size in number of entries
pub const MAX_CACHE_ENTRIES: usize = 10_000;

// =============================================================================
// Error Messages
// =============================================================================

/// Error message for missing configuration
pub const ERROR_CONFIG_NOT_FOUND: &str =
    "Configuration file not found. Run 'indexer-cli init' to create one.";

/// Error message for invalid configuration
pub const ERROR_CONFIG_INVALID: &str = "Configuration file is invalid. Please check the format.";

/// Error message for missing Google credentials
pub const ERROR_GOOGLE_CREDENTIALS_NOT_FOUND: &str =
    "Google service account credentials not found. Please configure your credentials.";

/// Error message for missing IndexNow key
pub const ERROR_INDEXNOW_KEY_NOT_FOUND: &str =
    "IndexNow API key not found. Please configure your API key.";

// =============================================================================
// Success Messages
// =============================================================================

/// Success message for initialization
pub const SUCCESS_INIT_COMPLETE: &str = "Initialization complete! Configuration file created.";

/// Success message for successful submission
pub const SUCCESS_SUBMISSION_COMPLETE: &str = "URLs submitted successfully!";

// =============================================================================
// Helper Functions
// =============================================================================

/// Returns the default configuration directory path
///
/// Expands `~` to the user's home directory.
pub fn default_config_dir() -> std::path::PathBuf {
    let home = crate::utils::file::user_home_dir().expect("Unable to determine home directory");
    home.join(DEFAULT_CONFIG_DIR_NAME)
}

/// Returns the full path to the default configuration file
pub fn default_config_file_path() -> std::path::PathBuf {
    default_config_dir().join(DEFAULT_CONFIG_FILE_NAME)
}

/// Returns the full path to the default database file
pub fn default_database_file_path() -> std::path::PathBuf {
    default_config_dir().join(DEFAULT_DATABASE_FILE_NAME)
}

/// Returns the full path to the default log file
pub fn default_log_file_path() -> std::path::PathBuf {
    default_config_dir().join(DEFAULT_LOG_FILE_NAME)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_not_empty() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_user_agent_format() {
        assert!(USER_AGENT.contains(APP_NAME));
        assert!(USER_AGENT.contains(VERSION));
    }

    #[test]
    fn test_google_quota_consistency() {
        assert!(GOOGLE_QUOTA_PUBLISH_PER_DAY > 0);
        assert!(GOOGLE_QUOTA_GET_METADATA_PER_MINUTE > 0);
        assert!(GOOGLE_QUOTA_TOTAL_PER_MINUTE > GOOGLE_QUOTA_GET_METADATA_PER_MINUTE);
    }

    #[test]
    fn test_indexnow_limits() {
        assert!(INDEXNOW_MAX_URLS_PER_REQUEST == 10_000);
        assert!(INDEXNOW_DEFAULT_BATCH_SIZE <= INDEXNOW_MAX_URLS_PER_REQUEST);
    }

    #[test]
    fn test_sitemap_limits() {
        assert!(SITEMAP_MAX_URLS == 50_000);
        assert!(SITEMAP_MAX_SIZE_BYTES == 52_428_800); // 50MB
    }

    #[test]
    fn test_indexnow_key_length_constraints() {
        assert!(INDEXNOW_KEY_MIN_LENGTH == 8);
        assert!(INDEXNOW_KEY_MAX_LENGTH == 128);
        assert!(INDEXNOW_KEY_RECOMMENDED_LENGTH >= INDEXNOW_KEY_MIN_LENGTH);
        assert!(INDEXNOW_KEY_RECOMMENDED_LENGTH <= INDEXNOW_KEY_MAX_LENGTH);
    }

    #[test]
    fn test_timeout_durations() {
        assert!(DEFAULT_HTTP_TIMEOUT.as_secs() == DEFAULT_HTTP_TIMEOUT_SECS);
        assert!(HTTP_CONNECT_TIMEOUT.as_secs() == HTTP_CONNECT_TIMEOUT_SECS);
        assert!(HTTP_CONNECT_TIMEOUT < DEFAULT_HTTP_TIMEOUT);
    }

    #[test]
    fn test_retry_configuration() {
        assert!(DEFAULT_MAX_RETRIES >= 1);
        assert!(DEFAULT_BACKOFF_FACTOR > 1.0);
        assert!(MAX_RETRY_DELAY > DEFAULT_INITIAL_RETRY_DELAY);
    }

    #[test]
    fn test_path_helpers() {
        let config_dir = default_config_dir();
        assert!(config_dir
            .to_string_lossy()
            .contains(DEFAULT_CONFIG_DIR_NAME));

        let config_path = default_config_file_path();
        assert!(config_path
            .to_string_lossy()
            .ends_with(DEFAULT_CONFIG_FILE_NAME));

        let db_path = default_database_file_path();
        assert!(db_path
            .to_string_lossy()
            .ends_with(DEFAULT_DATABASE_FILE_NAME));

        let log_path = default_log_file_path();
        assert!(log_path.to_string_lossy().ends_with(DEFAULT_LOG_FILE_NAME));
    }

    #[test]
    fn test_url_validation_constants() {
        assert!(MAX_URL_LENGTH == 2048);
        assert!(SUPPORTED_URL_SCHEMES.contains(&"https"));
        assert!(SUPPORTED_URL_SCHEMES.contains(&"http"));
    }
}
