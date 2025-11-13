//! Google Indexing API client implementation
//!
//! This module provides a comprehensive client for the Google Indexing API v3,
//! including service-account authentication, URL submission, batch operations, and quota management.
use crate::types::error::IndexerError;
use crate::utils::retry::{retry_with_condition, RetryConfig};
use chrono::{DateTime, Utc};
use hyper_util::client::legacy::connect::HttpConnector;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
use yup_oauth2::{
    authenticator::Authenticator, hyper_rustls::HttpsConnector, ServiceAccountAuthenticator,
};

/// Google Indexing API endpoint
const GOOGLE_INDEXING_API_ENDPOINT: &str = "https://indexing.googleapis.com/v3";

/// Google Search Console API endpoint (webmasters v3)
/// Note: `/webmasters/v3` is required; `/v1` returns 404.
const GOOGLE_SEARCH_CONSOLE_API_ENDPOINT: &str =
    "https://searchconsole.googleapis.com/webmasters/v3";

/// Google Indexing API scope
const GOOGLE_INDEXING_SCOPE: &str = "https://www.googleapis.com/auth/indexing";

/// Google Search Console API scope (read-only)
const GOOGLE_SEARCH_CONSOLE_SCOPE: &str = "https://www.googleapis.com/auth/webmasters.readonly";

/// Default batch size for batch operations
const DEFAULT_BATCH_SIZE: usize = 100;

/// Maximum URLs per batch request
const MAX_BATCH_SIZE: usize = 100;

/// Default quota limits
const DEFAULT_DAILY_PUBLISH_LIMIT: usize = 200;
const DEFAULT_RATE_LIMIT_PER_MINUTE: usize = 380;
const DEFAULT_METADATA_RATE_LIMIT_PER_MINUTE: usize = 180;

/// Notification type for URL updates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationType {
    /// URL has been updated or added
    #[serde(rename = "URL_UPDATED")]
    UrlUpdated,
    /// URL has been deleted
    #[serde(rename = "URL_DELETED")]
    UrlDeleted,
}

impl std::fmt::Display for NotificationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationType::UrlUpdated => write!(f, "URL_UPDATED"),
            NotificationType::UrlDeleted => write!(f, "URL_DELETED"),
        }
    }
}

/// URL notification request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UrlNotification {
    /// The URL to notify Google about
    url: String,
    /// The type of notification
    #[serde(rename = "type")]
    notification_type: String,
}

/// Latest update information
#[derive(Debug, Clone, Deserialize)]
pub struct LatestUpdate {
    /// The URL that was updated
    pub url: String,
    /// The type of notification
    #[serde(rename = "type")]
    pub notification_type: String,
    /// The time when the notification was sent
    #[serde(rename = "notifyTime")]
    pub notify_time: Option<String>,
}

/// URL notification metadata
#[derive(Debug, Clone, Deserialize)]
pub struct UrlNotificationMetadata {
    /// The URL
    pub url: String,
    /// Latest update information
    #[serde(rename = "latestUpdate")]
    pub latest_update: Option<LatestUpdate>,
    /// Latest remove information (for deleted URLs)
    #[serde(rename = "latestRemove")]
    pub latest_remove: Option<LatestUpdate>,
}

/// URL notification response
#[derive(Debug, Clone, Deserialize)]
pub struct UrlNotificationResponse {
    /// URL notification metadata
    #[serde(rename = "urlNotificationMetadata")]
    pub url_notification_metadata: UrlNotificationMetadata,
}

/// Metadata response for get_metadata operation
#[derive(Debug, Clone, Deserialize)]
pub struct MetadataResponse {
    /// URL notification metadata
    #[serde(rename = "urlNotificationMetadata")]
    pub url_notification_metadata: UrlNotificationMetadata,
}

/// Result of a URL submission
#[derive(Debug, Clone)]
pub struct SubmissionResult {
    /// The URL that was submitted
    pub url: String,
    /// Whether the submission was successful
    pub success: bool,
    /// HTTP status code
    pub status_code: Option<u16>,
    /// Response message
    pub message: String,
    /// Timestamp of the submission
    pub submitted_at: DateTime<Utc>,
}

/// Batch submission result
#[derive(Debug, Clone)]
pub struct BatchSubmissionResult {
    /// Total number of URLs processed
    pub total: usize,
    /// Number of successful submissions
    pub successful: usize,
    /// Number of failed submissions
    pub failed: usize,
    /// Individual results
    pub results: Vec<SubmissionResult>,
}

/// Rate limiter for API requests
#[derive(Debug)]
struct RateLimiter {
    /// Maximum requests per minute
    max_requests_per_minute: usize,
    /// Request timestamps
    request_times: Vec<DateTime<Utc>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    fn new(max_requests_per_minute: usize) -> Self {
        Self {
            max_requests_per_minute,
            request_times: Vec::new(),
        }
    }

    /// Wait if necessary to respect rate limits
    async fn wait_if_needed(&mut self) {
        let now = Utc::now();
        let one_minute_ago = now - chrono::Duration::minutes(1);

        // Remove old request times
        self.request_times.retain(|&time| time > one_minute_ago);

        // If we've hit the limit, wait until the oldest request expires
        if self.request_times.len() >= self.max_requests_per_minute {
            if let Some(&oldest) = self.request_times.first() {
                let wait_until = oldest + chrono::Duration::minutes(1);
                let wait_duration = (wait_until - now)
                    .to_std()
                    .unwrap_or(Duration::from_secs(1));

                if wait_duration > Duration::ZERO {
                    warn!(
                        "Rate limit reached, waiting {:?} before next request",
                        wait_duration
                    );
                    tokio::time::sleep(wait_duration).await;
                }
            }
        }

        // Record this request
        self.request_times.push(Utc::now());
    }
}

/// Google Indexing API client
pub struct GoogleIndexingClient {
    /// HTTP client
    client: reqwest::Client,
    /// Service account authenticator
    authenticator: Arc<Mutex<Authenticator<HttpsConnector<HttpConnector>>>>,
    /// Rate limiter
    rate_limiter: Arc<Mutex<RateLimiter>>,
    /// Daily publish limit
    daily_publish_limit: usize,
    /// Batch size for batch operations
    batch_size: usize,
}

impl GoogleIndexingClient {
    /// Create a new Google Indexing API client from service account file (legacy)
    ///
    /// # Arguments
    ///
    /// * `service_account_path` - Path to the Google Service Account JSON key file
    ///
    /// # Returns
    ///
    /// Returns a Result containing the client or an error
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The service account file does not exist
    /// - The service account file is invalid or malformed
    /// - The HTTP client cannot be created
    /// - Authentication with Google OAuth2 fails
    pub async fn from_service_account(service_account_path: PathBuf) -> Result<Self, IndexerError> {
        // Validate service account file exists
        if !service_account_path.exists() {
            return Err(IndexerError::GoogleServiceAccountNotFound {
                path: service_account_path,
            });
        }

        info!("Initializing Google Indexing API client with service account");
        debug!("Service account path: {:?}", service_account_path);

        // Create HTTP client
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| IndexerError::HttpRequestFailed {
                message: e.to_string(),
            })?;

        // Create authenticator
        let auth = Self::create_service_account_authenticator(&service_account_path).await?;

        Ok(Self {
            client,
            authenticator: Arc::new(Mutex::new(auth)),
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new(DEFAULT_RATE_LIMIT_PER_MINUTE))),
            daily_publish_limit: DEFAULT_DAILY_PUBLISH_LIMIT,
            batch_size: DEFAULT_BATCH_SIZE,
        })
    }

    /// Create a new Google Indexing API client with custom configuration (service account)
    ///
    /// # Arguments
    ///
    /// * `service_account_path` - Path to the Google Service Account JSON key file
    /// * `daily_publish_limit` - Daily publish limit
    /// * `rate_limit_per_minute` - Rate limit per minute
    /// * `batch_size` - Batch size for batch operations
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The service account file does not exist
    /// - The service account file is invalid or malformed
    /// - The HTTP client cannot be created
    /// - Authentication with Google OAuth2 fails
    pub async fn from_service_account_with_config(
        service_account_path: PathBuf,
        daily_publish_limit: usize,
        rate_limit_per_minute: usize,
        batch_size: usize,
    ) -> Result<Self, IndexerError> {
        let mut client = Self::from_service_account(service_account_path).await?;
        client.daily_publish_limit = daily_publish_limit;
        client.rate_limiter = Arc::new(Mutex::new(RateLimiter::new(rate_limit_per_minute)));
        client.batch_size = batch_size.min(MAX_BATCH_SIZE);
        Ok(client)
    }

    /// Create OAuth2 authenticator from service account file
    async fn create_service_account_authenticator(
        service_account_path: &PathBuf,
    ) -> Result<Authenticator<HttpsConnector<HttpConnector>>, IndexerError> {
        debug!(
            "Reading service account key from: {:?}",
            service_account_path
        );

        // Read service account key
        let service_account_key = yup_oauth2::read_service_account_key(&service_account_path)
            .await
            .map_err(|e| {
                let error_msg = e.to_string();
                error!("Failed to read service account key: {}", error_msg);

                // Check if it's a PEM parsing error
                if error_msg.contains("Not enough private keys in PEM")
                    || error_msg.contains("private_key")
                    || error_msg.contains("key")
                {
                    IndexerError::GoogleServiceAccountInvalid {
                        message: format!(
                            "Invalid service account JSON: {}. \
                            Ensure the file contains a valid 'private_key' field. \
                            Download the key as JSON (not P12) from Google Cloud Console.",
                            error_msg
                        ),
                    }
                } else {
                    IndexerError::GoogleServiceAccountInvalid { message: error_msg }
                }
            })?;

        // Validate that the key has required fields
        if service_account_key.client_email.is_empty() {
            error!("Service account key is missing client_email");
            return Err(IndexerError::GoogleServiceAccountInvalid {
                message: "Service account JSON is missing 'client_email' field".to_string(),
            });
        }

        debug!(
            "Service account email: {}",
            service_account_key.client_email
        );
        debug!("Project ID: {:?}", service_account_key.project_id);

        // Create authenticator
        let auth = ServiceAccountAuthenticator::builder(service_account_key)
            .build()
            .await
            .map_err(|e| {
                let error_msg = e.to_string();
                error!("Failed to create authenticator: {}", error_msg);

                IndexerError::GoogleAuthError {
                    message: format!(
                        "Failed to create authenticator from service account: {}. \
                        This usually means the private key is invalid or missing.",
                        error_msg
                    ),
                }
            })?;

        info!("Successfully created service account authenticator");
        Ok(auth)
    }

    /// Authenticate and get access token
    ///
    /// # Returns
    ///
    /// Returns the access token or an error
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The OAuth2 token request fails
    /// - No access token is returned by the authentication service
    pub async fn authenticate(&self) -> Result<String, IndexerError> {
        debug!("Authenticating with Google OAuth2 (service account)");
        self.get_access_token_for_scope(GOOGLE_INDEXING_SCOPE).await
    }

    async fn get_access_token_for_scope(&self, scope: &str) -> Result<String, IndexerError> {
        let auth = self.authenticator.lock().await;
        let token = auth
            .token(&[scope])
            .await
            .map_err(|e| IndexerError::GoogleAuthError {
                message: e.to_string(),
            })?;

        token
            .token()
            .ok_or_else(|| IndexerError::GoogleAuthError {
                message: "No access token returned".to_string(),
            })
            .map(|token| token.to_string())
    }

    /// Submit a single URL to Google Indexing API
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to submit
    /// * `notification_type` - The type of notification (URL_UPDATED or URL_DELETED)
    ///
    /// # Returns
    ///
    /// Returns the submission result or an error
    ///
    /// # Errors
    ///
    /// This function converts errors into unsuccessful submission results instead of
    /// propagating them. It returns `Ok(SubmissionResult)` with `success: false` if:
    /// - Authentication fails
    /// - The HTTP request fails
    /// - The API returns an error response
    /// - The request times out or is retried unsuccessfully
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use indexer_cli::api::google_indexing::{GoogleIndexingClient, NotificationType};
    /// use std::path::PathBuf;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = GoogleIndexingClient::new(
    ///         PathBuf::from("/path/to/service-account.json")
    ///     ).await?;
    ///
    ///     let result = client.publish_url(
    ///         "https://placeholder.test/page1",
    ///         NotificationType::UrlUpdated
    ///     ).await?;
    ///
    ///     println!("Submitted: {}", result.url);
    ///     Ok(())
    /// }
    /// ```
    pub async fn publish_url(
        &self,
        url: &str,
        notification_type: NotificationType,
    ) -> Result<SubmissionResult, IndexerError> {
        info!("Publishing URL: {} ({})", url, notification_type);

        // Wait for rate limiter
        self.rate_limiter.lock().await.wait_if_needed().await;

        // Prepare request
        let notification = UrlNotification {
            url: url.to_string(),
            notification_type: notification_type.to_string(),
        };

        // Get access token
        let access_token = self.authenticate().await?;

        // Build request URL
        let request_url = format!("{GOOGLE_INDEXING_API_ENDPOINT}/urlNotifications:publish");

        // Make request with retry logic
        let retry_config = RetryConfig::new()
            .with_max_retries(3)
            .with_initial_backoff(Duration::from_millis(500))
            .with_max_backoff(Duration::from_secs(10));

        let result = retry_with_condition(
            retry_config,
            |err: &IndexerError| err.is_retryable(),
            || async {
                let response = self
                    .client
                    .post(&request_url)
                    .header("Authorization", format!("Bearer {access_token}"))
                    .header("Content-Type", "application/json")
                    .json(&notification)
                    .send()
                    .await
                    .map_err(|e| IndexerError::HttpRequestFailed {
                        message: e.to_string(),
                    })?;

                let status_code = response.status();
                let status_code_u16 = status_code.as_u16();

                // Handle response
                match status_code {
                    StatusCode::OK => {
                        let _response_body: UrlNotificationResponse = response
                            .json()
                            .await
                            .map_err(|e| IndexerError::JsonDeserializationError {
                                message: e.to_string(),
                            })?;

                        debug!("Successfully published URL: {}", url);

                        Ok(SubmissionResult {
                            url: url.to_string(),
                            success: true,
                            status_code: Some(status_code_u16),
                            message: "Successfully submitted".to_string(),
                            submitted_at: Utc::now(),
                        })
                    }
                    StatusCode::BAD_REQUEST => {
                        let error_text = response.text().await.unwrap_or_default();
                        Err(IndexerError::GoogleInvalidRequest {
                            message: error_text,
                        })
                    }
                    StatusCode::FORBIDDEN => {
                        let error_text = response.text().await.unwrap_or_default();
                        Err(IndexerError::GooglePermissionDenied {
                            message: error_text,
                        })
                    }
                    StatusCode::TOO_MANY_REQUESTS => Err(IndexerError::GoogleRateLimitExceeded),
                    StatusCode::INTERNAL_SERVER_ERROR
                    | StatusCode::BAD_GATEWAY
                    | StatusCode::SERVICE_UNAVAILABLE
                    | StatusCode::GATEWAY_TIMEOUT => {
                        let error_text = response.text().await.unwrap_or_default();
                        Err(IndexerError::GoogleApiError {
                            status_code: status_code_u16,
                            message: format!("Server error: {error_text}"),
                        })
                    }
                    _ => {
                        let error_text = response.text().await.unwrap_or_default();
                        Err(IndexerError::GoogleApiError {
                            status_code: status_code_u16,
                            message: error_text,
                        })
                    }
                }
            },
        )
        .await;

        result.or_else(|e| {
            error!("Failed to publish URL {}: {}", url, e);
            Ok(SubmissionResult {
                url: url.to_string(),
                success: false,
                status_code: None,
                message: e.to_string(),
                submitted_at: Utc::now(),
            })
        })
    }

    /// Submit multiple URLs in batches
    ///
    /// # Arguments
    ///
    /// * `urls` - The URLs to submit
    /// * `notification_type` - The type of notification (URL_UPDATED or URL_DELETED)
    ///
    /// # Returns
    ///
    /// Returns the batch submission result
    ///
    /// # Errors
    ///
    /// This function does not return errors directly. Individual URL submission failures
    /// are captured in the `BatchSubmissionResult` with detailed error information for
    /// each failed URL. The function only returns `Err` if there's a catastrophic failure
    /// before any submissions can begin.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use indexer_cli::api::google_indexing::{GoogleIndexingClient, NotificationType};
    /// use std::path::PathBuf;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = GoogleIndexingClient::new(
    ///         PathBuf::from("/path/to/service-account.json")
    ///     ).await?;
    ///
    ///     let urls = vec![
    ///         "https://placeholder.test/page1".to_string(),
    ///         "https://placeholder.test/page2".to_string(),
    ///     ];
    ///
    ///     let result = client.batch_publish_urls(urls, NotificationType::UrlUpdated).await?;
    ///     println!("Success: {}/{}", result.successful, result.total);
    ///     Ok(())
    /// }
    /// ```
    pub async fn batch_publish_urls(
        &self,
        urls: Vec<String>,
        notification_type: NotificationType,
    ) -> Result<BatchSubmissionResult, IndexerError> {
        let total = urls.len();
        info!(
            "Starting batch submission of {} URLs ({})",
            total, notification_type
        );

        let mut results = Vec::new();
        let mut successful = 0;
        let mut failed = 0;

        // Split URLs into batches
        for (batch_idx, batch) in urls.chunks(self.batch_size).enumerate() {
            info!(
                "Processing batch {}/{} ({} URLs)",
                batch_idx + 1,
                total.div_ceil(self.batch_size),
                batch.len()
            );

            // Submit each URL in the batch
            for url in batch {
                match self.publish_url(url, notification_type).await {
                    Ok(result) => {
                        if result.success {
                            successful += 1;
                        } else {
                            failed += 1;
                        }
                        results.push(result);
                    }
                    Err(e) => {
                        error!("Failed to submit URL {}: {}", url, e);
                        failed += 1;
                        results.push(SubmissionResult {
                            url: url.clone(),
                            success: false,
                            status_code: None,
                            message: e.to_string(),
                            submitted_at: Utc::now(),
                        });
                    }
                }
            }

            // Add a small delay between batches to avoid overwhelming the API
            if batch_idx < total.div_ceil(self.batch_size) - 1 {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }

        info!(
            "Batch submission completed: {}/{} successful",
            successful, total
        );

        Ok(BatchSubmissionResult {
            total,
            successful,
            failed,
            results,
        })
    }

    /// Get metadata for a URL
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to get metadata for
    ///
    /// # Returns
    ///
    /// Returns the metadata response or an error
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Authentication fails
    /// - The HTTP request fails
    /// - The URL is not found in the indexing database
    /// - Permission is denied for the URL
    /// - The API returns any other error response
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use indexer_cli::api::google_indexing::GoogleIndexingClient;
    /// use std::path::PathBuf;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = GoogleIndexingClient::new(
    ///         PathBuf::from("/path/to/service-account.json")
    ///     ).await?;
    ///
    ///     let metadata = client.get_metadata("https://placeholder.test/page1").await?;
    ///     println!("Metadata: {:?}", metadata);
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_metadata(&self, url: &str) -> Result<MetadataResponse, IndexerError> {
        info!("Getting metadata for URL: {}", url);

        // Wait for rate limiter
        self.rate_limiter.lock().await.wait_if_needed().await;

        // Get access token
        let access_token = self.authenticate().await?;

        // Build request URL
        let encoded_url =
            percent_encoding::utf8_percent_encode(url, percent_encoding::NON_ALPHANUMERIC)
                .to_string();
        let request_url = format!(
            "{}/urlNotifications/metadata?url={}",
            GOOGLE_INDEXING_API_ENDPOINT, encoded_url
        );

        // Make request
        let response = self
            .client
            .get(&request_url)
            .header("Authorization", format!("Bearer {access_token}"))
            .send()
            .await
            .map_err(|e| IndexerError::HttpRequestFailed {
                message: e.to_string(),
            })?;

        let status_code = response.status();

        match status_code {
            StatusCode::OK => {
                let metadata: MetadataResponse =
                    response
                        .json()
                        .await
                        .map_err(|e| IndexerError::JsonDeserializationError {
                            message: e.to_string(),
                        })?;
                debug!("Successfully retrieved metadata for URL: {}", url);
                Ok(metadata)
            }
            StatusCode::NOT_FOUND => Err(IndexerError::GoogleApiError {
                status_code: status_code.as_u16(),
                message: "URL not found in indexing database".to_string(),
            }),
            StatusCode::FORBIDDEN => {
                let error_text = response.text().await.unwrap_or_default();
                Err(IndexerError::GooglePermissionDenied {
                    message: error_text,
                })
            }
            _ => {
                let error_text = response.text().await.unwrap_or_default();
                Err(IndexerError::GoogleApiError {
                    status_code: status_code.as_u16(),
                    message: error_text,
                })
            }
        }
    }

    /// Check API quota usage
    ///
    /// This is a placeholder implementation as Google doesn't provide a direct quota API.
    /// In a real implementation, you would track quota usage locally.
    ///
    /// # Returns
    ///
    /// Returns quota information
    ///
    /// # Errors
    ///
    /// This function currently never returns an error, but the signature supports
    /// potential future implementations that might query external quota tracking systems.
    pub async fn check_quota(&self) -> Result<QuotaInfo, IndexerError> {
        info!("Checking quota (local tracking)");

        // This is a simplified implementation
        // In a production system, you would track this in a database
        Ok(QuotaInfo {
            daily_publish_limit: self.daily_publish_limit,
            daily_publish_used: 0, // Would be tracked in database
            rate_limit_per_minute: DEFAULT_RATE_LIMIT_PER_MINUTE,
            metadata_rate_limit_per_minute: DEFAULT_METADATA_RATE_LIMIT_PER_MINUTE,
        })
    }

    /// Get list of sites from Google Search Console
    ///
    /// # Returns
    ///
    /// Returns a list of sites the authenticated user/service account has access to
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Authentication fails
    /// - The HTTP request fails
    /// - The API returns an error response
    pub async fn list_search_console_sites(&self) -> Result<Vec<SiteEntry>, IndexerError> {
        info!("Fetching Search Console sites list");

        // Get access token with Search Console scope
        let access_token = self
            .get_access_token_for_scope(GOOGLE_SEARCH_CONSOLE_SCOPE)
            .await?;

        // Build request URL
        let request_url = format!("{}/sites", GOOGLE_SEARCH_CONSOLE_API_ENDPOINT);

        // Make request
        let response = self
            .client
            .get(&request_url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| IndexerError::HttpRequestFailed {
                message: e.to_string(),
            })?;

        let status_code = response.status();

        match status_code {
            StatusCode::OK => {
                let sites_list: SitesListResponse =
                    response
                        .json()
                        .await
                        .map_err(|e| IndexerError::JsonDeserializationError {
                            message: e.to_string(),
                        })?;
                debug!(
                    "Successfully retrieved {} sites",
                    sites_list.site_entry.len()
                );
                Ok(sites_list.site_entry)
            }
            StatusCode::FORBIDDEN => {
                let error_text = response.text().await.unwrap_or_default();
                Err(IndexerError::GooglePermissionDenied {
                    message: format!("Cannot access Search Console API. Make sure the service account has Search Console permissions. Error: {}", error_text),
                })
            }
            _ => {
                let error_text = response.text().await.unwrap_or_default();
                Err(IndexerError::GoogleApiError {
                    status_code: status_code.as_u16(),
                    message: error_text,
                })
            }
        }
    }
}

/// Quota information
#[derive(Debug, Clone)]
pub struct QuotaInfo {
    /// Daily publish limit
    pub daily_publish_limit: usize,
    /// Daily publish quota used
    pub daily_publish_used: usize,
    /// Rate limit per minute
    pub rate_limit_per_minute: usize,
    /// Metadata rate limit per minute
    pub metadata_rate_limit_per_minute: usize,
}

/// Search Console site entry
#[derive(Debug, Clone, Deserialize)]
pub struct SiteEntry {
    /// Site URL
    #[serde(rename = "siteUrl")]
    pub site_url: String,
    /// Permission level
    #[serde(rename = "permissionLevel")]
    pub permission_level: String,
}

/// Search Console sites list response
#[derive(Debug, Clone, Deserialize)]
pub struct SitesListResponse {
    /// List of sites
    #[serde(rename = "siteEntry", default)]
    pub site_entry: Vec<SiteEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_type_display() {
        assert_eq!(NotificationType::UrlUpdated.to_string(), "URL_UPDATED");
        assert_eq!(NotificationType::UrlDeleted.to_string(), "URL_DELETED");
    }

    #[test]
    fn test_notification_type_serialization() {
        let notification = UrlNotification {
            url: "https://placeholder.test/page1".to_string(),
            notification_type: NotificationType::UrlUpdated.to_string(),
        };

        let json = serde_json::to_string(&notification).unwrap();
        assert!(json.contains("URL_UPDATED"));
        assert!(json.contains("https://placeholder.test/page1"));
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(5);

        // First 5 requests should be immediate
        for _ in 0..5 {
            limiter.wait_if_needed().await;
        }

        assert_eq!(limiter.request_times.len(), 5);
    }
}
