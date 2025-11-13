//! Database module for submission history tracking.
//!
//! This module provides all database-related functionality for storing and
//! retrieving URL submission history records. It includes:
//!
//! - **schema**: Database initialization, table creation, and migrations
//! - **models**: Data models for submission records (SubmissionRecord, ApiType, etc.)
//! - **queries**: Query functions for inserting, retrieving, and managing records
//!
//! # Example
//!
//! ```no_run
//! use indexer_cli::database::{
//!     schema::init_database,
//!     models::{SubmissionRecord, ApiType, ActionType, SubmissionStatus},
//!     queries::{insert_submission, get_submission_by_url, SubmissionFilters},
//! };
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize database
//! let db_path = Path::new("./data/indexer.db");
//! let conn = init_database(db_path)?;
//!
//! // Create a submission record
//! let record = SubmissionRecord::builder()
//!     .url("https://placeholder.test/page")
//!     .api(ApiType::Google)
//!     .action(ActionType::UrlUpdated)
//!     .status(SubmissionStatus::Success)
//!     .response_code(200)
//!     .build()?;
//!
//! // Insert the record
//! let id = insert_submission(&conn, &record)?;
//! println!("Inserted record with ID: {}", id);
//!
//! // Query the record
//! let retrieved = get_submission_by_url(&conn, "https://placeholder.test/page", ApiType::Google)?;
//! if let Some(record) = retrieved {
//!     println!("Found record: {:?}", record);
//! }
//! # Ok(())
//! # }
//! ```

pub mod models;
pub mod queries;
pub mod schema;

// Re-export commonly used types and functions for convenience

// Models
pub use models::{
    ActionType, ApiType, SubmissionRecord, SubmissionRecordBuilder, SubmissionStatus,
};

// Schema functions
pub use schema::{create_tables, get_schema_version, init_database, migrate_database};

// Query functions
pub use queries::{
    check_url_submitted, count_submissions, delete_old_submissions, get_submission_by_url,
    get_submissions_stats, insert_submission, list_submissions, SubmissionFilters, SubmissionStats,
};
