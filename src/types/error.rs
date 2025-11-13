//! Error type definitions for the indexer-cli application.
//!
//! This module provides comprehensive error handling using `thiserror` to cover all
//! possible error scenarios including configuration errors, API errors, network errors,
//! database errors, file I/O errors, sitemap parsing errors, and validation errors.

use std::path::PathBuf;
use thiserror::Error;

/// The main error type for the indexer-cli application.
///
/// This enum covers all possible error scenarios that can occur during the execution
/// of the application. Each variant includes relevant context information to help
/// with debugging and user feedback.
#[derive(Error, Debug)]
pub enum IndexerError {
    // ============================================================================
    // Configuration Errors
    // ============================================================================
    /// Configuration file not found
    #[error("Configuration file not found: {path}")]
    ConfigFileNotFound { path: PathBuf },

    /// Configuration file format error (invalid YAML/JSON)
    #[error("Configuration file format error: {message}")]
    ConfigFormatError { message: String },

    /// Configuration validation failed
    #[error("Configuration validation failed: {message}")]
    ConfigValidationError { message: String },

    /// Missing required configuration field
    #[error("Missing required configuration field: {field}")]
    ConfigMissingField { field: String },

    /// Invalid configuration value
    #[error("Invalid configuration value for '{field}': {message}")]
    ConfigInvalidValue { field: String, message: String },

    /// Configuration file permission error
    #[error("Permission denied accessing configuration file: {path}")]
    ConfigPermissionDenied { path: PathBuf },

    // ============================================================================
    // Google Indexing API Errors
    // ============================================================================
    /// Google API authentication failed
    #[error("Google API authentication failed: {message}")]
    GoogleAuthError { message: String },

    /// Google service account file not found
    #[error("Google service account file not found: {path}")]
    GoogleServiceAccountNotFound { path: PathBuf },

    /// Invalid Google service account JSON
    #[error("Invalid Google service account JSON: {message}")]
    GoogleServiceAccountInvalid { message: String },

    /// Google API request failed
    #[error("Google API request failed (HTTP {status_code}): {message}")]
    GoogleApiError { status_code: u16, message: String },

    /// Google API quota exceeded
    #[error("Google API quota exceeded: {message}")]
    GoogleQuotaExceeded { message: String },

    /// Google API rate limit exceeded
    #[error("Google API rate limit exceeded. Please try again later.")]
    GoogleRateLimitExceeded,

    /// Google API permission denied (403)
    #[error("Google API permission denied: {message}. Make sure the service account is added as owner in Search Console.")]
    GooglePermissionDenied { message: String },

    /// Google API invalid request (400)
    #[error("Google API invalid request: {message}")]
    GoogleInvalidRequest { message: String },

    // ============================================================================
    // IndexNow API Errors
    // ============================================================================
    /// IndexNow API request failed
    #[error("IndexNow API request failed (HTTP {status_code}): {message}")]
    IndexNowApiError { status_code: u16, message: String },

    /// IndexNow API key invalid (403)
    #[error("IndexNow API key is invalid or verification failed")]
    IndexNowInvalidKey,

    /// IndexNow API bad request (400)
    #[error("IndexNow API bad request: {message}")]
    IndexNowBadRequest { message: String },

    /// IndexNow API unprocessable entity (422)
    #[error(
        "IndexNow API unprocessable entity: {message}. URL may not belong to host or key mismatch."
    )]
    IndexNowUnprocessableEntity { message: String },

    /// IndexNow API rate limit (429)
    #[error("IndexNow API rate limit exceeded (possible spam detection). Please try again later.")]
    IndexNowRateLimitExceeded,

    /// IndexNow key file not accessible
    #[error("IndexNow key file not accessible at {url}: {message}")]
    IndexNowKeyFileNotAccessible { url: String, message: String },

    /// IndexNow key file content mismatch
    #[error("IndexNow key file content mismatch. Expected '{expected}' but got '{actual}'")]
    IndexNowKeyFileMismatch { expected: String, actual: String },

    /// IndexNow key location host mismatch
    #[error(
        "IndexNow key location host mismatch: expected '{expected_host}', got '{actual_host}'"
    )]
    IndexNowKeyLocationMismatch {
        expected_host: String,
        actual_host: String,
    },

    // ============================================================================
    // Network Errors
    // ============================================================================
    /// HTTP request failed
    #[error("HTTP request failed: {message}")]
    HttpRequestFailed { message: String },

    /// HTTP connection timeout
    #[error("HTTP connection timeout after {seconds} seconds")]
    HttpTimeout { seconds: u64 },

    /// Network unreachable
    #[error("Network unreachable: {message}")]
    NetworkUnreachable { message: String },

    /// DNS resolution failed
    #[error("DNS resolution failed for {host}: {message}")]
    DnsResolutionFailed { host: String, message: String },

    /// SSL/TLS error
    #[error("SSL/TLS error: {message}")]
    SslError { message: String },

    // ============================================================================
    // Database Errors
    // ============================================================================
    /// Database connection failed
    #[error("Database connection failed: {message}")]
    DatabaseConnectionFailed { message: String },

    /// Database query failed
    #[error("Database query failed: {message}")]
    DatabaseQueryFailed { message: String },

    /// Database migration failed
    #[error("Database migration failed: {message}")]
    DatabaseMigrationFailed { message: String },

    /// Database file not found
    #[error("Database file not found: {path}")]
    DatabaseFileNotFound { path: PathBuf },

    /// Database constraint violation
    #[error("Database constraint violation: {message}")]
    DatabaseConstraintViolation { message: String },

    /// Database transaction failed
    #[error("Database transaction failed: {message}")]
    DatabaseTransactionFailed { message: String },

    // ============================================================================
    // File I/O Errors
    // ============================================================================
    /// File not found
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },

    /// File read error
    #[error("Failed to read file {path}: {message}")]
    FileReadError { path: PathBuf, message: String },

    /// File write error
    #[error("Failed to write file {path}: {message}")]
    FileWriteError { path: PathBuf, message: String },

    /// File permission denied
    #[error("Permission denied accessing file: {path}")]
    FilePermissionDenied { path: PathBuf },

    /// Directory creation failed
    #[error("Failed to create directory {path}: {message}")]
    DirectoryCreationFailed { path: PathBuf, message: String },

    /// Invalid file format
    #[error("Invalid file format for {path}: expected {expected}, got {actual}")]
    InvalidFileFormat {
        path: PathBuf,
        expected: String,
        actual: String,
    },

    // ============================================================================
    // Sitemap Parsing Errors
    // ============================================================================
    /// Sitemap parsing failed
    #[error("Failed to parse sitemap: {message}")]
    SitemapParseError { message: String },

    /// Invalid sitemap URL
    #[error("Invalid sitemap URL: {url}")]
    SitemapInvalidUrl { url: String },

    /// Sitemap download failed
    #[error("Failed to download sitemap from {url}: {message}")]
    SitemapDownloadFailed { url: String, message: String },

    /// Sitemap too large
    #[error("Sitemap too large: {size} bytes exceeds limit of {limit} bytes")]
    SitemapTooLarge { size: usize, limit: usize },

    /// Sitemap has too many URLs
    #[error("Sitemap has too many URLs: {count} exceeds limit of {limit}")]
    SitemapTooManyUrls { count: usize, limit: usize },

    /// Invalid sitemap XML
    #[error("Invalid sitemap XML: {message}")]
    SitemapInvalidXml { message: String },

    /// Sitemap index recursion limit exceeded
    #[error("Sitemap index recursion limit exceeded (max {limit})")]
    SitemapRecursionLimitExceeded { limit: usize },

    /// Unsupported sitemap format
    #[error("Unsupported sitemap format: {format}")]
    SitemapUnsupportedFormat { format: String },

    // ============================================================================
    // Validation Errors
    // ============================================================================
    /// Invalid URL
    #[error("Invalid URL: {url}")]
    InvalidUrl { url: String },

    /// URL validation failed
    #[error("URL validation failed for '{url}': {message}")]
    UrlValidationFailed { url: String, message: String },

    /// Invalid API key
    #[error("Invalid API key: {message}")]
    InvalidApiKey { message: String },

    /// Invalid date format
    #[error("Invalid date format: {value}. Expected format: {expected}")]
    InvalidDateFormat { value: String, expected: String },

    /// Invalid regex pattern
    #[error("Invalid regex pattern: {pattern}. Error: {message}")]
    InvalidRegexPattern { pattern: String, message: String },

    /// Value out of range
    #[error("Value out of range for '{field}': {value}. Expected range: {min}-{max}")]
    ValueOutOfRange {
        field: String,
        value: String,
        min: String,
        max: String,
    },

    /// Missing required field
    #[error("Missing required field: {field}")]
    MissingRequiredField { field: String },

    // ============================================================================
    // Batch Processing Errors
    // ============================================================================
    /// Batch processing failed
    #[error("Batch processing failed: {successful} succeeded, {failed} failed")]
    BatchProcessingFailed { successful: usize, failed: usize },

    /// Batch size exceeds limit
    #[error("Batch size {size} exceeds limit of {limit}")]
    BatchSizeExceedsLimit { size: usize, limit: usize },

    // ============================================================================
    // Serialization/Deserialization Errors
    // ============================================================================
    /// JSON serialization error
    #[error("JSON serialization error: {message}")]
    JsonSerializationError { message: String },

    /// JSON deserialization error
    #[error("JSON deserialization error: {message}")]
    JsonDeserializationError { message: String },

    /// YAML serialization error
    #[error("YAML serialization error: {message}")]
    YamlSerializationError { message: String },

    /// YAML deserialization error
    #[error("YAML deserialization error: {message}")]
    YamlDeserializationError { message: String },

    // ============================================================================
    // Watch Mode Errors
    // ============================================================================
    /// Watch mode initialization failed
    #[error("Watch mode initialization failed: {message}")]
    WatchModeInitFailed { message: String },

    /// PID file error
    #[error("PID file error at {path}: {message}")]
    PidFileError { path: PathBuf, message: String },

    /// Process already running
    #[error("Process already running with PID {pid}")]
    ProcessAlreadyRunning { pid: u32 },

    // ============================================================================
    // Retry Errors
    // ============================================================================
    /// All retry attempts failed
    #[error("All {attempts} retry attempts failed: {message}")]
    RetryAttemptsExhausted { attempts: u32, message: String },

    // ============================================================================
    // General Errors
    // ============================================================================
    /// Operation cancelled by user
    #[error("Operation cancelled by user")]
    OperationCancelled,

    /// Unsupported operation
    #[error("Unsupported operation: {operation}")]
    UnsupportedOperation { operation: String },

    /// Internal error
    #[error("Internal error: {message}")]
    InternalError { message: String },

    /// Feature not implemented
    #[error("Feature not implemented: {feature}")]
    NotImplemented { feature: String },

    // ============================================================================
    // External Error Wrappers
    // ============================================================================
    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Reqwest HTTP client error
    #[error("HTTP client error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    /// Serde JSON error
    #[error("JSON error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    /// Serde YAML error
    #[error("YAML error: {0}")]
    SerdeYamlError(#[from] serde_yaml::Error),

    /// URL parse error
    #[error("URL parse error: {0}")]
    UrlParseError(#[from] url::ParseError),

    /// Regex error
    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),

    /// SQLite/Rusqlite error
    #[error("Database error: {0}")]
    RusqliteError(#[from] rusqlite::Error),

    /// XML parsing error
    #[error("XML parsing error: {0}")]
    XmlParseError(#[from] roxmltree::Error),

    /// Chrono parse error
    #[error("Date/time parsing error: {0}")]
    ChronoParseError(#[from] chrono::ParseError),

    /// Config library error
    #[error("Configuration error: {0}")]
    ConfigError(#[from] config::ConfigError),

    /// Generic error from anyhow
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

// Implement conversion from IndexerError to i32 exit codes
impl IndexerError {
    /// Convert error to exit code for CLI
    pub fn exit_code(&self) -> i32 {
        match self {
            // Configuration errors: 10-19
            Self::ConfigFileNotFound { .. }
            | Self::ConfigFormatError { .. }
            | Self::ConfigValidationError { .. }
            | Self::ConfigMissingField { .. }
            | Self::ConfigInvalidValue { .. }
            | Self::ConfigPermissionDenied { .. } => 10,

            // Google API errors: 20-29
            Self::GoogleAuthError { .. }
            | Self::GoogleServiceAccountNotFound { .. }
            | Self::GoogleServiceAccountInvalid { .. }
            | Self::GoogleApiError { .. }
            | Self::GoogleQuotaExceeded { .. }
            | Self::GoogleRateLimitExceeded
            | Self::GooglePermissionDenied { .. }
            | Self::GoogleInvalidRequest { .. } => 20,

            // IndexNow API errors: 30-39
            Self::IndexNowApiError { .. }
            | Self::IndexNowInvalidKey
            | Self::IndexNowBadRequest { .. }
            | Self::IndexNowUnprocessableEntity { .. }
            | Self::IndexNowRateLimitExceeded
            | Self::IndexNowKeyFileNotAccessible { .. }
            | Self::IndexNowKeyFileMismatch { .. }
            | Self::IndexNowKeyLocationMismatch { .. } => 30,

            // Network errors: 40-49
            Self::HttpRequestFailed { .. }
            | Self::HttpTimeout { .. }
            | Self::NetworkUnreachable { .. }
            | Self::DnsResolutionFailed { .. }
            | Self::SslError { .. } => 40,

            // Database errors: 50-59
            Self::DatabaseConnectionFailed { .. }
            | Self::DatabaseQueryFailed { .. }
            | Self::DatabaseMigrationFailed { .. }
            | Self::DatabaseFileNotFound { .. }
            | Self::DatabaseConstraintViolation { .. }
            | Self::DatabaseTransactionFailed { .. } => 50,

            // File I/O errors: 60-69
            Self::FileNotFound { .. }
            | Self::FileReadError { .. }
            | Self::FileWriteError { .. }
            | Self::FilePermissionDenied { .. }
            | Self::DirectoryCreationFailed { .. }
            | Self::InvalidFileFormat { .. } => 60,

            // Sitemap errors: 70-79
            Self::SitemapParseError { .. }
            | Self::SitemapInvalidUrl { .. }
            | Self::SitemapDownloadFailed { .. }
            | Self::SitemapTooLarge { .. }
            | Self::SitemapTooManyUrls { .. }
            | Self::SitemapInvalidXml { .. }
            | Self::SitemapRecursionLimitExceeded { .. }
            | Self::SitemapUnsupportedFormat { .. } => 70,

            // Validation errors: 80-89
            Self::InvalidUrl { .. }
            | Self::UrlValidationFailed { .. }
            | Self::InvalidApiKey { .. }
            | Self::InvalidDateFormat { .. }
            | Self::InvalidRegexPattern { .. }
            | Self::ValueOutOfRange { .. }
            | Self::MissingRequiredField { .. } => 80,

            // User-initiated: 90-99
            Self::OperationCancelled => 130, // Standard SIGINT exit code

            // General errors: 1
            _ => 1,
        }
    }

    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::GoogleRateLimitExceeded
                | Self::IndexNowRateLimitExceeded
                | Self::HttpTimeout { .. }
                | Self::NetworkUnreachable { .. }
                | Self::HttpRequestFailed { .. }
                | Self::SitemapDownloadFailed { .. }
        )
    }

    /// Check if the error is a network error
    pub fn is_network_error(&self) -> bool {
        matches!(
            self,
            Self::HttpRequestFailed { .. }
                | Self::HttpTimeout { .. }
                | Self::NetworkUnreachable { .. }
                | Self::DnsResolutionFailed { .. }
                | Self::SslError { .. }
                | Self::ReqwestError(_)
        )
    }

    /// Check if the error is an API error
    pub fn is_api_error(&self) -> bool {
        matches!(
            self,
            Self::GoogleApiError { .. }
                | Self::GoogleAuthError { .. }
                | Self::GoogleQuotaExceeded { .. }
                | Self::GoogleRateLimitExceeded
                | Self::GooglePermissionDenied { .. }
                | Self::GoogleInvalidRequest { .. }
                | Self::IndexNowApiError { .. }
                | Self::IndexNowInvalidKey
                | Self::IndexNowBadRequest { .. }
                | Self::IndexNowUnprocessableEntity { .. }
                | Self::IndexNowRateLimitExceeded
        )
    }

    /// Check if the error is a configuration error
    pub fn is_config_error(&self) -> bool {
        matches!(
            self,
            Self::ConfigFileNotFound { .. }
                | Self::ConfigFormatError { .. }
                | Self::ConfigValidationError { .. }
                | Self::ConfigMissingField { .. }
                | Self::ConfigInvalidValue { .. }
                | Self::ConfigPermissionDenied { .. }
        )
    }
}

// Implement ShouldRetry trait for retry logic
impl crate::utils::retry::ShouldRetry for IndexerError {
    fn should_retry(&self) -> bool {
        self.is_retryable()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = IndexerError::ConfigFileNotFound {
            path: PathBuf::from("/path/to/config.yaml"),
        };
        assert_eq!(
            error.to_string(),
            "Configuration file not found: /path/to/config.yaml"
        );
    }

    #[test]
    fn test_error_exit_code() {
        let error = IndexerError::ConfigFileNotFound {
            path: PathBuf::from("/path/to/config.yaml"),
        };
        assert_eq!(error.exit_code(), 10);

        let error = IndexerError::GoogleRateLimitExceeded;
        assert_eq!(error.exit_code(), 20);
    }

    #[test]
    fn test_is_retryable() {
        let error = IndexerError::GoogleRateLimitExceeded;
        assert!(error.is_retryable());

        let error = IndexerError::ConfigFileNotFound {
            path: PathBuf::from("/path/to/config.yaml"),
        };
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_is_network_error() {
        let error = IndexerError::HttpTimeout { seconds: 30 };
        assert!(error.is_network_error());

        let error = IndexerError::ConfigFileNotFound {
            path: PathBuf::from("/path/to/config.yaml"),
        };
        assert!(!error.is_network_error());
    }

    #[test]
    fn test_is_api_error() {
        let error = IndexerError::GoogleRateLimitExceeded;
        assert!(error.is_api_error());

        let error = IndexerError::IndexNowInvalidKey;
        assert!(error.is_api_error());

        let error = IndexerError::ConfigFileNotFound {
            path: PathBuf::from("/path/to/config.yaml"),
        };
        assert!(!error.is_api_error());
    }
}
