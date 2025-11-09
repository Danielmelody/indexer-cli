//! IndexNow API Client
//!
//! This module implements a complete client for the IndexNow API protocol.
//! IndexNow allows instant submission of URLs to multiple search engines including
//! Bing, Yandex, Seznam, and Naver through a unified API.
//!
//! # Overview
//!
//! The IndexNow protocol supports two submission methods:
//! - Single URL submission via GET request
//! - Batch URL submission (up to 10,000 URLs) via POST request
//!
//! # Examples
//!
//! ```no_run
//! use indexer_cli::api::indexnow::IndexNowClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create client with API key
//! let client = IndexNowClient::new(
//!     "your-api-key".to_string(),
//!     "https://example.com/your-api-key.txt".to_string(),
//!     vec!["https://api.indexnow.org/indexnow".to_string()],
//! )?;
//!
//! // Submit single URL
//! let response = client.submit_url(
//!     "https://example.com/page1",
//!     "https://api.indexnow.org/indexnow"
//! ).await?;
//!
//! // Submit multiple URLs
//! let urls = vec![
//!     "https://example.com/page1".to_string(),
//!     "https://example.com/page2".to_string(),
//! ];
//! let response = client.submit_urls(&urls, "https://api.indexnow.org/indexnow").await?;
//! # Ok(())
//! # }
//! ```

use crate::constants::{
    INDEXNOW_ENDPOINTS,
    INDEXNOW_KEY_MAX_LENGTH, INDEXNOW_KEY_MIN_LENGTH,
    INDEXNOW_MAX_URLS_PER_REQUEST, USER_AGENT,
};
use crate::types::error::IndexerError;
use crate::utils::retry::{retry_with_backoff, RetryConfig};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info};
use url::Url;

// =============================================================================
// IndexNow API Request/Response Types
// =============================================================================

/// Request body for IndexNow batch submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexNowRequest {
    /// The host/domain name (e.g., "example.com")
    pub host: String,

    /// The API key for authentication
    pub key: String,

    /// URL to the key file on the host
    #[serde(rename = "keyLocation")]
    pub key_location: String,

    /// List of URLs to submit (max 10,000)
    #[serde(rename = "urlList")]
    pub url_list: Vec<String>,
}

/// Response from IndexNow API submission
#[derive(Debug, Clone)]
pub struct IndexNowResponse {
    /// HTTP status code
    pub status_code: u16,

    /// Whether the submission was successful
    pub success: bool,

    /// Response message
    pub message: String,

    /// Endpoint that was used
    pub endpoint: String,
}

impl IndexNowResponse {
    /// Create a new IndexNowResponse
    #[must_use]
    pub fn new(status_code: u16, message: String, endpoint: String) -> Self {
        let success = matches!(status_code, 200 | 202);
        Self {
            status_code,
            success,
            message,
            endpoint,
        }
    }

    /// Check if the submission was successful (200 or 202)
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Check if the key is being verified (202)
    #[must_use]
    pub fn is_pending_verification(&self) -> bool {
        self.status_code == 202
    }
}

// =============================================================================
// IndexNow Client
// =============================================================================

/// Client for interacting with the IndexNow API
///
/// The IndexNowClient handles URL submissions to multiple search engines
/// through the IndexNow protocol. It supports both single and batch submissions,
/// automatic retry logic, and key verification.
pub struct IndexNowClient {
    /// HTTP client for making requests
    client: Client,

    /// API key for authentication (8-128 characters)
    api_key: String,

    /// URL to the key file location on the host
    key_location: String,

    /// List of IndexNow endpoints to submit to
    endpoints: Vec<String>,
}

impl IndexNowClient {
    /// Create a new IndexNowClient
    ///
    /// # Arguments
    ///
    /// * `api_key` - The IndexNow API key (8-128 characters, alphanumeric)
    /// * `key_location` - Full URL to the key file (e.g., "https://example.com/key.txt")
    /// * `endpoints` - List of IndexNow endpoints to use
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The API key is invalid (wrong length or format)
    /// - The key_location URL is invalid
    /// - The HTTP client cannot be created
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use indexer_cli::api::indexnow::IndexNowClient;
    /// let client = IndexNowClient::new(
    ///     "your-32-character-api-key".to_string(),
    ///     "https://example.com/your-32-character-api-key.txt".to_string(),
    ///     vec!["https://api.indexnow.org/indexnow".to_string()],
    /// )?;
    /// # Ok::<(), indexer_cli::types::error::IndexerError>(())
    /// ```
    pub fn new(
        api_key: String,
        key_location: String,
        endpoints: Vec<String>,
    ) -> Result<Self, IndexerError> {
        // Validate API key
        Self::validate_api_key(&api_key)?;

        // Validate key_location URL
        Url::parse(&key_location).map_err(|e| IndexerError::InvalidUrl {
            url: format!("Invalid key_location URL: {}", e),
        })?;

        // Validate endpoints
        if endpoints.is_empty() {
            return Err(IndexerError::ConfigValidationError {
                message: "At least one IndexNow endpoint is required".to_string(),
            });
        }

        // Create HTTP client with timeout
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| IndexerError::HttpRequestFailed {
                message: format!("Failed to create HTTP client: {}", e),
            })?;

        Ok(Self {
            client,
            api_key,
            key_location,
            endpoints,
        })
    }

    /// Create a client with default endpoints
    ///
    /// Uses the standard IndexNow endpoints: api.indexnow.org, bing.com, yandex.com
    pub fn with_default_endpoints(
        api_key: String,
        key_location: String,
    ) -> Result<Self, IndexerError> {
        let endpoints = INDEXNOW_ENDPOINTS
            .iter()
            .map(|&s| s.to_string())
            .collect();
        Self::new(api_key, key_location, endpoints)
    }

    /// Validate API key format and length
    fn validate_api_key(key: &str) -> Result<(), IndexerError> {
        let len = key.len();

        // Check length
        if len < INDEXNOW_KEY_MIN_LENGTH {
            return Err(IndexerError::InvalidApiKey {
                message: format!(
                    "API key too short: {} characters (minimum: {})",
                    len, INDEXNOW_KEY_MIN_LENGTH
                ),
            });
        }

        if len > INDEXNOW_KEY_MAX_LENGTH {
            return Err(IndexerError::InvalidApiKey {
                message: format!(
                    "API key too long: {} characters (maximum: {})",
                    len, INDEXNOW_KEY_MAX_LENGTH
                ),
            });
        }

        // Check characters (alphanumeric and hyphens only)
        if !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(IndexerError::InvalidApiKey {
                message: "API key must contain only alphanumeric characters and hyphens"
                    .to_string(),
            });
        }

        Ok(())
    }

    /// Submit a single URL to a specific search engine endpoint
    ///
    /// Uses the GET method for single URL submission:
    /// `GET https://api.indexnow.org/indexnow?url={url}&key={key}`
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to submit
    /// * `endpoint` - The IndexNow endpoint to use
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The URL is invalid
    /// - The HTTP request fails
    /// - The API returns an error status code
    pub async fn submit_url(
        &self,
        url: &str,
        endpoint: &str,
    ) -> Result<IndexNowResponse, IndexerError> {
        info!("Submitting single URL to IndexNow: {}", url);

        // Validate URL
        let parsed_url = Url::parse(url).map_err(|_| IndexerError::InvalidUrl {
            url: url.to_string(),
        })?;

        // Build request URL with query parameters
        let request_url = format!("{}?url={}&key={}", endpoint, parsed_url, self.api_key);

        debug!("IndexNow GET request to: {}", endpoint);

        // Execute request with retry
        let retry_config = RetryConfig {
            max_retries: 3,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            jitter: true,
        };

        let client = &self.client;
        let response = retry_with_backoff(
            retry_config,
            || async {
                client
                    .get(&request_url)
                    .send()
                    .await
                    .map_err(|e| IndexerError::HttpRequestFailed {
                        message: e.to_string(),
                    })
            },
        )
        .await?;

        let status = response.status();
        let status_code = status.as_u16();

        debug!("IndexNow response status: {}", status_code);

        // Handle response based on status code
        match status_code {
            200 => {
                info!("URL submitted successfully: {}", url);
                Ok(IndexNowResponse::new(
                    status_code,
                    "URL submitted successfully".to_string(),
                    endpoint.to_string(),
                ))
            }
            202 => {
                info!("URL accepted, key verification pending: {}", url);
                Ok(IndexNowResponse::new(
                    status_code,
                    "URL accepted, key verification in progress".to_string(),
                    endpoint.to_string(),
                ))
            }
            400 => {
                let body = response.text().await.map_err(|e| {
                    IndexerError::HttpRequestFailed {
                        message: format!("Failed to read response body: {}", e),
                    }
                })?;
                Err(IndexerError::IndexNowBadRequest {
                    message: format!("Invalid request format: {}", body),
                })
            }
            403 => Err(IndexerError::IndexNowInvalidKey),
            422 => {
                let body = response.text().await.map_err(|e| {
                    IndexerError::HttpRequestFailed {
                        message: format!("Failed to read response body: {}", e),
                    }
                })?;
                Err(IndexerError::IndexNowUnprocessableEntity {
                    message: format!(
                        "URL does not belong to host or key mismatch: {}",
                        body
                    ),
                })
            }
            429 => Err(IndexerError::IndexNowRateLimitExceeded),
            _ => {
                let body = response.text().await.map_err(|e| {
                    IndexerError::HttpRequestFailed {
                        message: format!("Failed to read response body: {}", e),
                    }
                })?;
                Err(IndexerError::IndexNowApiError {
                    status_code,
                    message: body,
                })
            }
        }
    }

    /// Submit multiple URLs to a specific search engine endpoint
    ///
    /// Uses the POST method with JSON body for batch submission.
    /// Maximum 10,000 URLs per request.
    ///
    /// # Arguments
    ///
    /// * `urls` - List of URLs to submit (max 10,000)
    /// * `endpoint` - The IndexNow endpoint to use
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The URL list is empty or exceeds 10,000 URLs
    /// - Any URL is invalid
    /// - The HTTP request fails
    /// - The API returns an error status code
    pub async fn submit_urls(
        &self,
        urls: &[String],
        endpoint: &str,
    ) -> Result<IndexNowResponse, IndexerError> {
        if urls.is_empty() {
            return Err(IndexerError::BatchProcessingFailed {
                successful: 0,
                failed: 0,
            });
        }

        if urls.len() > INDEXNOW_MAX_URLS_PER_REQUEST {
            return Err(IndexerError::BatchSizeExceedsLimit {
                size: urls.len(),
                limit: INDEXNOW_MAX_URLS_PER_REQUEST,
            });
        }

        info!(
            "Submitting {} URLs to IndexNow endpoint: {}",
            urls.len(),
            endpoint
        );

        // Extract host from first URL
        let first_url = Url::parse(&urls[0]).map_err(|_| IndexerError::InvalidUrl {
            url: urls[0].clone(),
        })?;

        let host = first_url
            .host_str()
            .ok_or_else(|| IndexerError::InvalidUrl {
                url: urls[0].clone(),
            })?
            .to_string();

        // Build request body
        let request_body = IndexNowRequest {
            host,
            key: self.api_key.clone(),
            key_location: self.key_location.clone(),
            url_list: urls.to_vec(),
        };

        debug!("IndexNow POST request to: {}", endpoint);

        // Execute request with retry
        let retry_config = RetryConfig {
            max_retries: 3,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            jitter: true,
        };

        let client = &self.client;
        let response = retry_with_backoff(
            retry_config,
            || async {
                client
                    .post(endpoint)
                    .json(&request_body)
                    .send()
                    .await
                    .map_err(|e| IndexerError::HttpRequestFailed {
                        message: e.to_string(),
                    })
            },
        )
        .await?;

        let status = response.status();
        let status_code = status.as_u16();

        debug!("IndexNow response status: {}", status_code);

        // Handle response based on status code
        match status_code {
            200 => {
                info!("Batch submitted successfully: {} URLs", urls.len());
                Ok(IndexNowResponse::new(
                    status_code,
                    format!("{} URLs submitted successfully", urls.len()),
                    endpoint.to_string(),
                ))
            }
            202 => {
                info!(
                    "Batch accepted, key verification pending: {} URLs",
                    urls.len()
                );
                Ok(IndexNowResponse::new(
                    status_code,
                    format!(
                        "{} URLs accepted, key verification in progress",
                        urls.len()
                    ),
                    endpoint.to_string(),
                ))
            }
            400 => {
                let body = response.text().await.map_err(|e| {
                    IndexerError::HttpRequestFailed {
                        message: format!("Failed to read response body: {}", e),
                    }
                })?;
                Err(IndexerError::IndexNowBadRequest {
                    message: format!("Invalid request format: {}", body),
                })
            }
            403 => Err(IndexerError::IndexNowInvalidKey),
            422 => {
                let body = response.text().await.map_err(|e| {
                    IndexerError::HttpRequestFailed {
                        message: format!("Failed to read response body: {}", e),
                    }
                })?;
                Err(IndexerError::IndexNowUnprocessableEntity {
                    message: format!(
                        "URLs do not belong to host or key mismatch: {}",
                        body
                    ),
                })
            }
            429 => Err(IndexerError::IndexNowRateLimitExceeded),
            _ => {
                let body = response.text().await.map_err(|e| {
                    IndexerError::HttpRequestFailed {
                        message: format!("Failed to read response body: {}", e),
                    }
                })?;
                Err(IndexerError::IndexNowApiError {
                    status_code,
                    message: body,
                })
            }
        }
    }

    /// Submit URLs to all configured search engine endpoints
    ///
    /// Submits the same URLs to all endpoints concurrently.
    /// Returns results for all endpoints, including any failures.
    ///
    /// # Arguments
    ///
    /// * `urls` - List of URLs to submit
    ///
    /// # Returns
    ///
    /// A vector of results, one for each endpoint. Each result contains
    /// either the successful response or an error.
    pub async fn submit_to_all(
        &self,
        urls: &[String],
    ) -> Vec<Result<IndexNowResponse, IndexerError>> {
        info!(
            "Submitting {} URLs to all {} endpoints",
            urls.len(),
            self.endpoints.len()
        );

        let mut tasks = Vec::new();

        for endpoint in &self.endpoints {
            let endpoint = endpoint.clone();
            let urls = urls.to_vec();
            let client = self.clone_for_concurrent();

            let task = tokio::spawn(async move {
                if urls.len() == 1 {
                    client.submit_url(&urls[0], &endpoint).await
                } else {
                    client.submit_urls(&urls, &endpoint).await
                }
            });

            tasks.push(task);
        }

        // Wait for all tasks to complete
        let mut results = Vec::new();
        for task in tasks {
            match task.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(Err(IndexerError::InternalError {
                    message: format!("Task join error: {}", e),
                })),
            }
        }

        results
    }

    /// Clone the client for concurrent usage
    fn clone_for_concurrent(&self) -> Self {
        Self {
            client: self.client.clone(),
            api_key: self.api_key.clone(),
            key_location: self.key_location.clone(),
            endpoints: self.endpoints.clone(),
        }
    }

    /// Verify that the key file is accessible at the specified location
    ///
    /// Checks that:
    /// 1. The key file URL is accessible (HTTP 200)
    /// 2. The file content matches the API key
    ///
    /// # Arguments
    ///
    /// * `host` - The host/domain to check (e.g., "example.com")
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The key file is not accessible
    /// - The file content does not match the API key
    pub async fn verify_key_file(&self, host: &str) -> Result<(), IndexerError> {
        info!("Verifying IndexNow key file for host: {}", host);

        // Build key file URL
        let key_file_url = format!("https://{}/{}.txt", host, self.api_key);

        debug!("Checking key file at: {}", key_file_url);

        // Fetch key file
        let response = self
            .client
            .get(&key_file_url)
            .send()
            .await
            .map_err(|e| IndexerError::IndexNowKeyFileNotAccessible {
                url: key_file_url.clone(),
                message: e.to_string(),
            })?;

        if !response.status().is_success() {
            return Err(IndexerError::IndexNowKeyFileNotAccessible {
                url: key_file_url,
                message: format!("HTTP {}", response.status()),
            });
        }

        // Read content
        let content = response
            .text()
            .await
            .map_err(|e| IndexerError::IndexNowKeyFileNotAccessible {
                url: key_file_url.clone(),
                message: format!("Failed to read response: {}", e),
            })?;

        // Verify content matches API key
        let content = content.trim();
        if content != self.api_key {
            return Err(IndexerError::IndexNowKeyFileMismatch {
                expected: self.api_key.clone(),
                actual: content.to_string(),
            });
        }

        info!("Key file verified successfully");
        Ok(())
    }

    /// Generate a new IndexNow API key
    ///
    /// Generates a random hexadecimal string of the specified length.
    ///
    /// # Arguments
    ///
    /// * `length` - Length of the key (8-128, recommended: 32)
    ///
    /// # Errors
    ///
    /// Returns an error if the length is outside the valid range.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indexer_cli::api::indexnow::IndexNowClient;
    /// let key = IndexNowClient::generate_key(32)?;
    /// assert_eq!(key.len(), 32);
    /// # Ok::<(), indexer_cli::types::error::IndexerError>(())
    /// ```
    pub fn generate_key(length: usize) -> Result<String, IndexerError> {
        if !(INDEXNOW_KEY_MIN_LENGTH..=INDEXNOW_KEY_MAX_LENGTH).contains(&length) {
            return Err(IndexerError::ValueOutOfRange {
                field: "key_length".to_string(),
                value: length.to_string(),
                min: INDEXNOW_KEY_MIN_LENGTH.to_string(),
                max: INDEXNOW_KEY_MAX_LENGTH.to_string(),
            });
        }

        use rand::Rng;

        const CHARSET: &[u8] = b"0123456789abcdef";
        let mut rng = rand::rng();

        let key: String = (0..length)
            .map(|_| {
                let idx = rng.random_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        info!("Generated new IndexNow API key (length: {})", length);
        Ok(key)
    }

    /// Get the configured API key
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Get the key file location URL
    pub fn key_location(&self) -> &str {
        &self.key_location
    }

    /// Get the list of configured endpoints
    pub fn endpoints(&self) -> &[String] {
        &self.endpoints
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_api_key() {
        // Valid keys
        assert!(IndexNowClient::validate_api_key("12345678").is_ok());
        assert!(IndexNowClient::validate_api_key("a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6").is_ok());
        assert!(IndexNowClient::validate_api_key("key-with-hyphens-123").is_ok());

        // Too short
        assert!(IndexNowClient::validate_api_key("1234567").is_err());

        // Too long
        let long_key = "a".repeat(129);
        assert!(IndexNowClient::validate_api_key(&long_key).is_err());

        // Invalid characters
        assert!(IndexNowClient::validate_api_key("key_with_underscore").is_err());
        assert!(IndexNowClient::validate_api_key("key with spaces").is_err());
        assert!(IndexNowClient::validate_api_key("key@special!chars").is_err());
    }

    #[test]
    fn test_generate_key() {
        // Valid lengths
        assert!(IndexNowClient::generate_key(8).is_ok());
        assert!(IndexNowClient::generate_key(32).is_ok());
        assert!(IndexNowClient::generate_key(128).is_ok());

        // Check generated key length
        let key = IndexNowClient::generate_key(32).unwrap();
        assert_eq!(key.len(), 32);

        // Check generated key contains only hex characters
        assert!(key.chars().all(|c| c.is_ascii_hexdigit()));

        // Invalid lengths
        assert!(IndexNowClient::generate_key(7).is_err());
        assert!(IndexNowClient::generate_key(129).is_err());
    }

    #[test]
    fn test_indexnow_response() {
        let response = IndexNowResponse::new(
            200,
            "Success".to_string(),
            "https://api.indexnow.org/indexnow".to_string(),
        );
        assert!(response.is_success());
        assert!(!response.is_pending_verification());

        let response = IndexNowResponse::new(
            202,
            "Pending".to_string(),
            "https://api.indexnow.org/indexnow".to_string(),
        );
        assert!(response.is_success());
        assert!(response.is_pending_verification());

        let response = IndexNowResponse::new(
            400,
            "Bad Request".to_string(),
            "https://api.indexnow.org/indexnow".to_string(),
        );
        assert!(!response.is_success());
        assert!(!response.is_pending_verification());
    }

    #[tokio::test]
    async fn test_create_client() {
        let result = IndexNowClient::new(
            "a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6".to_string(),
            "https://example.com/a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6.txt".to_string(),
            vec!["https://api.indexnow.org/indexnow".to_string()],
        );
        assert!(result.is_ok());

        // Invalid key
        let result = IndexNowClient::new(
            "short".to_string(),
            "https://example.com/key.txt".to_string(),
            vec!["https://api.indexnow.org/indexnow".to_string()],
        );
        assert!(result.is_err());

        // Invalid key_location
        let result = IndexNowClient::new(
            "a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6".to_string(),
            "not-a-url".to_string(),
            vec!["https://api.indexnow.org/indexnow".to_string()],
        );
        assert!(result.is_err());

        // Empty endpoints
        let result = IndexNowClient::new(
            "a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6".to_string(),
            "https://example.com/key.txt".to_string(),
            vec![],
        );
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_with_default_endpoints() {
        let result = IndexNowClient::with_default_endpoints(
            "a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6".to_string(),
            "https://example.com/a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6.txt".to_string(),
        );
        assert!(result.is_ok());

        let client = result.unwrap();
        assert_eq!(client.endpoints().len(), INDEXNOW_ENDPOINTS.len());
    }
}
