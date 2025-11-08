//! Utilities module
//!
//! This module provides common utility functions used throughout the application.

pub mod file;
pub mod logger;
pub mod retry;
pub mod validators;

// Re-export commonly used items for convenience
pub use logger::{init_logger, init_simple_logger, LogDestination, LogRotation, LoggerConfig};
pub use retry::{retry, retry_with_backoff, retry_with_condition, RetryConfig, ShouldRetry};
pub use validators::{
    validate_date, validate_date_range, validate_domain, validate_email, validate_file_path,
    validate_https_url, validate_indexnow_key, validate_port, validate_url, validate_urls,
    INDEXNOW_KEY_MAX_LENGTH, INDEXNOW_KEY_MIN_LENGTH,
};

pub use file::{
    dir_exists, ensure_dir_exists, ensure_dir_exists_sync, expand_path, file_exists,
    get_file_extension, read_content, read_file, read_file_sync, write_bytes, write_file,
    write_file_sync,
};
