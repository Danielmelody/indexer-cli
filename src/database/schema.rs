//! Database schema initialization and migration.
//!
//! This module handles creating and maintaining the SQLite database schema
//! for storing submission history records.

use crate::types::IndexerError;
use rusqlite::Connection;
use std::path::Path;
use tracing::{debug, info};

/// Current database schema version
const SCHEMA_VERSION: i32 = 1;

/// SQL to create the submission_history table
const CREATE_SUBMISSION_HISTORY_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS submission_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL,
    api TEXT NOT NULL,
    action TEXT NOT NULL,
    status TEXT NOT NULL,
    response_code INTEGER,
    response_message TEXT,
    submitted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    metadata TEXT
)
"#;

/// SQL to create indexes on the submission_history table
const CREATE_INDEXES: [&str; 4] = [
    "CREATE INDEX IF NOT EXISTS idx_url ON submission_history(url)",
    "CREATE INDEX IF NOT EXISTS idx_api ON submission_history(api)",
    "CREATE INDEX IF NOT EXISTS idx_status ON submission_history(status)",
    "CREATE INDEX IF NOT EXISTS idx_submitted_at ON submission_history(submitted_at)",
];

/// SQL to create the schema_version table for tracking migrations
const CREATE_SCHEMA_VERSION_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS schema_version (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    version INTEGER NOT NULL,
    applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
)
"#;

/// Initialize the database and create all necessary tables and indexes.
///
/// This function will:
/// 1. Create the database file if it doesn't exist
/// 2. Create the schema_version table
/// 3. Create the submission_history table
/// 4. Create all necessary indexes
/// 5. Run any pending migrations
///
/// # Arguments
///
/// * `db_path` - Path to the SQLite database file
///
/// # Returns
///
/// A Result containing the database Connection or an IndexerError
///
/// # Example
///
/// ```no_run
/// use indexer_cli::database::schema::init_database;
/// use std::path::Path;
///
/// let db_path = Path::new("./data/indexer.db");
/// let conn = init_database(db_path).expect("Failed to initialize database");
/// ```
pub fn init_database(db_path: &Path) -> Result<Connection, IndexerError> {
    // Create parent directories if they don't exist
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| IndexerError::DirectoryCreationFailed {
            path: parent.to_path_buf(),
            message: e.to_string(),
        })?;
    }

    debug!("Opening database at: {}", db_path.display());

    // Open or create the database
    let conn = Connection::open(db_path).map_err(|e| IndexerError::DatabaseConnectionFailed {
        message: format!("Failed to open database at {}: {}", db_path.display(), e),
    })?;

    // Enable foreign key support
    conn.execute("PRAGMA foreign_keys = ON", [])
        .map_err(|e| IndexerError::DatabaseQueryFailed {
            message: format!("Failed to enable foreign keys: {}", e),
        })?;

    // Enable WAL mode for better concurrency
    conn.execute("PRAGMA journal_mode = WAL", [])
        .map_err(|e| IndexerError::DatabaseQueryFailed {
            message: format!("Failed to enable WAL mode: {}", e),
        })?;

    info!("Initializing database schema");

    // Create schema version table
    create_schema_version_table(&conn)?;

    // Create all tables
    create_tables(&conn)?;

    // Run migrations if needed
    migrate_database(&conn)?;

    info!("Database initialization complete");

    Ok(conn)
}

/// Create the schema_version table
pub(crate) fn create_schema_version_table(conn: &Connection) -> Result<(), IndexerError> {
    debug!("Creating schema_version table");

    conn.execute(CREATE_SCHEMA_VERSION_TABLE, [])
        .map_err(|e| IndexerError::DatabaseQueryFailed {
            message: format!("Failed to create schema_version table: {}", e),
        })?;

    // Check if we need to initialize the version
    let version_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_version WHERE id = 1)",
            [],
            |row| row.get(0),
        )
        .map_err(|e| IndexerError::DatabaseQueryFailed {
            message: format!("Failed to check schema version: {}", e),
        })?;

    if !version_exists {
        debug!("Initializing schema version to {}", SCHEMA_VERSION);
        conn.execute(
            "INSERT INTO schema_version (id, version) VALUES (1, ?1)",
            [SCHEMA_VERSION],
        )
        .map_err(|e| IndexerError::DatabaseQueryFailed {
            message: format!("Failed to initialize schema version: {}", e),
        })?;
    }

    Ok(())
}

/// Create all database tables and indexes.
///
/// # Arguments
///
/// * `conn` - Database connection
///
/// # Returns
///
/// A Result indicating success or an IndexerError
pub fn create_tables(conn: &Connection) -> Result<(), IndexerError> {
    debug!("Creating submission_history table");

    // Create submission_history table
    conn.execute(CREATE_SUBMISSION_HISTORY_TABLE, [])
        .map_err(|e| IndexerError::DatabaseQueryFailed {
            message: format!("Failed to create submission_history table: {}", e),
        })?;

    // Create all indexes
    for (i, index_sql) in CREATE_INDEXES.iter().enumerate() {
        debug!("Creating index {}/{}", i + 1, CREATE_INDEXES.len());
        conn.execute(index_sql, [])
            .map_err(|e| IndexerError::DatabaseQueryFailed {
                message: format!("Failed to create index: {}", e),
            })?;
    }

    info!("All tables and indexes created successfully");

    Ok(())
}

/// Run database migrations to upgrade the schema.
///
/// This function checks the current schema version and applies any
/// necessary migrations to bring it up to the latest version.
///
/// # Arguments
///
/// * `conn` - Database connection
///
/// # Returns
///
/// A Result indicating success or an IndexerError
pub fn migrate_database(conn: &Connection) -> Result<(), IndexerError> {
    // Get current schema version
    let current_version: i32 = conn
        .query_row(
            "SELECT version FROM schema_version WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .map_err(|e| IndexerError::DatabaseQueryFailed {
            message: format!("Failed to get schema version: {}", e),
        })?;

    debug!(
        "Current schema version: {}, target version: {}",
        current_version, SCHEMA_VERSION
    );

    if current_version < SCHEMA_VERSION {
        info!(
            "Migrating database from version {} to {}",
            current_version, SCHEMA_VERSION
        );

        // Run migrations in order
        for version in (current_version + 1)..=SCHEMA_VERSION {
            apply_migration(conn, version)?;
        }

        info!("Database migration complete");
    } else if current_version > SCHEMA_VERSION {
        return Err(IndexerError::DatabaseMigrationFailed {
            message: format!(
                "Database schema version {} is newer than supported version {}",
                current_version, SCHEMA_VERSION
            ),
        });
    } else {
        debug!("Database schema is up to date");
    }

    Ok(())
}

/// Apply a specific migration version
fn apply_migration(conn: &Connection, version: i32) -> Result<(), IndexerError> {
    info!("Applying migration version {}", version);

    match version {
        1 => {
            // Version 1 is the initial schema, already created
            debug!("Version 1 migration: initial schema (already created)");
        }
        // Future migrations would go here
        // 2 => {
        //     conn.execute("ALTER TABLE submission_history ADD COLUMN new_field TEXT", [])?;
        // }
        _ => {
            return Err(IndexerError::DatabaseMigrationFailed {
                message: format!("Unknown migration version: {}", version),
            });
        }
    }

    // Update schema version
    conn.execute(
        "UPDATE schema_version SET version = ?1, applied_at = CURRENT_TIMESTAMP WHERE id = 1",
        [version],
    )
    .map_err(|e| IndexerError::DatabaseQueryFailed {
        message: format!("Failed to update schema version: {}", e),
    })?;

    Ok(())
}

/// Get the current schema version from the database
pub fn get_schema_version(conn: &Connection) -> Result<i32, IndexerError> {
    conn.query_row(
        "SELECT version FROM schema_version WHERE id = 1",
        [],
        |row| row.get(0),
    )
    .map_err(|e| IndexerError::DatabaseQueryFailed {
        message: format!("Failed to get schema version: {}", e),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_init_database_in_memory() {
        let conn = Connection::open_in_memory().unwrap();
        create_schema_version_table(&conn).unwrap();
        create_tables(&conn).unwrap();

        // Verify table exists
        let table_exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='submission_history')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(table_exists);

        // Verify schema version
        let version = get_schema_version(&conn).unwrap();
        assert_eq!(version, SCHEMA_VERSION);
    }

    #[test]
    fn test_create_tables() {
        let conn = Connection::open_in_memory().unwrap();
        create_schema_version_table(&conn).unwrap();
        create_tables(&conn).unwrap();

        // Check that all indexes were created
        let index_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND tbl_name='submission_history'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(index_count, CREATE_INDEXES.len() as i32);
    }

    #[test]
    fn test_migrate_database() {
        let conn = Connection::open_in_memory().unwrap();
        create_schema_version_table(&conn).unwrap();
        create_tables(&conn).unwrap();

        // Migration should succeed (no-op since we're already at current version)
        migrate_database(&conn).unwrap();

        let version = get_schema_version(&conn).unwrap();
        assert_eq!(version, SCHEMA_VERSION);
    }
}
