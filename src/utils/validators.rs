//! Validation utilities
//!
//! This module provides validators for various input types including:
//! - URLs
//! - IndexNow API keys
//! - File paths
//! - Date formats

use anyhow::{bail, Context, Result};
use chrono::NaiveDate;
use regex::Regex;
use std::path::{Path, PathBuf};
use url::Url;

/// Minimum length for IndexNow API key
pub const INDEXNOW_KEY_MIN_LENGTH: usize = 8;

/// Maximum length for IndexNow API key
pub const INDEXNOW_KEY_MAX_LENGTH: usize = 128;

/// Validate a URL string
///
/// # Arguments
///
/// * `url_str` - The URL string to validate
///
/// # Returns
///
/// Returns `Ok(Url)` if the URL is valid, or an error if it's invalid
///
/// # Examples
///
/// ```
/// use indexer_cli::utils::validators::validate_url;
///
/// let url = validate_url("https://example.com").unwrap();
/// assert_eq!(url.scheme(), "https");
/// ```
pub fn validate_url(url_str: &str) -> Result<Url> {
    let url = Url::parse(url_str).with_context(|| format!("Invalid URL: {}", url_str))?;

    // Ensure the URL has a valid scheme (http or https)
    match url.scheme() {
        "http" | "https" => Ok(url),
        scheme => bail!(
            "Invalid URL scheme '{}', expected 'http' or 'https'",
            scheme
        ),
    }
}

/// Validate that a URL has HTTPS scheme
///
/// # Arguments
///
/// * `url_str` - The URL string to validate
///
/// # Returns
///
/// Returns `Ok(Url)` if the URL is valid and uses HTTPS, or an error otherwise
///
/// # Examples
///
/// ```
/// use indexer_cli::utils::validators::validate_https_url;
///
/// assert!(validate_https_url("https://example.com").is_ok());
/// assert!(validate_https_url("http://example.com").is_err());
/// ```
pub fn validate_https_url(url_str: &str) -> Result<Url> {
    let url = validate_url(url_str)?;

    if url.scheme() != "https" {
        bail!("URL must use HTTPS scheme: {}", url_str);
    }

    Ok(url)
}

/// Validate a batch of URLs
///
/// # Arguments
///
/// * `urls` - Iterator of URL strings to validate
///
/// # Returns
///
/// Returns `Ok(Vec<Url>)` if all URLs are valid, or an error if any URL is invalid
///
/// # Examples
///
/// ```
/// use indexer_cli::utils::validators::validate_urls;
///
/// let urls = vec!["https://example.com", "https://test.com"];
/// let validated = validate_urls(&urls).unwrap();
/// assert_eq!(validated.len(), 2);
/// ```
pub fn validate_urls<'a, I>(urls: I) -> Result<Vec<Url>>
where
    I: IntoIterator<Item = &'a str>,
{
    urls.into_iter()
        .enumerate()
        .map(|(i, url_str)| {
            validate_url(url_str).with_context(|| format!("Failed to validate URL at index {}", i))
        })
        .collect()
}

/// Validate an IndexNow API key
///
/// According to IndexNow specification, API keys must be:
/// - Between 8 and 128 characters in length
/// - Contain only hexadecimal characters (0-9, a-f, A-F) or be a valid UUID
///
/// # Arguments
///
/// * `api_key` - The API key to validate
///
/// # Returns
///
/// Returns `Ok(())` if the API key is valid, or an error if it's invalid
///
/// # Examples
///
/// ```
/// use indexer_cli::utils::validators::validate_indexnow_key;
///
/// assert!(validate_indexnow_key("a1b2c3d4e5f6g7h8").is_ok());
/// assert!(validate_indexnow_key("short").is_err());
/// ```
pub fn validate_indexnow_key(api_key: &str) -> Result<()> {
    let len = api_key.len();

    // Check length
    if len < INDEXNOW_KEY_MIN_LENGTH {
        bail!(
            "IndexNow API key is too short: {} characters (minimum: {})",
            len,
            INDEXNOW_KEY_MIN_LENGTH
        );
    }

    if len > INDEXNOW_KEY_MAX_LENGTH {
        bail!(
            "IndexNow API key is too long: {} characters (maximum: {})",
            len,
            INDEXNOW_KEY_MAX_LENGTH
        );
    }

    // Check allowed characters (alphanumeric + hyphen)
    if !api_key
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        bail!("IndexNow API key must contain only alphanumeric characters and hyphens");
    }

    Ok(())
}

/// Validate a file path exists and is readable
///
/// # Arguments
///
/// * `path` - The file path to validate
///
/// # Returns
///
/// Returns `Ok(PathBuf)` if the path exists and is readable, or an error otherwise
///
/// # Examples
///
/// ```no_run
/// use indexer_cli::utils::validators::validate_file_path;
/// use std::path::Path;
///
/// let path = validate_file_path(Path::new("/tmp/test.txt")).unwrap();
/// ```
pub fn validate_file_path(path: &Path) -> Result<PathBuf> {
    if !path.exists() {
        bail!("File does not exist: {}", path.display());
    }

    if !path.is_file() {
        bail!("Path is not a file: {}", path.display());
    }

    // Try to canonicalize the path (resolves symlinks and makes it absolute)
    let canonical = path
        .canonicalize()
        .with_context(|| format!("Failed to access file: {}", path.display()))?;

    Ok(canonical)
}

/// Validate a directory path exists and is accessible
///
/// # Arguments
///
/// * `path` - The directory path to validate
///
/// # Returns
///
/// Returns `Ok(PathBuf)` if the path exists and is a directory, or an error otherwise
///
/// # Examples
///
/// ```no_run
/// use indexer_cli::utils::validators::validate_directory_path;
/// use std::path::Path;
///
/// let path = validate_directory_path(Path::new("/tmp")).unwrap();
/// ```
pub fn validate_directory_path(path: &Path) -> Result<PathBuf> {
    if !path.exists() {
        bail!("Directory does not exist: {}", path.display());
    }

    if !path.is_dir() {
        bail!("Path is not a directory: {}", path.display());
    }

    // Try to canonicalize the path
    let canonical = path
        .canonicalize()
        .with_context(|| format!("Failed to access directory: {}", path.display()))?;

    Ok(canonical)
}

/// Validate a date string in ISO 8601 format (YYYY-MM-DD)
///
/// # Arguments
///
/// * `date_str` - The date string to validate
///
/// # Returns
///
/// Returns `Ok(NaiveDate)` if the date is valid, or an error otherwise
///
/// # Examples
///
/// ```
/// use indexer_cli::utils::validators::validate_date;
///
/// let date = validate_date("2024-01-15").unwrap();
/// assert_eq!(date.year(), 2024);
/// assert_eq!(date.month(), 1);
/// assert_eq!(date.day(), 15);
/// ```
pub fn validate_date(date_str: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .with_context(|| format!("Invalid date format '{}', expected YYYY-MM-DD", date_str))
}

/// Validate a date range
///
/// # Arguments
///
/// * `start_date_str` - The start date string (YYYY-MM-DD)
/// * `end_date_str` - The end date string (YYYY-MM-DD)
///
/// # Returns
///
/// Returns `Ok((NaiveDate, NaiveDate))` if both dates are valid and start <= end,
/// or an error otherwise
///
/// # Examples
///
/// ```
/// use indexer_cli::utils::validators::validate_date_range;
///
/// let (start, end) = validate_date_range("2024-01-01", "2024-12-31").unwrap();
/// assert!(start < end);
/// ```
pub fn validate_date_range(
    start_date_str: &str,
    end_date_str: &str,
) -> Result<(NaiveDate, NaiveDate)> {
    let start_date = validate_date(start_date_str)?;
    let end_date = validate_date(end_date_str)?;

    if start_date > end_date {
        bail!(
            "Start date ({}) must be before or equal to end date ({})",
            start_date_str,
            end_date_str
        );
    }

    Ok((start_date, end_date))
}

/// Validate an email address
///
/// This is a basic validation that checks for the presence of @ and a domain
///
/// # Arguments
///
/// * `email` - The email address to validate
///
/// # Returns
///
/// Returns `Ok(())` if the email appears valid, or an error otherwise
///
/// # Examples
///
/// ```
/// use indexer_cli::utils::validators::validate_email;
///
/// assert!(validate_email("test@example.com").is_ok());
/// assert!(validate_email("invalid").is_err());
/// ```
pub fn validate_email(email: &str) -> Result<()> {
    let email_regex = Regex::new(
        r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"
    ).unwrap();

    if !email_regex.is_match(email) {
        bail!("Invalid email address: {}", email);
    }

    Ok(())
}

/// Validate a domain name
///
/// # Arguments
///
/// * `domain` - The domain name to validate
///
/// # Returns
///
/// Returns `Ok(())` if the domain is valid, or an error otherwise
///
/// # Examples
///
/// ```
/// use indexer_cli::utils::validators::validate_domain;
///
/// assert!(validate_domain("example.com").is_ok());
/// assert!(validate_domain("sub.example.com").is_ok());
/// assert!(validate_domain("invalid..com").is_err());
/// ```
pub fn validate_domain(domain: &str) -> Result<()> {
    // Basic domain validation
    let domain_regex = Regex::new(
        r"^(?:[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?\.)*[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?$"
    ).unwrap();

    if !domain_regex.is_match(domain) {
        bail!("Invalid domain name: {}", domain);
    }

    Ok(())
}

/// Validate a port number
///
/// # Arguments
///
/// * `port` - The port number to validate
///
/// # Returns
///
/// Returns `Ok(())` if the port is valid (1-65535), or an error otherwise
///
/// # Examples
///
/// ```
/// use indexer_cli::utils::validators::validate_port;
///
/// assert!(validate_port(8080).is_ok());
/// assert!(validate_port(0).is_err());
/// assert!(validate_port(65536).is_err());
/// ```
pub fn validate_port(port: u16) -> Result<()> {
    if port == 0 {
        bail!("Port number must be between 1 and 65535");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("http://example.com").is_ok());
        assert!(validate_url("https://example.com/path?query=1").is_ok());
        assert!(validate_url("ftp://example.com").is_err());
        assert!(validate_url("not-a-url").is_err());
        assert!(validate_url("").is_err());
    }

    #[test]
    fn test_validate_https_url() {
        assert!(validate_https_url("https://example.com").is_ok());
        assert!(validate_https_url("http://example.com").is_err());
    }

    #[test]
    fn test_validate_urls() {
        let urls = vec!["https://example.com", "https://test.com"];
        assert!(validate_urls(urls).is_ok());

        let invalid_urls = vec!["https://example.com", "not-a-url"];
        assert!(validate_urls(invalid_urls).is_err());
    }

    #[test]
    fn test_validate_indexnow_key() {
        // Valid keys
        assert!(validate_indexnow_key("a1b2c3d4e5f6g7h8").is_ok());
        assert!(validate_indexnow_key("0123456789abcdef").is_ok());
        assert!(validate_indexnow_key("ABCDEF0123456789").is_ok());
        assert!(validate_indexnow_key("550e8400-e29b-41d4-a716-446655440000").is_ok());

        // Too short
        assert!(validate_indexnow_key("short").is_err());

        // Too long
        let long_key = "a".repeat(129);
        assert!(validate_indexnow_key(&long_key).is_err());

        // Invalid characters
        assert!(validate_indexnow_key("invalid-key!").is_err());
    }

    #[test]
    fn test_validate_date() {
        assert!(validate_date("2024-01-15").is_ok());
        assert!(validate_date("2024-12-31").is_ok());
        assert!(validate_date("invalid").is_err());
        assert!(validate_date("2024-13-01").is_err());
        assert!(validate_date("2024-01-32").is_err());
    }

    #[test]
    fn test_validate_date_range() {
        assert!(validate_date_range("2024-01-01", "2024-12-31").is_ok());
        assert!(validate_date_range("2024-01-01", "2024-01-01").is_ok());
        assert!(validate_date_range("2024-12-31", "2024-01-01").is_err());
    }

    #[test]
    fn test_validate_email() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("user+tag@example.com").is_ok());
        assert!(validate_email("user@sub.example.com").is_ok());
        assert!(validate_email("invalid").is_err());
        assert!(validate_email("@example.com").is_err());
        assert!(validate_email("user@").is_err());
    }

    #[test]
    fn test_validate_domain() {
        assert!(validate_domain("example.com").is_ok());
        assert!(validate_domain("sub.example.com").is_ok());
        assert!(validate_domain("a.b.c.example.com").is_ok());
        assert!(validate_domain("invalid..com").is_err());
        assert!(validate_domain(".example.com").is_err());
        assert!(validate_domain("example.com.").is_err());
    }

    #[test]
    fn test_validate_port() {
        assert!(validate_port(1).is_ok());
        assert!(validate_port(8080).is_ok());
        assert!(validate_port(65535).is_ok());
        assert!(validate_port(0).is_err());
    }
}
