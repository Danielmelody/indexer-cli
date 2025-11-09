//! Token storage module for securely storing and retrieving OAuth tokens.
//!
//! This module handles the secure storage of OAuth 2.0 tokens including
//! access tokens, refresh tokens, and token metadata.

use crate::types::error::IndexerError;
use crate::types::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Token storage structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredToken {
    /// Access token
    pub access_token: String,

    /// Refresh token (used to get new access tokens)
    pub refresh_token: String,

    /// Token expiration time
    pub expires_at: DateTime<Utc>,

    /// Token scope(s)
    pub scopes: Vec<String>,

    /// Token type (usually "Bearer")
    #[serde(default = "default_token_type")]
    pub token_type: String,
}

fn default_token_type() -> String {
    "Bearer".to_string()
}

impl StoredToken {
    /// Create a new stored token
    pub fn new(
        access_token: String,
        refresh_token: String,
        expires_in: i64,
        scopes: Vec<String>,
    ) -> Self {
        let expires_at = Utc::now() + chrono::Duration::seconds(expires_in);

        Self {
            access_token,
            refresh_token,
            expires_at,
            scopes,
            token_type: default_token_type(),
        }
    }

    /// Check if the token is expired or will expire soon (within 5 minutes)
    pub fn is_expired(&self) -> bool {
        let buffer = chrono::Duration::minutes(5);
        Utc::now() + buffer >= self.expires_at
    }

    /// Check if the token has the required scopes
    pub fn has_scopes(&self, required_scopes: &[String]) -> bool {
        required_scopes.iter().all(|scope| self.scopes.contains(scope))
    }
}

/// Token store for managing OAuth tokens
pub struct TokenStore {
    /// Path to the token file
    token_file_path: PathBuf,
}

impl TokenStore {
    /// Create a new token store
    ///
    /// # Arguments
    ///
    /// * `token_file_path` - Path to the token file
    ///
    /// # Returns
    ///
    /// Returns a new TokenStore instance
    pub fn new(token_file_path: PathBuf) -> Self {
        Self { token_file_path }
    }

    /// Get the default token store for Google OAuth
    ///
    /// Tokens are stored in `~/.indexer-cli/google_oauth_token.json`
    pub fn google_default() -> Result<Self> {
        let token_dir = Self::get_token_directory()?;
        let token_file = token_dir.join("google_oauth_token.json");
        Ok(Self::new(token_file))
    }

    /// Get the token directory (~/.indexer-cli/)
    fn get_token_directory() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| IndexerError::InternalError {
            message: "Could not determine home directory".to_string(),
        })?;

        let token_dir = home.join(".indexer-cli");

        // Create directory if it doesn't exist
        if !token_dir.exists() {
            fs::create_dir_all(&token_dir).map_err(|e| {
                IndexerError::DirectoryCreationFailed {
                    path: token_dir.clone(),
                    message: e.to_string(),
                }
            })?;
        }

        Ok(token_dir)
    }

    /// Save a token to storage
    ///
    /// # Arguments
    ///
    /// * `token` - The token to save
    ///
    /// # Returns
    ///
    /// Returns Ok if the token was saved successfully
    ///
    /// # Errors
    ///
    /// Returns an error if the token file cannot be written
    pub fn save_token(&self, token: &StoredToken) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.token_file_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    IndexerError::DirectoryCreationFailed {
                        path: parent.to_path_buf(),
                        message: e.to_string(),
                    }
                })?;
            }
        }

        // Serialize token to JSON
        let json = serde_json::to_string_pretty(token).map_err(|e| {
            IndexerError::JsonSerializationError {
                message: e.to_string(),
            }
        })?;

        // Write to file
        fs::write(&self.token_file_path, json).map_err(|e| {
            IndexerError::FileWriteError {
                path: self.token_file_path.clone(),
                message: e.to_string(),
            }
        })?;

        // Set file permissions to 600 (read/write for owner only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&self.token_file_path)
                .map_err(|e| IndexerError::FileReadError {
                    path: self.token_file_path.clone(),
                    message: e.to_string(),
                })?
                .permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&self.token_file_path, perms).map_err(|_e| {
                IndexerError::FilePermissionDenied {
                    path: self.token_file_path.clone(),
                }
            })?;
        }

        Ok(())
    }

    /// Load a token from storage
    ///
    /// # Returns
    ///
    /// Returns the stored token if it exists
    ///
    /// # Errors
    ///
    /// Returns an error if the token file cannot be read or parsed
    pub fn load_token(&self) -> Result<StoredToken> {
        if !self.token_file_path.exists() {
            return Err(IndexerError::FileNotFound {
                path: self.token_file_path.clone(),
            });
        }

        let json = fs::read_to_string(&self.token_file_path).map_err(|e| {
            IndexerError::FileReadError {
                path: self.token_file_path.clone(),
                message: e.to_string(),
            }
        })?;

        let token: StoredToken = serde_json::from_str(&json).map_err(|e| {
            IndexerError::JsonDeserializationError {
                message: e.to_string(),
            }
        })?;

        Ok(token)
    }

    /// Check if a token exists
    ///
    /// # Returns
    ///
    /// Returns true if a token file exists
    pub fn has_token(&self) -> bool {
        self.token_file_path.exists()
    }

    /// Delete the stored token
    ///
    /// # Returns
    ///
    /// Returns Ok if the token was deleted successfully
    ///
    /// # Errors
    ///
    /// Returns an error if the token file cannot be deleted
    pub fn delete_token(&self) -> Result<()> {
        if self.token_file_path.exists() {
            fs::remove_file(&self.token_file_path).map_err(|e| {
                IndexerError::FileWriteError {
                    path: self.token_file_path.clone(),
                    message: e.to_string(),
                }
            })?;
        }
        Ok(())
    }

    /// Get the path to the token file
    pub fn token_file_path(&self) -> &Path {
        &self.token_file_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_stored_token_creation() {
        let token = StoredToken::new(
            "access_token".to_string(),
            "refresh_token".to_string(),
            3600,
            vec!["scope1".to_string(), "scope2".to_string()],
        );

        assert_eq!(token.access_token, "access_token");
        assert_eq!(token.refresh_token, "refresh_token");
        assert!(!token.is_expired());
        assert!(token.has_scopes(&vec!["scope1".to_string()]));
        assert!(token.has_scopes(&vec!["scope1".to_string(), "scope2".to_string()]));
        assert!(!token.has_scopes(&vec!["scope3".to_string()]));
    }

    #[test]
    fn test_token_expiration() {
        let token = StoredToken::new(
            "access_token".to_string(),
            "refresh_token".to_string(),
            -1, // Already expired
            vec!["scope1".to_string()],
        );

        assert!(token.is_expired());
    }

    #[test]
    fn test_token_store_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let token_path = temp_dir.path().join("token.json");
        let store = TokenStore::new(token_path);

        let token = StoredToken::new(
            "access_token".to_string(),
            "refresh_token".to_string(),
            3600,
            vec!["scope1".to_string()],
        );

        // Save token
        store.save_token(&token).unwrap();

        // Check if token exists
        assert!(store.has_token());

        // Load token
        let loaded_token = store.load_token().unwrap();
        assert_eq!(loaded_token.access_token, "access_token");
        assert_eq!(loaded_token.refresh_token, "refresh_token");

        // Delete token
        store.delete_token().unwrap();
        assert!(!store.has_token());
    }
}
