//! Batch submission manager service.
//!
//! This module provides intelligent batch submission management, coordinating
//! URL submissions to Google Indexing API and IndexNow API with automatic
//! batching, history tracking, progress reporting, and error handling.

use crate::api::google_indexing::{GoogleIndexingClient, NotificationType};
use crate::api::indexnow::IndexNowClient;
use crate::database::models::{ActionType, ApiType, SubmissionRecord, SubmissionStatus};
use crate::database::queries::{check_url_submitted, insert_submission};
use crate::types::error::IndexerError;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use futures::stream::{self, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rusqlite::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

/// Maximum batch size for Google Indexing API
const GOOGLE_MAX_BATCH_SIZE: usize = 100;

/// Maximum batch size for IndexNow API
const INDEXNOW_MAX_BATCH_SIZE: usize = 10_000;

/// Default concurrent batches to process
const DEFAULT_CONCURRENT_BATCHES: usize = 3;

// =============================================================================
// Configuration
// =============================================================================

/// Configuration for batch submission operations
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Batch size for Google submissions (max 100)
    pub google_batch_size: usize,

    /// Batch size for IndexNow submissions (max 10,000)
    pub indexnow_batch_size: usize,

    /// Whether to check history before submission
    pub check_history: bool,

    /// Number of concurrent batches to process
    pub concurrent_batches: usize,

    /// Whether to show progress bars
    pub progress_bar: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            google_batch_size: GOOGLE_MAX_BATCH_SIZE,
            indexnow_batch_size: INDEXNOW_MAX_BATCH_SIZE,
            check_history: true,
            concurrent_batches: DEFAULT_CONCURRENT_BATCHES,
            progress_bar: true,
        }
    }
}

impl BatchConfig {
    /// Create a new batch configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set Google batch size (will be capped at max allowed)
    pub fn with_google_batch_size(mut self, size: usize) -> Self {
        self.google_batch_size = size.min(GOOGLE_MAX_BATCH_SIZE);
        self
    }

    /// Set IndexNow batch size (will be capped at max allowed)
    pub fn with_indexnow_batch_size(mut self, size: usize) -> Self {
        self.indexnow_batch_size = size.min(INDEXNOW_MAX_BATCH_SIZE);
        self
    }

    /// Set whether to check history before submission
    pub fn with_check_history(mut self, check: bool) -> Self {
        self.check_history = check;
        self
    }

    /// Set number of concurrent batches
    pub fn with_concurrent_batches(mut self, count: usize) -> Self {
        self.concurrent_batches = count.max(1);
        self
    }

    /// Set whether to show progress bars
    pub fn with_progress_bar(mut self, show: bool) -> Self {
        self.progress_bar = show;
        self
    }
}

// =============================================================================
// Result Types
// =============================================================================

/// Results for API submissions
#[derive(Debug, Clone)]
pub struct ApiResults {
    /// Number of successful submissions
    pub successful: usize,

    /// Number of failed submissions
    pub failed: usize,

    /// List of error messages
    pub errors: Vec<String>,
}

impl ApiResults {
    /// Create new empty API results
    fn new() -> Self {
        Self {
            successful: 0,
            failed: 0,
            errors: Vec::new(),
        }
    }

    /// Add a successful submission
    fn add_success(&mut self) {
        self.successful += 1;
    }

    /// Add a failed submission
    fn add_failure(&mut self, error: String) {
        self.failed += 1;
        self.errors.push(error);
    }
}

/// Overall batch submission result
#[derive(Debug, Clone)]
pub struct BatchResult {
    /// Total number of URLs processed
    pub total_urls: usize,

    /// Number of URLs submitted (not skipped)
    pub submitted: usize,

    /// Number of URLs skipped (already submitted recently)
    pub skipped: usize,

    /// Google API results (if used)
    pub google_results: Option<ApiResults>,

    /// IndexNow API results (if used)
    pub indexnow_results: Option<ApiResults>,
}

impl BatchResult {
    /// Create a new empty batch result
    fn new(total_urls: usize) -> Self {
        Self {
            total_urls,
            submitted: 0,
            skipped: 0,
            google_results: None,
            indexnow_results: None,
        }
    }

    /// Get total successful submissions across all APIs
    pub fn total_successful(&self) -> usize {
        let google = self.google_results.as_ref().map(|r| r.successful).unwrap_or(0);
        let indexnow = self.indexnow_results.as_ref().map(|r| r.successful).unwrap_or(0);
        google + indexnow
    }

    /// Get total failed submissions across all APIs
    pub fn total_failed(&self) -> usize {
        let google = self.google_results.as_ref().map(|r| r.failed).unwrap_or(0);
        let indexnow = self.indexnow_results.as_ref().map(|r| r.failed).unwrap_or(0);
        google + indexnow
    }

    /// Check if the operation was successful overall
    pub fn is_success(&self) -> bool {
        self.total_failed() == 0
    }
}

// =============================================================================
// Batch Submitter
// =============================================================================

/// History manager wrapper to handle database operations
pub struct HistoryManager {
    db_conn: Arc<Mutex<Connection>>,
}

impl HistoryManager {
    /// Create a new history manager
    pub fn new(db_conn: Connection) -> Self {
        Self {
            db_conn: Arc::new(Mutex::new(db_conn)),
        }
    }

    /// Check if a URL was submitted to a specific API within a time window
    pub async fn is_url_submitted(
        &self,
        url: &str,
        api: ApiType,
        since: Option<DateTime<Utc>>,
    ) -> Result<bool, IndexerError> {
        let conn = self.db_conn.lock().await;
        check_url_submitted(&conn, url, api, since)
    }

    /// Record a submission to the database
    pub async fn record_submission(&self, record: SubmissionRecord) -> Result<i64, IndexerError> {
        let conn = self.db_conn.lock().await;
        insert_submission(&conn, &record)
    }
}

/// Intelligent batch submission manager
pub struct BatchSubmitter {
    /// Google Indexing API client (optional)
    google_client: Option<Arc<GoogleIndexingClient>>,

    /// IndexNow API client (optional)
    indexnow_client: Option<Arc<IndexNowClient>>,

    /// History manager for tracking submissions
    history_manager: Arc<HistoryManager>,

    /// Batch configuration
    config: BatchConfig,
}

impl BatchSubmitter {
    /// Create a new batch submitter
    ///
    /// # Arguments
    ///
    /// * `google_client` - Optional Google Indexing API client
    /// * `indexnow_client` - Optional IndexNow API client
    /// * `history_manager` - History manager for tracking submissions
    /// * `config` - Batch configuration
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use indexer_cli::services::batch_submitter::{BatchSubmitter, BatchConfig, HistoryManager};
    /// use indexer_cli::api::google_indexing::GoogleIndexingClient;
    /// use indexer_cli::database::init_database;
    /// use std::path::PathBuf;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Initialize database and history manager
    /// let db_conn = init_database(&PathBuf::from("./data/indexer.db"))?;
    /// let history_manager = Arc::new(HistoryManager::new(db_conn));
    ///
    /// // Initialize Google client
    /// let google_client = GoogleIndexingClient::new(
    ///     PathBuf::from("./service-account.json")
    /// ).await?;
    ///
    /// // Create batch submitter
    /// let submitter = BatchSubmitter::new(
    ///     Some(Arc::new(google_client)),
    ///     None,
    ///     history_manager,
    ///     BatchConfig::default(),
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(
        google_client: Option<Arc<GoogleIndexingClient>>,
        indexnow_client: Option<Arc<IndexNowClient>>,
        history_manager: Arc<HistoryManager>,
        config: BatchConfig,
    ) -> Self {
        Self {
            google_client,
            indexnow_client,
            history_manager,
            config,
        }
    }

    /// Submit URLs to Google Indexing API
    ///
    /// # Arguments
    ///
    /// * `urls` - List of URLs to submit
    /// * `action` - Action type (URL_UPDATED or URL_DELETED)
    ///
    /// # Returns
    ///
    /// Returns the batch submission result
    pub async fn submit_to_google(
        &self,
        urls: Vec<String>,
        action: NotificationType,
    ) -> Result<BatchResult, IndexerError> {
        let google_client = self.google_client.as_ref().ok_or_else(|| {
            IndexerError::ConfigMissingField {
                field: "google_client".to_string(),
            }
        })?;

        info!(
            "Starting Google batch submission: {} URLs with action {:?}",
            urls.len(),
            action
        );

        let total_urls = urls.len();
        let mut result = BatchResult::new(total_urls);
        let mut api_results = ApiResults::new();

        // Filter URLs based on history
        let urls_to_submit = if self.config.check_history {
            self.filter_submitted_urls(&urls, ApiType::Google, Some(ChronoDuration::hours(24)))
                .await?
        } else {
            urls.clone()
        };

        result.skipped = total_urls - urls_to_submit.len();
        result.submitted = urls_to_submit.len();

        if urls_to_submit.is_empty() {
            info!("All URLs already submitted recently, skipping");
            result.google_results = Some(api_results);
            return Ok(result);
        }

        // Create progress bar
        let progress = if self.config.progress_bar {
            let pb = ProgressBar::new(urls_to_submit.len() as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
                    .expect("Invalid progress bar template")
                    .progress_chars("=>-"),
            );
            pb.set_message("Submitting to Google...");
            Some(pb)
        } else {
            None
        };

        // Split into batches
        let batches: Vec<_> = urls_to_submit
            .chunks(self.config.google_batch_size)
            .map(|chunk| chunk.to_vec())
            .collect();

        info!("Processing {} batches of up to {} URLs each", batches.len(), self.config.google_batch_size);

        // Process batches concurrently
        let mut batch_stream = stream::iter(batches)
            .map(|batch| {
                let client = Arc::clone(google_client);
                let history = Arc::clone(&self.history_manager);
                let pb = progress.clone();
                let action_type = match action {
                    NotificationType::UrlUpdated => ActionType::UrlUpdated,
                    NotificationType::UrlDeleted => ActionType::UrlDeleted,
                };

                async move {
                    let mut batch_results = Vec::new();

                    for url in &batch {
                        let submission_result = client.publish_url(url, action).await;

                        // Record to history
                        let record = match &submission_result {
                            Ok(sub_result) => SubmissionRecord::builder()
                                .url(url.clone())
                                .api(ApiType::Google)
                                .action(action_type)
                                .status(if sub_result.success {
                                    SubmissionStatus::Success
                                } else {
                                    SubmissionStatus::Failed
                                })
                                .response_code(sub_result.status_code.map(|c| c as i32))
                                .response_message(sub_result.message.clone())
                                .submitted_at(sub_result.submitted_at)
                                .build(),
                            Err(e) => SubmissionRecord::builder()
                                .url(url.clone())
                                .api(ApiType::Google)
                                .action(action_type)
                                .status(SubmissionStatus::Failed)
                                .response_message(e.to_string())
                                .build(),
                        };

                        if let Ok(rec) = record {
                            let _ = history.record_submission(rec).await;
                        }

                        batch_results.push(submission_result);

                        if let Some(ref pb) = pb {
                            pb.inc(1);
                        }
                    }

                    batch_results
                }
            })
            .buffer_unordered(self.config.concurrent_batches);

        // Collect results
        while let Some(batch_results) = batch_stream.next().await {
            for submission_result in batch_results {
                match submission_result {
                    Ok(sub_result) => {
                        if sub_result.success {
                            api_results.add_success();
                        } else {
                            api_results.add_failure(sub_result.message);
                        }
                    }
                    Err(e) => {
                        api_results.add_failure(e.to_string());
                    }
                }
            }
        }

        if let Some(pb) = progress {
            pb.finish_with_message("Google submission complete");
        }

        result.google_results = Some(api_results);

        info!(
            "Google batch submission completed: {}/{} successful",
            result.google_results.as_ref().unwrap().successful,
            result.submitted
        );

        Ok(result)
    }

    /// Submit URLs to IndexNow API
    ///
    /// # Arguments
    ///
    /// * `urls` - List of URLs to submit
    ///
    /// # Returns
    ///
    /// Returns the batch submission result
    pub async fn submit_to_indexnow(
        &self,
        urls: Vec<String>,
    ) -> Result<BatchResult, IndexerError> {
        let indexnow_client = self.indexnow_client.as_ref().ok_or_else(|| {
            IndexerError::ConfigMissingField {
                field: "indexnow_client".to_string(),
            }
        })?;

        info!("Starting IndexNow batch submission: {} URLs", urls.len());

        let total_urls = urls.len();
        let mut result = BatchResult::new(total_urls);
        let mut api_results = ApiResults::new();

        // Filter URLs based on history
        let urls_to_submit = if self.config.check_history {
            self.filter_submitted_urls(&urls, ApiType::IndexNow, Some(ChronoDuration::hours(24)))
                .await?
        } else {
            urls.clone()
        };

        result.skipped = total_urls - urls_to_submit.len();
        result.submitted = urls_to_submit.len();

        if urls_to_submit.is_empty() {
            info!("All URLs already submitted recently, skipping");
            result.indexnow_results = Some(api_results);
            return Ok(result);
        }

        // Create progress bar
        let progress = if self.config.progress_bar {
            let pb = ProgressBar::new(urls_to_submit.len() as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] {bar:40.green/blue} {pos}/{len} {msg}")
                    .expect("Invalid progress bar template")
                    .progress_chars("=>-"),
            );
            pb.set_message("Submitting to IndexNow...");
            Some(pb)
        } else {
            None
        };

        // Split into batches
        let batches: Vec<_> = urls_to_submit
            .chunks(self.config.indexnow_batch_size)
            .map(|chunk| chunk.to_vec())
            .collect();

        info!("Processing {} batches of up to {} URLs each", batches.len(), self.config.indexnow_batch_size);

        // Process batches - submit to all endpoints concurrently
        for batch in batches {
            let batch_size = batch.len();
            let responses = indexnow_client.submit_to_all(&batch).await;

            // Process responses from all endpoints
            for (idx, response) in responses.iter().enumerate() {
                match response {
                    Ok(resp) => {
                        if resp.is_success() {
                            // Only count success once per URL (from first endpoint)
                            if idx == 0 {
                                api_results.successful += batch_size;
                            }
                            debug!("IndexNow endpoint {} successful: {}", resp.endpoint, batch_size);
                        } else {
                            warn!("IndexNow endpoint {} failed: {}", resp.endpoint, resp.message);
                            if idx == 0 {
                                api_results.add_failure(format!(
                                    "Endpoint {}: {}",
                                    resp.endpoint, resp.message
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        error!("IndexNow submission error: {}", e);
                        if idx == 0 {
                            api_results.add_failure(e.to_string());
                        }
                    }
                }
            }

            // Record to history (use first response status)
            let is_success = responses.first().map(|r| r.is_ok()).unwrap_or(false);
            for url in &batch {
                let record = SubmissionRecord::builder()
                    .url(url.clone())
                    .api(ApiType::IndexNow)
                    .action(ActionType::UrlUpdated)
                    .status(if is_success {
                        SubmissionStatus::Success
                    } else {
                        SubmissionStatus::Failed
                    })
                    .response_code(
                        responses
                            .first()
                            .and_then(|r| r.as_ref().ok())
                            .map(|r| r.status_code as i32),
                    )
                    .build();

                if let Ok(rec) = record {
                    let _ = self.history_manager.record_submission(rec).await;
                }
            }

            if let Some(ref pb) = progress {
                pb.inc(batch_size as u64);
            }
        }

        if let Some(pb) = progress {
            pb.finish_with_message("IndexNow submission complete");
        }

        result.indexnow_results = Some(api_results);

        info!(
            "IndexNow batch submission completed: {}/{} successful",
            result.indexnow_results.as_ref().unwrap().successful,
            result.submitted
        );

        Ok(result)
    }

    /// Submit URLs to all enabled APIs concurrently
    ///
    /// # Arguments
    ///
    /// * `urls` - List of URLs to submit
    /// * `action` - Action type for Google API (URL_UPDATED or URL_DELETED)
    ///
    /// # Returns
    ///
    /// Returns the aggregated batch submission result
    pub async fn submit_to_all(
        &self,
        urls: Vec<String>,
        action: NotificationType,
    ) -> Result<BatchResult, IndexerError> {
        info!(
            "Starting batch submission to all APIs: {} URLs",
            urls.len()
        );

        let total_urls = urls.len();
        let mut final_result = BatchResult::new(total_urls);

        // Create multi-progress for concurrent submissions
        let _multi = if self.config.progress_bar {
            Some(MultiProgress::new())
        } else {
            None
        };

        let mut handles = vec![];

        // Submit to Google if client is available
        if self.google_client.is_some() {
            let urls_clone = urls.clone();
            let self_clone = Self {
                google_client: self.google_client.clone(),
                indexnow_client: None,
                history_manager: Arc::clone(&self.history_manager),
                config: self.config.clone(),
            };

            let handle = tokio::spawn(async move {
                self_clone.submit_to_google(urls_clone, action).await
            });
            handles.push(("google", handle));
        }

        // Submit to IndexNow if client is available
        if self.indexnow_client.is_some() {
            let urls_clone = urls.clone();
            let self_clone = Self {
                google_client: None,
                indexnow_client: self.indexnow_client.clone(),
                history_manager: Arc::clone(&self.history_manager),
                config: self.config.clone(),
            };

            let handle = tokio::spawn(async move {
                self_clone.submit_to_indexnow(urls_clone).await
            });
            handles.push(("indexnow", handle));
        }

        // Wait for all submissions to complete
        for (api_name, handle) in handles {
            match handle.await {
                Ok(Ok(result)) => {
                    match api_name {
                        "google" => {
                            final_result.google_results = result.google_results;
                            final_result.submitted = final_result.submitted.max(result.submitted);
                            final_result.skipped = final_result.skipped.max(result.skipped);
                        }
                        "indexnow" => {
                            final_result.indexnow_results = result.indexnow_results;
                            final_result.submitted = final_result.submitted.max(result.submitted);
                            final_result.skipped = final_result.skipped.max(result.skipped);
                        }
                        _ => {}
                    }
                }
                Ok(Err(e)) => {
                    error!("API {} submission failed: {}", api_name, e);
                }
                Err(e) => {
                    error!("API {} task join error: {}", api_name, e);
                }
            }
        }

        info!(
            "All API batch submissions completed: {} submitted, {} skipped, {} successful, {} failed",
            final_result.submitted,
            final_result.skipped,
            final_result.total_successful(),
            final_result.total_failed()
        );

        Ok(final_result)
    }

    /// Filter out URLs that have been submitted recently
    ///
    /// # Arguments
    ///
    /// * `urls` - List of URLs to filter
    /// * `api` - API type to check against
    /// * `since` - Only consider submissions newer than this duration
    ///
    /// # Returns
    ///
    /// Returns a list of URLs that have not been submitted or need resubmission
    pub async fn filter_submitted_urls(
        &self,
        urls: &[String],
        api: ApiType,
        since: Option<ChronoDuration>,
    ) -> Result<Vec<String>, IndexerError> {
        debug!(
            "Filtering {} URLs against {} history",
            urls.len(),
            api
        );

        let since_timestamp = since.map(|d| Utc::now() - d);
        let mut filtered = Vec::new();

        for url in urls {
            let is_submitted = self
                .history_manager
                .is_url_submitted(url, api, since_timestamp)
                .await?;

            if !is_submitted {
                filtered.push(url.clone());
            } else {
                debug!("URL already submitted recently: {}", url);
            }
        }

        info!(
            "Filtered {} URLs, {} need submission",
            urls.len(),
            filtered.len()
        );

        Ok(filtered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_config_defaults() {
        let config = BatchConfig::default();
        assert_eq!(config.google_batch_size, GOOGLE_MAX_BATCH_SIZE);
        assert_eq!(config.indexnow_batch_size, INDEXNOW_MAX_BATCH_SIZE);
        assert!(config.check_history);
        assert_eq!(config.concurrent_batches, DEFAULT_CONCURRENT_BATCHES);
        assert!(config.progress_bar);
    }

    #[test]
    fn test_batch_config_builder() {
        let config = BatchConfig::new()
            .with_google_batch_size(50)
            .with_indexnow_batch_size(5000)
            .with_check_history(false)
            .with_concurrent_batches(5)
            .with_progress_bar(false);

        assert_eq!(config.google_batch_size, 50);
        assert_eq!(config.indexnow_batch_size, 5000);
        assert!(!config.check_history);
        assert_eq!(config.concurrent_batches, 5);
        assert!(!config.progress_bar);
    }

    #[test]
    fn test_batch_config_capping() {
        let config = BatchConfig::new()
            .with_google_batch_size(200) // Should be capped at 100
            .with_indexnow_batch_size(20000); // Should be capped at 10,000

        assert_eq!(config.google_batch_size, GOOGLE_MAX_BATCH_SIZE);
        assert_eq!(config.indexnow_batch_size, INDEXNOW_MAX_BATCH_SIZE);
    }

    #[test]
    fn test_api_results() {
        let mut results = ApiResults::new();
        assert_eq!(results.successful, 0);
        assert_eq!(results.failed, 0);

        results.add_success();
        results.add_success();
        assert_eq!(results.successful, 2);

        results.add_failure("Error 1".to_string());
        assert_eq!(results.failed, 1);
        assert_eq!(results.errors.len(), 1);
    }

    #[test]
    fn test_batch_result() {
        let mut result = BatchResult::new(100);
        assert_eq!(result.total_urls, 100);
        assert_eq!(result.submitted, 0);
        assert_eq!(result.skipped, 0);

        let mut google_results = ApiResults::new();
        google_results.successful = 50;
        google_results.failed = 5;
        result.google_results = Some(google_results);

        let mut indexnow_results = ApiResults::new();
        indexnow_results.successful = 45;
        indexnow_results.failed = 0;
        result.indexnow_results = Some(indexnow_results);

        assert_eq!(result.total_successful(), 95);
        assert_eq!(result.total_failed(), 5);
        assert!(!result.is_success());
    }
}
