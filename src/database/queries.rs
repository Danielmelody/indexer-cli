//! Database query operations for submission history.
//!
//! This module provides functions for inserting, querying, and managing
//! submission history records in the SQLite database.

use super::models::{ApiType, SubmissionRecord, SubmissionStatus};
use crate::types::IndexerError;
use chrono::{DateTime, Duration, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use tracing::{debug, info};

/// Filters for querying submission records
#[derive(Debug, Default, Clone)]
pub struct SubmissionFilters {
    /// Filter by URL pattern (supports SQL LIKE wildcards)
    pub url_pattern: Option<String>,
    /// Filter by API type
    pub api: Option<ApiType>,
    /// Filter by status
    pub status: Option<SubmissionStatus>,
    /// Filter by submissions after this timestamp
    pub after: Option<DateTime<Utc>>,
    /// Filter by submissions before this timestamp
    pub before: Option<DateTime<Utc>>,
    /// Maximum number of records to return
    pub limit: Option<usize>,
    /// Number of records to skip (for pagination)
    pub offset: Option<usize>,
}

impl SubmissionFilters {
    /// Create a new empty filter set
    pub fn new() -> Self {
        Self::default()
    }

    /// Set URL pattern filter
    pub fn url_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.url_pattern = Some(pattern.into());
        self
    }

    /// Set API type filter
    pub fn api(mut self, api: ApiType) -> Self {
        self.api = Some(api);
        self
    }

    /// Set status filter
    pub fn status(mut self, status: SubmissionStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Set after timestamp filter
    pub fn after(mut self, timestamp: DateTime<Utc>) -> Self {
        self.after = Some(timestamp);
        self
    }

    /// Set before timestamp filter
    pub fn before(mut self, timestamp: DateTime<Utc>) -> Self {
        self.before = Some(timestamp);
        self
    }

    /// Set limit
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set offset
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Build the WHERE clause and parameters for the filters
    fn build_where_clause(&self) -> (String, Vec<Box<dyn rusqlite::ToSql>>) {
        let mut conditions = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref pattern) = self.url_pattern {
            conditions.push("url LIKE ?".to_string());
            params.push(Box::new(pattern.clone()));
        }

        if let Some(api) = self.api {
            conditions.push("api = ?".to_string());
            params.push(Box::new(api.as_str().to_string()));
        }

        if let Some(status) = self.status {
            conditions.push("status = ?".to_string());
            params.push(Box::new(status.as_str().to_string()));
        }

        if let Some(after) = self.after {
            conditions.push("submitted_at > ?".to_string());
            params.push(Box::new(after));
        }

        if let Some(before) = self.before {
            conditions.push("submitted_at < ?".to_string());
            params.push(Box::new(before));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        (where_clause, params)
    }
}

/// Insert a submission record into the database.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `record` - The submission record to insert
///
/// # Returns
///
/// The ID of the inserted record or an IndexerError
pub fn insert_submission(
    conn: &Connection,
    record: &SubmissionRecord,
) -> Result<i64, IndexerError> {
    debug!("Inserting submission record for URL: {}", record.url);

    let metadata_str = record
        .metadata
        .as_ref()
        .and_then(|m| serde_json::to_string(m).ok());

    let result = conn.execute(
        r#"
        INSERT INTO submission_history
        (url, api, action, status, response_code, response_message, submitted_at, metadata)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
        params![
            record.url,
            record.api.as_str(),
            record.action.as_str(),
            record.status.as_str(),
            record.response_code,
            record.response_message,
            record.submitted_at,
            metadata_str,
        ],
    );

    match result {
        Ok(_) => {
            let id = conn.last_insert_rowid();
            debug!("Inserted submission record with ID: {}", id);
            Ok(id)
        }
        Err(e) => Err(IndexerError::DatabaseQueryFailed {
            message: format!("Failed to insert submission record: {}", e),
        }),
    }
}

/// Get the most recent submission record for a URL and API combination.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `url` - The URL to search for
/// * `api` - The API type to filter by
///
/// # Returns
///
/// The most recent submission record or None if not found
pub fn get_submission_by_url(
    conn: &Connection,
    url: &str,
    api: ApiType,
) -> Result<Option<SubmissionRecord>, IndexerError> {
    debug!("Querying submission for URL: {} and API: {}", url, api);

    let result = conn
        .query_row(
            r#"
            SELECT id, url, api, action, status, response_code, response_message, submitted_at, metadata
            FROM submission_history
            WHERE url = ?1 AND api = ?2
            ORDER BY submitted_at DESC
            LIMIT 1
            "#,
            params![url, api.as_str()],
            |row| SubmissionRecord::from_row(row),
        )
        .optional()
        .map_err(|e| IndexerError::DatabaseQueryFailed {
            message: format!("Failed to query submission by URL: {}", e),
        })?;

    Ok(result)
}

/// List submission records with optional filtering.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `filters` - Optional filters to apply
///
/// # Returns
///
/// A vector of submission records matching the filters
pub fn list_submissions(
    conn: &Connection,
    filters: &SubmissionFilters,
) -> Result<Vec<SubmissionRecord>, IndexerError> {
    debug!("Listing submissions with filters: {:?}", filters);

    let (where_clause, params) = filters.build_where_clause();

    let mut query = format!(
        r#"
        SELECT id, url, api, action, status, response_code, response_message, submitted_at, metadata
        FROM submission_history
        {}
        ORDER BY submitted_at DESC
        "#,
        where_clause
    );

    if let Some(limit) = filters.limit {
        query.push_str(&format!(" LIMIT {}", limit));
    }

    if let Some(offset) = filters.offset {
        query.push_str(&format!(" OFFSET {}", offset));
    }

    let mut stmt = conn
        .prepare(&query)
        .map_err(|e| IndexerError::DatabaseQueryFailed {
            message: format!("Failed to prepare query: {}", e),
        })?;

    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let records = stmt
        .query_map(param_refs.as_slice(), |row| SubmissionRecord::from_row(row))
        .map_err(|e| IndexerError::DatabaseQueryFailed {
            message: format!("Failed to execute query: {}", e),
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| IndexerError::DatabaseQueryFailed {
            message: format!("Failed to fetch records: {}", e),
        })?;

    debug!("Found {} submission records", records.len());
    Ok(records)
}

/// Statistics about submissions
#[derive(Debug, Clone)]
pub struct SubmissionStats {
    /// Total number of submissions
    pub total: i64,
    /// Number of successful submissions
    pub success: i64,
    /// Number of failed submissions
    pub failed: i64,
    /// Number of Google API submissions
    pub google: i64,
    /// Number of IndexNow API submissions
    pub indexnow: i64,
    /// Most recent submission timestamp
    pub last_submission: Option<DateTime<Utc>>,
}

/// Get statistics about submission history.
///
/// # Arguments
///
/// * `conn` - Database connection
///
/// # Returns
///
/// Statistics about the submission history
pub fn get_submissions_stats(conn: &Connection) -> Result<SubmissionStats, IndexerError> {
    debug!("Getting submission statistics");

    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM submission_history", [], |row| {
            row.get(0)
        })
        .map_err(|e| IndexerError::DatabaseQueryFailed {
            message: format!("Failed to get total count: {}", e),
        })?;

    let success: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM submission_history WHERE status = 'success'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| IndexerError::DatabaseQueryFailed {
            message: format!("Failed to get success count: {}", e),
        })?;

    let failed: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM submission_history WHERE status = 'failed'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| IndexerError::DatabaseQueryFailed {
            message: format!("Failed to get failed count: {}", e),
        })?;

    let google: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM submission_history WHERE api = 'google'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| IndexerError::DatabaseQueryFailed {
            message: format!("Failed to get Google count: {}", e),
        })?;

    let indexnow: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM submission_history WHERE api = 'indexnow'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| IndexerError::DatabaseQueryFailed {
            message: format!("Failed to get IndexNow count: {}", e),
        })?;

    let last_submission: Option<DateTime<Utc>> = conn
        .query_row(
            "SELECT MAX(submitted_at) FROM submission_history",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| IndexerError::DatabaseQueryFailed {
            message: format!("Failed to get last submission time: {}", e),
        })?
        .flatten();

    let stats = SubmissionStats {
        total,
        success,
        failed,
        google,
        indexnow,
        last_submission,
    };

    debug!(
        "Submission statistics: total={}, success={}, failed={}, google={}, indexnow={}",
        stats.total, stats.success, stats.failed, stats.google, stats.indexnow
    );

    Ok(stats)
}

/// Delete old submission records older than a specified number of days.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `days` - Delete records older than this many days
///
/// # Returns
///
/// The number of records deleted
pub fn delete_old_submissions(conn: &Connection, days: i64) -> Result<usize, IndexerError> {
    let cutoff = Utc::now() - Duration::days(days);
    info!(
        "Deleting submissions older than {} days (before {})",
        days, cutoff
    );

    let deleted = conn
        .execute(
            "DELETE FROM submission_history WHERE submitted_at < ?1",
            params![cutoff],
        )
        .map_err(|e| IndexerError::DatabaseQueryFailed {
            message: format!("Failed to delete old submissions: {}", e),
        })?;

    info!("Deleted {} old submission records", deleted);
    Ok(deleted)
}

/// Check if a URL has been submitted to a specific API within a time window.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `url` - The URL to check
/// * `api` - The API type to check
/// * `since` - Check for submissions after this timestamp
///
/// # Returns
///
/// True if the URL was submitted since the given timestamp, false otherwise
pub fn check_url_submitted(
    conn: &Connection,
    url: &str,
    api: ApiType,
    since: DateTime<Utc>,
) -> Result<bool, IndexerError> {
    debug!(
        "Checking if URL {} was submitted to {} since {}",
        url, api, since
    );

    let exists: bool = conn
        .query_row(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM submission_history
                WHERE url = ?1 AND api = ?2 AND submitted_at > ?3
            )
            "#,
            params![url, api.as_str(), since],
            |row| row.get(0),
        )
        .map_err(|e| IndexerError::DatabaseQueryFailed {
            message: format!("Failed to check if URL was submitted: {}", e),
        })?;

    debug!("URL submission check result: {}", exists);
    Ok(exists)
}

/// Count submissions matching the given filters.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `filters` - Filters to apply
///
/// # Returns
///
/// The number of records matching the filters
pub fn count_submissions(
    conn: &Connection,
    filters: &SubmissionFilters,
) -> Result<i64, IndexerError> {
    let (where_clause, params) = filters.build_where_clause();

    let query = format!("SELECT COUNT(*) FROM submission_history {}", where_clause);

    let mut stmt = conn
        .prepare(&query)
        .map_err(|e| IndexerError::DatabaseQueryFailed {
            message: format!("Failed to prepare count query: {}", e),
        })?;

    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let count: i64 = stmt
        .query_row(param_refs.as_slice(), |row| row.get(0))
        .map_err(|e| IndexerError::DatabaseQueryFailed {
            message: format!("Failed to execute count query: {}", e),
        })?;

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::models::{ActionType, SubmissionRecord};
    use crate::database::schema::create_tables;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::database::schema::create_schema_version_table(&conn).unwrap();
        create_tables(&conn).unwrap();
        conn
    }

    #[test]
    fn test_insert_and_get_submission() {
        let conn = setup_test_db();

        let record = SubmissionRecord::builder()
            .url("https://placeholder.test/page1")
            .api(ApiType::Google)
            .action(ActionType::UrlUpdated)
            .status(SubmissionStatus::Success)
            .response_code(200)
            .build()
            .unwrap();

        let id = insert_submission(&conn, &record).unwrap();
        assert!(id > 0);

        let retrieved =
            get_submission_by_url(&conn, "https://placeholder.test/page1", ApiType::Google)
                .unwrap()
                .unwrap();

        assert_eq!(retrieved.url, record.url);
        assert_eq!(retrieved.api, record.api);
        assert_eq!(retrieved.status, record.status);
    }

    #[test]
    fn test_list_submissions_with_filters() {
        let conn = setup_test_db();

        // Insert test records
        for i in 1..=5 {
            let record = SubmissionRecord::builder()
                .url(format!("https://placeholder.test/page{}", i))
                .api(if i % 2 == 0 {
                    ApiType::Google
                } else {
                    ApiType::IndexNow
                })
                .action(ActionType::UrlUpdated)
                .status(if i % 3 == 0 {
                    SubmissionStatus::Failed
                } else {
                    SubmissionStatus::Success
                })
                .build()
                .unwrap();
            insert_submission(&conn, &record).unwrap();
        }

        // Test filtering by API
        let filters = SubmissionFilters::new().api(ApiType::Google);
        let results = list_submissions(&conn, &filters).unwrap();
        assert_eq!(results.len(), 2);

        // Test filtering by status
        let filters = SubmissionFilters::new().status(SubmissionStatus::Success);
        let results = list_submissions(&conn, &filters).unwrap();
        assert_eq!(results.len(), 4);

        // Test limit
        let filters = SubmissionFilters::new().limit(3);
        let results = list_submissions(&conn, &filters).unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_get_submissions_stats() {
        let conn = setup_test_db();

        // Insert test records
        for i in 1..=10 {
            let record = SubmissionRecord::builder()
                .url(format!("https://placeholder.test/page{}", i))
                .api(if i <= 6 {
                    ApiType::Google
                } else {
                    ApiType::IndexNow
                })
                .action(ActionType::UrlUpdated)
                .status(if i <= 7 {
                    SubmissionStatus::Success
                } else {
                    SubmissionStatus::Failed
                })
                .build()
                .unwrap();
            insert_submission(&conn, &record).unwrap();
        }

        let stats = get_submissions_stats(&conn).unwrap();
        assert_eq!(stats.total, 10);
        assert_eq!(stats.success, 7);
        assert_eq!(stats.failed, 3);
        assert_eq!(stats.google, 6);
        assert_eq!(stats.indexnow, 4);
        assert!(stats.last_submission.is_some());
    }

    #[test]
    fn test_check_url_submitted() {
        let conn = setup_test_db();

        let record = SubmissionRecord::builder()
            .url("https://placeholder.test/test")
            .api(ApiType::Google)
            .action(ActionType::UrlUpdated)
            .status(SubmissionStatus::Success)
            .build()
            .unwrap();

        insert_submission(&conn, &record).unwrap();

        // Check within time window
        let since = Utc::now() - Duration::hours(1);
        let exists = check_url_submitted(
            &conn,
            "https://placeholder.test/test",
            ApiType::Google,
            since,
        )
        .unwrap();
        assert!(exists);

        // Check outside time window
        let since = Utc::now() + Duration::hours(1);
        let exists = check_url_submitted(
            &conn,
            "https://placeholder.test/test",
            ApiType::Google,
            since,
        )
        .unwrap();
        assert!(!exists);

        // Check different API
        let since = Utc::now() - Duration::hours(1);
        let exists = check_url_submitted(
            &conn,
            "https://placeholder.test/test",
            ApiType::IndexNow,
            since,
        )
        .unwrap();
        assert!(!exists);
    }

    #[test]
    fn test_delete_old_submissions() {
        let conn = setup_test_db();

        // Insert records
        for i in 1..=5 {
            let record = SubmissionRecord::builder()
                .url(format!("https://placeholder.test/page{}", i))
                .api(ApiType::Google)
                .action(ActionType::UrlUpdated)
                .status(SubmissionStatus::Success)
                .build()
                .unwrap();
            insert_submission(&conn, &record).unwrap();
        }

        // Delete records older than 0 days (should delete nothing since all are recent)
        let deleted = delete_old_submissions(&conn, 0).unwrap();
        assert_eq!(deleted, 0);

        let stats = get_submissions_stats(&conn).unwrap();
        assert_eq!(stats.total, 5);
    }

    #[test]
    fn test_count_submissions() {
        let conn = setup_test_db();

        // Insert test records
        for i in 1..=10 {
            let record = SubmissionRecord::builder()
                .url(format!("https://placeholder.test/page{}", i))
                .api(ApiType::Google)
                .action(ActionType::UrlUpdated)
                .status(SubmissionStatus::Success)
                .build()
                .unwrap();
            insert_submission(&conn, &record).unwrap();
        }

        let filters = SubmissionFilters::new();
        let count = count_submissions(&conn, &filters).unwrap();
        assert_eq!(count, 10);

        let filters = SubmissionFilters::new().api(ApiType::Google);
        let count = count_submissions(&conn, &filters).unwrap();
        assert_eq!(count, 10);
    }
}
