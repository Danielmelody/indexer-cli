//! URL processor service
//!
//! This module provides URL processing and normalization utilities including:
//! - URL normalization (scheme, fragment, trailing slash handling)
//! - Host extraction and validation
//! - URL deduplication
//! - Batch processing utilities

use crate::types::error::IndexerError;
use std::collections::HashSet;
use url::Url;

/// URL processor for normalizing and validating URLs
pub struct UrlProcessor;

impl UrlProcessor {
    /// Normalize a URL by applying standard transformations
    ///
    /// Normalization includes:
    /// - Converting scheme to lowercase
    /// - Removing fragment (#section)
    /// - Removing default ports (80 for HTTP, 443 for HTTPS)
    /// - Optionally normalizing trailing slashes
    ///
    /// # Examples
    ///
    /// ```
    /// use indexer_cli::services::url_processor::UrlProcessor;
    ///
    /// let normalized = UrlProcessor::normalize_url("HTTPS://Example.com/Path#fragment").unwrap();
    /// assert_eq!(normalized, "https://placeholder.test/Path");
    /// ```
    pub fn normalize_url(url: &str) -> Result<String, IndexerError> {
        let mut parsed = Url::parse(url).map_err(|_| IndexerError::InvalidUrl {
            url: url.to_string(),
        })?;

        // Remove fragment
        parsed.set_fragment(None);

        // Normalize scheme to lowercase (URL crate already does this)
        // Normalize host to lowercase (URL crate already does this)

        // Remove default ports
        if let Some(port) = parsed.port() {
            let scheme = parsed.scheme();
            if (scheme == "http" && port == 80) || (scheme == "https" && port == 443) {
                let _ = parsed.set_port(None);
            }
        }

        Ok(parsed.to_string())
    }

    /// Normalize a URL with trailing slash handling
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to normalize
    /// * `add_trailing_slash` - If true, adds trailing slash to paths without file extensions
    ///
    /// # Examples
    ///
    /// ```
    /// use indexer_cli::services::url_processor::UrlProcessor;
    ///
    /// let normalized = UrlProcessor::normalize_url_with_trailing_slash(
    ///     "https://placeholder.test/path",
    ///     true
    /// ).unwrap();
    /// assert_eq!(normalized, "https://placeholder.test/path/");
    /// ```
    pub fn normalize_url_with_trailing_slash(
        url: &str,
        add_trailing_slash: bool,
    ) -> Result<String, IndexerError> {
        let mut normalized = Self::normalize_url(url)?;

        if add_trailing_slash {
            let parsed = Url::parse(&normalized)?;
            let path = parsed.path();

            // Only add trailing slash if:
            // 1. Path doesn't already end with /
            // 2. Path doesn't appear to be a file (no extension)
            if !path.ends_with('/') && !Self::has_file_extension(path) {
                normalized.push('/');
            }
        } else {
            // Remove trailing slash if it exists (except for root path)
            let parsed = Url::parse(&normalized)?;
            let path = parsed.path();

            if path.len() > 1 && path.ends_with('/') {
                normalized.pop();
            }
        }

        Ok(normalized)
    }

    /// Extract the host (domain) from a URL
    ///
    /// # Examples
    ///
    /// ```
    /// use indexer_cli::services::url_processor::UrlProcessor;
    ///
    /// let host = UrlProcessor::extract_host("https://www.placeholder.test/path?query=1").unwrap();
    /// assert_eq!(host, "www.placeholder.test");
    /// ```
    pub fn extract_host(url: &str) -> Result<String, IndexerError> {
        let parsed = Url::parse(url).map_err(|_| IndexerError::InvalidUrl {
            url: url.to_string(),
        })?;

        parsed
            .host_str()
            .map(|h| h.to_string())
            .ok_or_else(|| IndexerError::InvalidUrl {
                url: url.to_string(),
            })
    }

    /// Validate that all URLs belong to the specified host
    ///
    /// # Arguments
    ///
    /// * `urls` - List of URLs to validate
    /// * `host` - The expected host (domain)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if all URLs belong to the host, otherwise returns an error
    /// with the first mismatched URL.
    ///
    /// # Examples
    ///
    /// ```
    /// use indexer_cli::services::url_processor::UrlProcessor;
    ///
    /// let urls = vec![
    ///     "https://placeholder.test/page1".to_string(),
    ///     "https://placeholder.test/page2".to_string(),
    /// ];
    ///
    /// assert!(UrlProcessor::validate_urls_for_host(&urls, "placeholder.test").is_ok());
    /// ```
    pub fn validate_urls_for_host(urls: &[String], host: &str) -> Result<(), IndexerError> {
        for url in urls {
            let url_host = Self::extract_host(url)?;

            if url_host != host {
                return Err(IndexerError::UrlValidationFailed {
                    url: url.clone(),
                    message: format!(
                        "URL host '{}' does not match expected host '{}'",
                        url_host, host
                    ),
                });
            }
        }

        Ok(())
    }

    /// Deduplicate a list of URLs
    ///
    /// This function normalizes URLs before deduplication to ensure that
    /// URLs that are functionally identical are treated as duplicates.
    ///
    /// # Examples
    ///
    /// ```
    /// use indexer_cli::services::url_processor::UrlProcessor;
    ///
    /// let urls = vec![
    ///     "https://placeholder.test/page".to_string(),
    ///     "https://placeholder.test/page#section".to_string(),
    ///     "HTTPS://EXAMPLE.COM/page".to_string(),
    /// ];
    ///
    /// let deduplicated = UrlProcessor::deduplicate_urls(urls).unwrap();
    /// assert_eq!(deduplicated.len(), 1);
    /// ```
    pub fn deduplicate_urls(urls: Vec<String>) -> Result<Vec<String>, IndexerError> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();

        for url in urls {
            // Normalize before checking for duplicates
            let normalized = Self::normalize_url(&url)?;

            if seen.insert(normalized.clone()) {
                result.push(normalized);
            }
        }

        Ok(result)
    }

    /// Deduplicate URLs while preserving original order
    ///
    /// Similar to `deduplicate_urls` but maintains the original order of URLs
    /// (first occurrence is kept).
    pub fn deduplicate_urls_preserve_order(urls: Vec<String>) -> Result<Vec<String>, IndexerError> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();

        for url in urls {
            let normalized = Self::normalize_url(&url)?;

            if seen.insert(normalized.clone()) {
                result.push(normalized);
            }
        }

        Ok(result)
    }

    /// Split URLs into batches of specified size
    ///
    /// This is useful for processing URLs in chunks, e.g., for API rate limiting
    /// or parallel processing.
    ///
    /// # Examples
    ///
    /// ```
    /// use indexer_cli::services::url_processor::UrlProcessor;
    ///
    /// let urls = vec![
    ///     "https://placeholder.test/1".to_string(),
    ///     "https://placeholder.test/2".to_string(),
    ///     "https://placeholder.test/3".to_string(),
    ///     "https://placeholder.test/4".to_string(),
    ///     "https://placeholder.test/5".to_string(),
    /// ];
    ///
    /// let batches = UrlProcessor::batch_urls(urls, 2);
    /// assert_eq!(batches.len(), 3);
    /// assert_eq!(batches[0].len(), 2);
    /// assert_eq!(batches[1].len(), 2);
    /// assert_eq!(batches[2].len(), 1);
    /// ```
    pub fn batch_urls(urls: Vec<String>, batch_size: usize) -> Vec<Vec<String>> {
        if batch_size == 0 {
            return vec![urls];
        }

        urls.chunks(batch_size)
            .map(|chunk| chunk.to_vec())
            .collect()
    }

    /// Check if a path appears to have a file extension
    fn has_file_extension(path: &str) -> bool {
        if let Some(last_segment) = path.split('/').last() {
            // Check if the last segment contains a dot and has characters after it
            if let Some(dot_pos) = last_segment.rfind('.') {
                return dot_pos < last_segment.len() - 1 && dot_pos > 0;
            }
        }
        false
    }

    /// Validate URL format
    ///
    /// Checks if the URL has a valid format and uses HTTP or HTTPS scheme.
    ///
    /// # Examples
    ///
    /// ```
    /// use indexer_cli::services::url_processor::UrlProcessor;
    ///
    /// assert!(UrlProcessor::validate_url("https://placeholder.test").is_ok());
    /// assert!(UrlProcessor::validate_url("ftp://placeholder.test").is_err());
    /// ```
    pub fn validate_url(url: &str) -> Result<(), IndexerError> {
        let parsed = Url::parse(url).map_err(|_| IndexerError::InvalidUrl {
            url: url.to_string(),
        })?;

        let scheme = parsed.scheme();
        if scheme != "http" && scheme != "https" {
            return Err(IndexerError::UrlValidationFailed {
                url: url.to_string(),
                message: format!(
                    "Invalid scheme '{}'. Only http and https are supported.",
                    scheme
                ),
            });
        }

        if parsed.host_str().is_none() {
            return Err(IndexerError::UrlValidationFailed {
                url: url.to_string(),
                message: "URL must have a host".to_string(),
            });
        }

        Ok(())
    }

    /// Validate a batch of URLs
    ///
    /// Returns a tuple of (valid_urls, invalid_urls_with_errors)
    pub fn validate_urls(urls: Vec<String>) -> (Vec<String>, Vec<(String, IndexerError)>) {
        let mut valid = Vec::new();
        let mut invalid = Vec::new();

        for url in urls {
            match Self::validate_url(&url) {
                Ok(_) => valid.push(url),
                Err(e) => invalid.push((url, e)),
            }
        }

        (valid, invalid)
    }

    /// Extract the path component from a URL
    ///
    /// # Examples
    ///
    /// ```
    /// use indexer_cli::services::url_processor::UrlProcessor;
    ///
    /// let path = UrlProcessor::extract_path("https://placeholder.test/path/to/page?query=1").unwrap();
    /// assert_eq!(path, "/path/to/page");
    /// ```
    pub fn extract_path(url: &str) -> Result<String, IndexerError> {
        let parsed = Url::parse(url).map_err(|_| IndexerError::InvalidUrl {
            url: url.to_string(),
        })?;

        Ok(parsed.path().to_string())
    }

    /// Convert relative URL to absolute URL given a base URL
    ///
    /// # Examples
    ///
    /// ```
    /// use indexer_cli::services::url_processor::UrlProcessor;
    ///
    /// let absolute = UrlProcessor::resolve_relative_url(
    ///     "https://placeholder.test/base/",
    ///     "page.html"
    /// ).unwrap();
    /// assert_eq!(absolute, "https://placeholder.test/base/page.html");
    /// ```
    pub fn resolve_relative_url(base: &str, relative: &str) -> Result<String, IndexerError> {
        let base_url = Url::parse(base).map_err(|_| IndexerError::InvalidUrl {
            url: base.to_string(),
        })?;

        let resolved = base_url
            .join(relative)
            .map_err(|_| IndexerError::UrlValidationFailed {
                url: relative.to_string(),
                message: format!(
                    "Failed to resolve relative URL '{}' with base '{}'",
                    relative, base
                ),
            })?;

        Ok(resolved.to_string())
    }

    /// Check if two URLs point to the same resource (after normalization)
    ///
    /// # Examples
    ///
    /// ```
    /// use indexer_cli::services::url_processor::UrlProcessor;
    ///
    /// assert!(UrlProcessor::urls_equal(
    ///     "https://placeholder.test/page",
    ///     "https://placeholder.test/page#section"
    /// ).unwrap());
    /// ```
    pub fn urls_equal(url1: &str, url2: &str) -> Result<bool, IndexerError> {
        let normalized1 = Self::normalize_url(url1)?;
        let normalized2 = Self::normalize_url(url2)?;

        Ok(normalized1 == normalized2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_url() {
        // Remove fragment
        assert_eq!(
            UrlProcessor::normalize_url("https://placeholder.test/page#section").unwrap(),
            "https://placeholder.test/page"
        );

        // Lowercase scheme and host
        assert_eq!(
            UrlProcessor::normalize_url("HTTPS://EXAMPLE.COM/Path").unwrap(),
            "https://placeholder.test/Path"
        );

        // Remove default ports
        assert_eq!(
            UrlProcessor::normalize_url("https://placeholder.test:443/page").unwrap(),
            "https://placeholder.test/page"
        );

        assert_eq!(
            UrlProcessor::normalize_url("http://placeholder.test:80/page").unwrap(),
            "http://placeholder.test/page"
        );

        // Keep non-default ports
        assert_eq!(
            UrlProcessor::normalize_url("https://placeholder.test:8080/page").unwrap(),
            "https://placeholder.test:8080/page"
        );
    }

    #[test]
    fn test_normalize_url_with_trailing_slash() {
        // Add trailing slash
        assert_eq!(
            UrlProcessor::normalize_url_with_trailing_slash("https://placeholder.test/path", true)
                .unwrap(),
            "https://placeholder.test/path/"
        );

        // Don't add to files
        assert_eq!(
            UrlProcessor::normalize_url_with_trailing_slash(
                "https://placeholder.test/file.html",
                true
            )
            .unwrap(),
            "https://placeholder.test/file.html"
        );

        // Remove trailing slash
        assert_eq!(
            UrlProcessor::normalize_url_with_trailing_slash(
                "https://placeholder.test/path/",
                false
            )
            .unwrap(),
            "https://placeholder.test/path"
        );

        // Keep root slash
        assert_eq!(
            UrlProcessor::normalize_url_with_trailing_slash("https://placeholder.test/", false)
                .unwrap(),
            "https://placeholder.test/"
        );
    }

    #[test]
    fn test_extract_host() {
        assert_eq!(
            UrlProcessor::extract_host("https://www.placeholder.test/path?query=1").unwrap(),
            "www.placeholder.test"
        );

        assert_eq!(
            UrlProcessor::extract_host("https://placeholder.test:8080/path").unwrap(),
            "placeholder.test"
        );
    }

    #[test]
    fn test_validate_urls_for_host() {
        let urls = vec![
            "https://placeholder.test/page1".to_string(),
            "https://placeholder.test/page2".to_string(),
        ];

        assert!(UrlProcessor::validate_urls_for_host(&urls, "placeholder.test").is_ok());

        let mixed_urls = vec![
            "https://placeholder.test/page1".to_string(),
            "https://other.com/page2".to_string(),
        ];

        assert!(UrlProcessor::validate_urls_for_host(&mixed_urls, "placeholder.test").is_err());
    }

    #[test]
    fn test_deduplicate_urls() {
        let urls = vec![
            "https://placeholder.test/page".to_string(),
            "https://placeholder.test/page#section".to_string(),
            "HTTPS://EXAMPLE.COM/page".to_string(),
            "https://placeholder.test/other".to_string(),
        ];

        let deduplicated = UrlProcessor::deduplicate_urls(urls).unwrap();
        assert_eq!(deduplicated.len(), 2);
        assert!(deduplicated.contains(&"https://placeholder.test/page".to_string()));
        assert!(deduplicated.contains(&"https://placeholder.test/other".to_string()));
    }

    #[test]
    fn test_batch_urls() {
        let urls = vec![
            "https://placeholder.test/1".to_string(),
            "https://placeholder.test/2".to_string(),
            "https://placeholder.test/3".to_string(),
            "https://placeholder.test/4".to_string(),
            "https://placeholder.test/5".to_string(),
        ];

        let batches = UrlProcessor::batch_urls(urls.clone(), 2);
        assert_eq!(batches.len(), 3);
        assert_eq!(batches[0].len(), 2);
        assert_eq!(batches[1].len(), 2);
        assert_eq!(batches[2].len(), 1);

        // Test with batch_size = 0 (returns all in one batch)
        let batches = UrlProcessor::batch_urls(urls.clone(), 0);
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].len(), 5);
    }

    #[test]
    fn test_has_file_extension() {
        assert!(UrlProcessor::has_file_extension("/path/file.html"));
        assert!(UrlProcessor::has_file_extension("/path/file.txt"));
        assert!(!UrlProcessor::has_file_extension("/path/to/directory"));
        assert!(!UrlProcessor::has_file_extension("/path/.hidden"));
        assert!(!UrlProcessor::has_file_extension("/path/file."));
    }

    #[test]
    fn test_validate_url() {
        assert!(UrlProcessor::validate_url("https://placeholder.test").is_ok());
        assert!(UrlProcessor::validate_url("http://placeholder.test").is_ok());
        assert!(UrlProcessor::validate_url("ftp://placeholder.test").is_err());
        assert!(UrlProcessor::validate_url("not-a-url").is_err());
    }

    #[test]
    fn test_validate_urls() {
        let urls = vec![
            "https://placeholder.test/page1".to_string(),
            "invalid-url".to_string(),
            "https://placeholder.test/page2".to_string(),
            "ftp://placeholder.test".to_string(),
        ];

        let (valid, invalid) = UrlProcessor::validate_urls(urls);
        assert_eq!(valid.len(), 2);
        assert_eq!(invalid.len(), 2);
    }

    #[test]
    fn test_extract_path() {
        assert_eq!(
            UrlProcessor::extract_path("https://placeholder.test/path/to/page?query=1").unwrap(),
            "/path/to/page"
        );

        assert_eq!(
            UrlProcessor::extract_path("https://placeholder.test/").unwrap(),
            "/"
        );
    }

    #[test]
    fn test_resolve_relative_url() {
        assert_eq!(
            UrlProcessor::resolve_relative_url("https://placeholder.test/base/", "page.html")
                .unwrap(),
            "https://placeholder.test/base/page.html"
        );

        assert_eq!(
            UrlProcessor::resolve_relative_url(
                "https://placeholder.test/base/page",
                "../other.html"
            )
            .unwrap(),
            "https://placeholder.test/other.html"
        );

        assert_eq!(
            UrlProcessor::resolve_relative_url("https://placeholder.test/base/", "/absolute/path")
                .unwrap(),
            "https://placeholder.test/absolute/path"
        );
    }

    #[test]
    fn test_urls_equal() {
        assert!(UrlProcessor::urls_equal(
            "https://placeholder.test/page",
            "https://placeholder.test/page#section"
        )
        .unwrap());

        assert!(UrlProcessor::urls_equal(
            "HTTPS://EXAMPLE.COM/page",
            "https://placeholder.test/page"
        )
        .unwrap());

        assert!(!UrlProcessor::urls_equal(
            "https://placeholder.test/page1",
            "https://placeholder.test/page2"
        )
        .unwrap());
    }
}
