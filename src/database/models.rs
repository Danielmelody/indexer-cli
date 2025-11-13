//! Database models for submission history tracking.
//!
//! This module defines the data models used to store and retrieve
//! submission history records from the SQLite database.

use chrono::{DateTime, Utc};
use rusqlite::Row;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Type of indexing API used for submission
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiType {
    /// Google Indexing API
    Google,
    /// IndexNow API
    IndexNow,
}

impl ApiType {
    /// Convert to database string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            ApiType::Google => "google",
            ApiType::IndexNow => "indexnow",
        }
    }

    /// Parse from database string representation
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "google" => Ok(ApiType::Google),
            "indexnow" => Ok(ApiType::IndexNow),
            _ => Err(format!("Invalid API type: {}", s)),
        }
    }
}

impl fmt::Display for ApiType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Type of action performed on the URL
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActionType {
    /// URL was updated/added
    UrlUpdated,
    /// URL was deleted
    UrlDeleted,
}

impl ActionType {
    /// Convert to database string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            ActionType::UrlUpdated => "URL_UPDATED",
            ActionType::UrlDeleted => "URL_DELETED",
        }
    }

    /// Parse from database string representation
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "URL_UPDATED" => Ok(ActionType::UrlUpdated),
            "URL_DELETED" => Ok(ActionType::UrlDeleted),
            _ => Err(format!("Invalid action type: {}", s)),
        }
    }
}

impl fmt::Display for ActionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Status of the submission
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SubmissionStatus {
    /// Submission succeeded
    Success,
    /// Submission failed
    Failed,
}

impl SubmissionStatus {
    /// Convert to database string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            SubmissionStatus::Success => "success",
            SubmissionStatus::Failed => "failed",
        }
    }

    /// Parse from database string representation
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "success" => Ok(SubmissionStatus::Success),
            "failed" => Ok(SubmissionStatus::Failed),
            _ => Err(format!("Invalid submission status: {}", s)),
        }
    }
}

impl fmt::Display for SubmissionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A record of a URL submission to an indexing API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionRecord {
    /// Database record ID (None for new records)
    pub id: Option<i64>,
    /// The URL that was submitted
    pub url: String,
    /// The API used for submission
    pub api: ApiType,
    /// The action performed
    pub action: ActionType,
    /// The status of the submission
    pub status: SubmissionStatus,
    /// HTTP response code (if available)
    pub response_code: Option<i32>,
    /// Response message from the API
    pub response_message: Option<String>,
    /// Timestamp when the submission was made
    pub submitted_at: DateTime<Utc>,
    /// Additional metadata as JSON
    pub metadata: Option<serde_json::Value>,
}

impl SubmissionRecord {
    /// Create a new submission record builder
    pub fn builder() -> SubmissionRecordBuilder {
        SubmissionRecordBuilder::default()
    }

    /// Try to parse a SubmissionRecord from a database row
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        let metadata_str: Option<String> = row.get(8)?;
        let metadata = metadata_str
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok());

        Ok(SubmissionRecord {
            id: row.get(0)?,
            url: row.get(1)?,
            api: ApiType::from_str(&row.get::<_, String>(2)?).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    2,
                    rusqlite::types::Type::Text,
                    Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
                )
            })?,
            action: ActionType::from_str(&row.get::<_, String>(3)?).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    3,
                    rusqlite::types::Type::Text,
                    Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
                )
            })?,
            status: SubmissionStatus::from_str(&row.get::<_, String>(4)?).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    4,
                    rusqlite::types::Type::Text,
                    Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
                )
            })?,
            response_code: row.get(5)?,
            response_message: row.get(6)?,
            submitted_at: row.get(7)?,
            metadata,
        })
    }
}

/// Builder for creating SubmissionRecord instances
#[derive(Default)]
pub struct SubmissionRecordBuilder {
    url: Option<String>,
    api: Option<ApiType>,
    action: Option<ActionType>,
    status: Option<SubmissionStatus>,
    response_code: Option<i32>,
    response_message: Option<String>,
    submitted_at: Option<DateTime<Utc>>,
    metadata: Option<serde_json::Value>,
}

impl SubmissionRecordBuilder {
    /// Set the URL
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Set the API type
    pub fn api(mut self, api: ApiType) -> Self {
        self.api = Some(api);
        self
    }

    /// Set the action type
    pub fn action(mut self, action: ActionType) -> Self {
        self.action = Some(action);
        self
    }

    /// Set the status
    pub fn status(mut self, status: SubmissionStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Set the response code
    pub fn response_code(mut self, code: i32) -> Self {
        self.response_code = Some(code);
        self
    }

    /// Set the response message
    pub fn response_message(mut self, message: impl Into<String>) -> Self {
        self.response_message = Some(message.into());
        self
    }

    /// Set the submitted timestamp
    pub fn submitted_at(mut self, timestamp: DateTime<Utc>) -> Self {
        self.submitted_at = Some(timestamp);
        self
    }

    /// Set the metadata
    pub fn metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Build the SubmissionRecord
    pub fn build(self) -> Result<SubmissionRecord, String> {
        Ok(SubmissionRecord {
            id: None,
            url: self.url.ok_or("URL is required")?,
            api: self.api.ok_or("API type is required")?,
            action: self.action.ok_or("Action type is required")?,
            status: self.status.ok_or("Status is required")?,
            response_code: self.response_code,
            response_message: self.response_message,
            submitted_at: self.submitted_at.unwrap_or_else(Utc::now),
            metadata: self.metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_type_conversion() {
        assert_eq!(ApiType::Google.as_str(), "google");
        assert_eq!(ApiType::IndexNow.as_str(), "indexnow");
        assert_eq!(ApiType::from_str("google").unwrap(), ApiType::Google);
        assert_eq!(ApiType::from_str("GOOGLE").unwrap(), ApiType::Google);
        assert_eq!(ApiType::from_str("indexnow").unwrap(), ApiType::IndexNow);
        assert!(ApiType::from_str("invalid").is_err());
    }

    #[test]
    fn test_action_type_conversion() {
        assert_eq!(ActionType::UrlUpdated.as_str(), "URL_UPDATED");
        assert_eq!(ActionType::UrlDeleted.as_str(), "URL_DELETED");
        assert_eq!(
            ActionType::from_str("URL_UPDATED").unwrap(),
            ActionType::UrlUpdated
        );
        assert_eq!(
            ActionType::from_str("URL_DELETED").unwrap(),
            ActionType::UrlDeleted
        );
        assert!(ActionType::from_str("invalid").is_err());
    }

    #[test]
    fn test_submission_status_conversion() {
        assert_eq!(SubmissionStatus::Success.as_str(), "success");
        assert_eq!(SubmissionStatus::Failed.as_str(), "failed");
        assert_eq!(
            SubmissionStatus::from_str("success").unwrap(),
            SubmissionStatus::Success
        );
        assert_eq!(
            SubmissionStatus::from_str("SUCCESS").unwrap(),
            SubmissionStatus::Success
        );
        assert_eq!(
            SubmissionStatus::from_str("failed").unwrap(),
            SubmissionStatus::Failed
        );
        assert!(SubmissionStatus::from_str("invalid").is_err());
    }

    #[test]
    fn test_submission_record_builder() {
        let record = SubmissionRecord::builder()
            .url("https://placeholder.test/page")
            .api(ApiType::Google)
            .action(ActionType::UrlUpdated)
            .status(SubmissionStatus::Success)
            .response_code(200)
            .response_message("OK")
            .build()
            .unwrap();

        assert_eq!(record.url, "https://placeholder.test/page");
        assert_eq!(record.api, ApiType::Google);
        assert_eq!(record.action, ActionType::UrlUpdated);
        assert_eq!(record.status, SubmissionStatus::Success);
        assert_eq!(record.response_code, Some(200));
        assert_eq!(record.response_message, Some("OK".to_string()));
        assert!(record.id.is_none());
    }

    #[test]
    fn test_submission_record_builder_missing_fields() {
        let result = SubmissionRecord::builder()
            .url("https://placeholder.test/page")
            .build();
        assert!(result.is_err());
    }
}
