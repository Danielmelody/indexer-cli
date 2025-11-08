//! History manager service for managing submission history records.
//!
//! This module provides a high-level service layer for managing URL submission
//! history, including recording submissions, querying history, generating statistics,
//! and exporting data to various formats.

use crate::database::{
    models::{ActionType, ApiType, SubmissionRecord, SubmissionStatus},
    queries::{
        check_url_submitted, count_submissions, delete_old_submissions, get_submission_by_url,
        get_submissions_stats, insert_submission, list_submissions, SubmissionFilters,
        SubmissionStats as DbSubmissionStats,
    },
    schema::init_database,
};
use crate::types::IndexerError;
use chrono::{DateTime, Duration, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing::{debug, info, warn};

/// History manager for managing submission records
pub struct HistoryManager {
    /// Path to the database file
    db_path: PathBuf,
    /// Database connection (thread-safe with Arc<Mutex>)
    connection: Arc<Mutex<Connection>>,
    /// Number of days to retain history records
    retention_days: i64,
}

/// Statistics about submission history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionStats {
    /// Total number of submissions
    pub total_submissions: i64,
    /// Number of successful submissions
    pub successful: i64,
    /// Number of failed submissions
    pub failed: i64,
    /// Number of Google API submissions
    pub google_count: i64,
    /// Number of IndexNow API submissions
    pub indexnow_count: i64,
    /// Number of submissions in the last 7 days
    pub last_7_days: i64,
    /// Number of submissions in the last 30 days
    pub last_30_days: i64,
    /// Timestamp of the last submission
    pub last_submission: Option<DateTime<Utc>>,
}

/// Filters for querying submission history
#[derive(Debug, Clone, Default)]
pub struct HistoryFilters {
    /// Filter by API type
    pub api: Option<ApiType>,
    /// Filter by submission status
    pub status: Option<SubmissionStatus>,
    /// Filter by submissions after this date
    pub date_from: Option<DateTime<Utc>>,
    /// Filter by submissions before this date
    pub date_to: Option<DateTime<Utc>>,
    /// Filter by URL pattern (supports SQL LIKE wildcards)
    pub url_pattern: Option<String>,
}

impl HistoryFilters {
    /// Create a new empty filter set
    pub fn new() -> Self {
        Self::default()
    }

    /// Convert to database filters
    fn to_db_filters(&self) -> SubmissionFilters {
        let mut filters = SubmissionFilters::new();

        if let Some(api) = self.api {
            filters = filters.api(api);
        }

        if let Some(status) = self.status {
            filters = filters.status(status);
        }

        if let Some(date_from) = self.date_from {
            filters = filters.after(date_from);
        }

        if let Some(date_to) = self.date_to {
            filters = filters.before(date_to);
        }

        if let Some(ref pattern) = self.url_pattern {
            filters = filters.url_pattern(pattern.clone());
        }

        filters
    }
}

impl HistoryManager {
    /// Create a new HistoryManager instance
    ///
    /// # Arguments
    ///
    /// * `db_path` - Path to the SQLite database file
    /// * `retention_days` - Number of days to retain history records (0 = keep forever)
    ///
    /// # Returns
    ///
    /// A Result containing the HistoryManager instance or an IndexerError
    ///
    /// # Example
    ///
    /// ```no_run
    /// use indexer_cli::services::history_manager::HistoryManager;
    /// use std::path::Path;
    ///
    /// let manager = HistoryManager::new(
    ///     Path::new("./data/indexer.db"),
    ///     90  // Keep records for 90 days
    /// ).expect("Failed to create history manager");
    /// ```
    pub fn new(db_path: &Path, retention_days: i64) -> Result<Self, IndexerError> {
        info!(
            "Initializing HistoryManager with database at: {}",
            db_path.display()
        );

        // Initialize database and run migrations
        let connection = init_database(db_path)?;

        Ok(Self {
            db_path: db_path.to_path_buf(),
            connection: Arc::new(Mutex::new(connection)),
            retention_days,
        })
    }

    /// Get a database connection (with automatic reconnection if needed)
    fn get_connection(&self) -> Result<std::sync::MutexGuard<Connection>, IndexerError> {
        self.connection.lock().map_err(|e| {
            IndexerError::DatabaseConnectionFailed {
                message: format!("Failed to acquire database lock: {}", e),
            }
        })
    }

    /// Record a single URL submission
    ///
    /// # Arguments
    ///
    /// * `url` - The URL that was submitted
    /// * `api` - The API used for submission
    /// * `action` - The action performed
    /// * `status` - The status of the submission
    /// * `response` - Optional response information (code and message)
    ///
    /// # Returns
    ///
    /// The ID of the inserted record or an IndexerError
    ///
    /// # Example
    ///
    /// ```no_run
    /// use indexer_cli::services::history_manager::HistoryManager;
    /// use indexer_cli::database::{ApiType, ActionType, SubmissionStatus};
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = HistoryManager::new(Path::new("./data/indexer.db"), 90)?;
    /// let id = manager.record_submission(
    ///     "https://example.com/page",
    ///     ApiType::Google,
    ///     ActionType::UrlUpdated,
    ///     SubmissionStatus::Success,
    ///     Some((200, "OK".to_string()))
    /// )?;
    /// println!("Recorded submission with ID: {}", id);
    /// # Ok(())
    /// # }
    /// ```
    pub fn record_submission(
        &self,
        url: &str,
        api: ApiType,
        action: ActionType,
        status: SubmissionStatus,
        response: Option<(i32, String)>,
    ) -> Result<i64, IndexerError> {
        debug!(
            "Recording submission: url={}, api={}, action={}, status={}",
            url, api, action, status
        );

        let (response_code, response_message) = response
            .map(|(code, msg)| (Some(code), Some(msg)))
            .unwrap_or((None, None));

        let record = SubmissionRecord::builder()
            .url(url)
            .api(api)
            .action(action)
            .status(status)
            .response_code(response_code.unwrap_or(0))
            .response_message(response_message.unwrap_or_default())
            .submitted_at(Utc::now())
            .build()
            .map_err(|e| IndexerError::DatabaseQueryFailed {
                message: format!("Failed to build submission record: {}", e),
            })?;

        let conn = self.get_connection()?;
        insert_submission(&conn, &record)
    }

    /// Record multiple submissions in a batch (using a transaction for performance)
    ///
    /// # Arguments
    ///
    /// * `records` - Vector of submission records to insert
    ///
    /// # Returns
    ///
    /// The number of records inserted or an IndexerError
    ///
    /// # Example
    ///
    /// ```no_run
    /// use indexer_cli::services::history_manager::HistoryManager;
    /// use indexer_cli::database::{SubmissionRecord, ApiType, ActionType, SubmissionStatus};
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = HistoryManager::new(Path::new("./data/indexer.db"), 90)?;
    ///
    /// let records = vec![
    ///     SubmissionRecord::builder()
    ///         .url("https://example.com/page1")
    ///         .api(ApiType::Google)
    ///         .action(ActionType::UrlUpdated)
    ///         .status(SubmissionStatus::Success)
    ///         .build()?,
    ///     SubmissionRecord::builder()
    ///         .url("https://example.com/page2")
    ///         .api(ApiType::IndexNow)
    ///         .action(ActionType::UrlUpdated)
    ///         .status(SubmissionStatus::Success)
    ///         .build()?,
    /// ];
    ///
    /// let count = manager.record_batch_submissions(&records)?;
    /// println!("Recorded {} submissions", count);
    /// # Ok(())
    /// # }
    /// ```
    pub fn record_batch_submissions(
        &self,
        records: &[SubmissionRecord],
    ) -> Result<usize, IndexerError> {
        info!("Recording batch of {} submissions", records.len());

        let conn = self.get_connection()?;

        // Start transaction
        let tx = conn
            .transaction()
            .map_err(|e| IndexerError::DatabaseTransactionFailed {
                message: format!("Failed to start transaction: {}", e),
            })?;

        let mut count = 0;
        for record in records {
            insert_submission(&tx, record)?;
            count += 1;
        }

        // Commit transaction
        tx.commit()
            .map_err(|e| IndexerError::DatabaseTransactionFailed {
                message: format!("Failed to commit transaction: {}", e),
            })?;

        info!("Successfully recorded {} submissions", count);
        Ok(count)
    }

    /// Check if a URL has been submitted to a specific API within a time window
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to check
    /// * `api` - The API type to check
    /// * `since` - Check for submissions after this timestamp
    ///
    /// # Returns
    ///
    /// True if the URL was submitted since the given timestamp, false otherwise
    ///
    /// # Example
    ///
    /// ```no_run
    /// use indexer_cli::services::history_manager::HistoryManager;
    /// use indexer_cli::database::ApiType;
    /// use chrono::{Utc, Duration};
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = HistoryManager::new(Path::new("./data/indexer.db"), 90)?;
    ///
    /// let one_day_ago = Utc::now() - Duration::days(1);
    /// let submitted = manager.is_url_submitted(
    ///     "https://example.com/page",
    ///     ApiType::Google,
    ///     one_day_ago
    /// )?;
    ///
    /// if submitted {
    ///     println!("URL was submitted in the last 24 hours");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_url_submitted(
        &self,
        url: &str,
        api: ApiType,
        since: DateTime<Utc>,
    ) -> Result<bool, IndexerError> {
        let conn = self.get_connection()?;
        check_url_submitted(&conn, url, api, since)
    }

    /// Get submission history for a specific URL and API
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to query
    /// * `api` - The API type to filter by
    ///
    /// # Returns
    ///
    /// A vector of submission records ordered by timestamp (most recent first)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use indexer_cli::services::history_manager::HistoryManager;
    /// use indexer_cli::database::ApiType;
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = HistoryManager::new(Path::new("./data/indexer.db"), 90)?;
    ///
    /// let history = manager.get_submission_history(
    ///     "https://example.com/page",
    ///     ApiType::Google
    /// )?;
    ///
    /// for record in history {
    ///     println!("{:?}: {}", record.submitted_at, record.status);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_submission_history(
        &self,
        url: &str,
        api: ApiType,
    ) -> Result<Vec<SubmissionRecord>, IndexerError> {
        let conn = self.get_connection()?;

        let filters = SubmissionFilters::new().url_pattern(url.to_string()).api(api);

        list_submissions(&conn, &filters)
    }

    /// List recent submissions with optional filtering and pagination
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of records to return
    /// * `filters` - Optional filters to apply
    ///
    /// # Returns
    ///
    /// A vector of submission records matching the criteria
    ///
    /// # Example
    ///
    /// ```no_run
    /// use indexer_cli::services::history_manager::{HistoryManager, HistoryFilters};
    /// use indexer_cli::database::{ApiType, SubmissionStatus};
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = HistoryManager::new(Path::new("./data/indexer.db"), 90)?;
    ///
    /// let mut filters = HistoryFilters::new();
    /// filters.api = Some(ApiType::Google);
    /// filters.status = Some(SubmissionStatus::Success);
    ///
    /// let recent = manager.list_recent_submissions(100, Some(filters))?;
    /// println!("Found {} recent submissions", recent.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_recent_submissions(
        &self,
        limit: usize,
        filters: Option<HistoryFilters>,
    ) -> Result<Vec<SubmissionRecord>, IndexerError> {
        let conn = self.get_connection()?;

        let mut db_filters = filters
            .map(|f| f.to_db_filters())
            .unwrap_or_else(SubmissionFilters::new);

        db_filters = db_filters.limit(limit);

        list_submissions(&conn, &db_filters)
    }

    /// Get comprehensive statistics about submission history
    ///
    /// # Arguments
    ///
    /// * `filters` - Optional filters to apply
    ///
    /// # Returns
    ///
    /// Statistics about the submission history
    ///
    /// # Example
    ///
    /// ```no_run
    /// use indexer_cli::services::history_manager::HistoryManager;
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = HistoryManager::new(Path::new("./data/indexer.db"), 90)?;
    ///
    /// let stats = manager.get_statistics(None)?;
    /// println!("Total submissions: {}", stats.total_submissions);
    /// println!("Success rate: {:.1}%",
    ///     (stats.successful as f64 / stats.total_submissions as f64) * 100.0);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_statistics(
        &self,
        filters: Option<HistoryFilters>,
    ) -> Result<SubmissionStats, IndexerError> {
        let conn = self.get_connection()?;

        // Get basic stats
        let db_stats = get_submissions_stats(&conn)?;

        // Calculate time-based statistics
        let seven_days_ago = Utc::now() - Duration::days(7);
        let thirty_days_ago = Utc::now() - Duration::days(30);

        let last_7_days = {
            let mut db_filters = filters
                .as_ref()
                .map(|f| f.to_db_filters())
                .unwrap_or_else(SubmissionFilters::new);
            db_filters = db_filters.after(seven_days_ago);
            count_submissions(&conn, &db_filters)?
        };

        let last_30_days = {
            let mut db_filters = filters
                .map(|f| f.to_db_filters())
                .unwrap_or_else(SubmissionFilters::new);
            db_filters = db_filters.after(thirty_days_ago);
            count_submissions(&conn, &db_filters)?
        };

        Ok(SubmissionStats {
            total_submissions: db_stats.total,
            successful: db_stats.success,
            failed: db_stats.failed,
            google_count: db_stats.google,
            indexnow_count: db_stats.indexnow,
            last_7_days,
            last_30_days,
            last_submission: db_stats.last_submission,
        })
    }

    /// Clean up old submission records older than the specified number of days
    ///
    /// # Arguments
    ///
    /// * `days` - Delete records older than this many days (overrides retention_days)
    ///
    /// # Returns
    ///
    /// The number of records deleted
    ///
    /// # Example
    ///
    /// ```no_run
    /// use indexer_cli::services::history_manager::HistoryManager;
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = HistoryManager::new(Path::new("./data/indexer.db"), 90)?;
    ///
    /// // Clean records older than 180 days
    /// let deleted = manager.clean_old_records(180)?;
    /// println!("Deleted {} old records", deleted);
    /// # Ok(())
    /// # }
    /// ```
    pub fn clean_old_records(&self, days: i64) -> Result<usize, IndexerError> {
        if days <= 0 {
            warn!("Invalid days value for cleanup: {}. Skipping cleanup.", days);
            return Ok(0);
        }

        info!("Cleaning records older than {} days", days);
        let conn = self.get_connection()?;
        delete_old_submissions(&conn, days)
    }

    /// Clean up old records based on the configured retention period
    ///
    /// # Returns
    ///
    /// The number of records deleted
    pub fn clean_old_records_auto(&self) -> Result<usize, IndexerError> {
        if self.retention_days <= 0 {
            debug!("Retention period not set (or set to 0), skipping automatic cleanup");
            return Ok(0);
        }

        self.clean_old_records(self.retention_days)
    }

    /// Export submission history to CSV format
    ///
    /// # Arguments
    ///
    /// * `output_path` - Path to the output CSV file
    /// * `filters` - Optional filters to apply
    ///
    /// # Returns
    ///
    /// The number of records exported or an IndexerError
    ///
    /// # Example
    ///
    /// ```no_run
    /// use indexer_cli::services::history_manager::HistoryManager;
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = HistoryManager::new(Path::new("./data/indexer.db"), 90)?;
    ///
    /// let count = manager.export_to_csv(
    ///     Path::new("./exports/submissions.csv"),
    ///     None
    /// )?;
    /// println!("Exported {} records to CSV", count);
    /// # Ok(())
    /// # }
    /// ```
    pub fn export_to_csv(
        &self,
        output_path: &Path,
        filters: Option<HistoryFilters>,
    ) -> Result<usize, IndexerError> {
        info!("Exporting submissions to CSV: {}", output_path.display());

        let conn = self.get_connection()?;
        let db_filters = filters
            .map(|f| f.to_db_filters())
            .unwrap_or_else(SubmissionFilters::new);

        let records = list_submissions(&conn, &db_filters)?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                IndexerError::DirectoryCreationFailed {
                    path: parent.to_path_buf(),
                    message: e.to_string(),
                }
            })?;
        }

        // Write CSV
        let file = std::fs::File::create(output_path).map_err(|e| {
            IndexerError::FileWriteError {
                path: output_path.to_path_buf(),
                message: e.to_string(),
            }
        })?;

        let mut writer = csv::Writer::from_writer(file);

        // Write header
        writer
            .write_record(&[
                "id",
                "url",
                "api",
                "action",
                "status",
                "response_code",
                "response_message",
                "submitted_at",
                "metadata",
            ])
            .map_err(|e| IndexerError::FileWriteError {
                path: output_path.to_path_buf(),
                message: format!("Failed to write CSV header: {}", e),
            })?;

        // Write records
        for record in &records {
            let metadata_str = record
                .metadata
                .as_ref()
                .and_then(|m| serde_json::to_string(m).ok())
                .unwrap_or_default();

            writer
                .write_record(&[
                    record.id.map(|i| i.to_string()).unwrap_or_default(),
                    record.url.clone(),
                    record.api.to_string(),
                    record.action.to_string(),
                    record.status.to_string(),
                    record
                        .response_code
                        .map(|c| c.to_string())
                        .unwrap_or_default(),
                    record.response_message.clone().unwrap_or_default(),
                    record.submitted_at.to_rfc3339(),
                    metadata_str,
                ])
                .map_err(|e| IndexerError::FileWriteError {
                    path: output_path.to_path_buf(),
                    message: format!("Failed to write CSV record: {}", e),
                })?;
        }

        writer.flush().map_err(|e| IndexerError::FileWriteError {
            path: output_path.to_path_buf(),
            message: format!("Failed to flush CSV writer: {}", e),
        })?;

        info!(
            "Successfully exported {} records to CSV",
            records.len()
        );
        Ok(records.len())
    }

    /// Export submission history to JSON format
    ///
    /// # Arguments
    ///
    /// * `output_path` - Path to the output JSON file
    /// * `filters` - Optional filters to apply
    ///
    /// # Returns
    ///
    /// The number of records exported or an IndexerError
    ///
    /// # Example
    ///
    /// ```no_run
    /// use indexer_cli::services::history_manager::HistoryManager;
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = HistoryManager::new(Path::new("./data/indexer.db"), 90)?;
    ///
    /// let count = manager.export_to_json(
    ///     Path::new("./exports/submissions.json"),
    ///     None
    /// )?;
    /// println!("Exported {} records to JSON", count);
    /// # Ok(())
    /// # }
    /// ```
    pub fn export_to_json(
        &self,
        output_path: &Path,
        filters: Option<HistoryFilters>,
    ) -> Result<usize, IndexerError> {
        info!("Exporting submissions to JSON: {}", output_path.display());

        let conn = self.get_connection()?;
        let db_filters = filters
            .map(|f| f.to_db_filters())
            .unwrap_or_else(SubmissionFilters::new);

        let records = list_submissions(&conn, &db_filters)?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                IndexerError::DirectoryCreationFailed {
                    path: parent.to_path_buf(),
                    message: e.to_string(),
                }
            })?;
        }

        // Write JSON
        let json =
            serde_json::to_string_pretty(&records).map_err(|e| {
                IndexerError::JsonSerializationError {
                    message: e.to_string(),
                }
            })?;

        std::fs::write(output_path, json).map_err(|e| IndexerError::FileWriteError {
            path: output_path.to_path_buf(),
            message: e.to_string(),
        })?;

        info!(
            "Successfully exported {} records to JSON",
            records.len()
        );
        Ok(records.len())
    }

    /// Get the database path
    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    /// Get the retention period in days
    pub fn retention_days(&self) -> i64 {
        self.retention_days
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_manager() -> (HistoryManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let manager = HistoryManager::new(&db_path, 90).unwrap();
        (manager, temp_dir)
    }

    #[test]
    fn test_new_history_manager() {
        let (manager, _temp_dir) = setup_test_manager();
        assert_eq!(manager.retention_days(), 90);
    }

    #[test]
    fn test_record_submission() {
        let (manager, _temp_dir) = setup_test_manager();

        let id = manager
            .record_submission(
                "https://example.com/test",
                ApiType::Google,
                ActionType::UrlUpdated,
                SubmissionStatus::Success,
                Some((200, "OK".to_string())),
            )
            .unwrap();

        assert!(id > 0);
    }

    #[test]
    fn test_record_batch_submissions() {
        let (manager, _temp_dir) = setup_test_manager();

        let records = vec![
            SubmissionRecord::builder()
                .url("https://example.com/page1")
                .api(ApiType::Google)
                .action(ActionType::UrlUpdated)
                .status(SubmissionStatus::Success)
                .build()
                .unwrap(),
            SubmissionRecord::builder()
                .url("https://example.com/page2")
                .api(ApiType::IndexNow)
                .action(ActionType::UrlUpdated)
                .status(SubmissionStatus::Success)
                .build()
                .unwrap(),
        ];

        let count = manager.record_batch_submissions(&records).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_is_url_submitted() {
        let (manager, _temp_dir) = setup_test_manager();

        manager
            .record_submission(
                "https://example.com/test",
                ApiType::Google,
                ActionType::UrlUpdated,
                SubmissionStatus::Success,
                None,
            )
            .unwrap();

        let one_hour_ago = Utc::now() - Duration::hours(1);
        let submitted = manager
            .is_url_submitted("https://example.com/test", ApiType::Google, one_hour_ago)
            .unwrap();
        assert!(submitted);

        let one_hour_future = Utc::now() + Duration::hours(1);
        let not_submitted = manager
            .is_url_submitted("https://example.com/test", ApiType::Google, one_hour_future)
            .unwrap();
        assert!(!not_submitted);
    }

    #[test]
    fn test_get_submission_history() {
        let (manager, _temp_dir) = setup_test_manager();

        manager
            .record_submission(
                "https://example.com/test",
                ApiType::Google,
                ActionType::UrlUpdated,
                SubmissionStatus::Success,
                None,
            )
            .unwrap();

        let history = manager
            .get_submission_history("https://example.com/test", ApiType::Google)
            .unwrap();

        assert_eq!(history.len(), 1);
        assert_eq!(history[0].url, "https://example.com/test");
    }

    #[test]
    fn test_get_statistics() {
        let (manager, _temp_dir) = setup_test_manager();

        // Insert test data
        for i in 1..=10 {
            manager
                .record_submission(
                    &format!("https://example.com/page{}", i),
                    if i % 2 == 0 {
                        ApiType::Google
                    } else {
                        ApiType::IndexNow
                    },
                    ActionType::UrlUpdated,
                    if i % 3 == 0 {
                        SubmissionStatus::Failed
                    } else {
                        SubmissionStatus::Success
                    },
                    None,
                )
                .unwrap();
        }

        let stats = manager.get_statistics(None).unwrap();
        assert_eq!(stats.total_submissions, 10);
        assert_eq!(stats.successful, 7);
        assert_eq!(stats.failed, 3);
        assert_eq!(stats.google_count, 5);
        assert_eq!(stats.indexnow_count, 5);
        assert!(stats.last_submission.is_some());
    }

    #[test]
    fn test_list_recent_submissions() {
        let (manager, _temp_dir) = setup_test_manager();

        for i in 1..=5 {
            manager
                .record_submission(
                    &format!("https://example.com/page{}", i),
                    ApiType::Google,
                    ActionType::UrlUpdated,
                    SubmissionStatus::Success,
                    None,
                )
                .unwrap();
        }

        let recent = manager.list_recent_submissions(3, None).unwrap();
        assert_eq!(recent.len(), 3);
    }

    #[test]
    fn test_export_to_json() {
        let (manager, temp_dir) = setup_test_manager();

        manager
            .record_submission(
                "https://example.com/test",
                ApiType::Google,
                ActionType::UrlUpdated,
                SubmissionStatus::Success,
                None,
            )
            .unwrap();

        let output_path = temp_dir.path().join("export.json");
        let count = manager.export_to_json(&output_path, None).unwrap();

        assert_eq!(count, 1);
        assert!(output_path.exists());
    }

    #[test]
    fn test_clean_old_records() {
        let (manager, _temp_dir) = setup_test_manager();

        manager
            .record_submission(
                "https://example.com/test",
                ApiType::Google,
                ActionType::UrlUpdated,
                SubmissionStatus::Success,
                None,
            )
            .unwrap();

        // Should delete nothing (records are recent)
        let deleted = manager.clean_old_records(0).unwrap();
        assert_eq!(deleted, 0);
    }
}
