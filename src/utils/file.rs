//! File operation utilities
//!
//! This module provides utilities for file and directory operations including:
//! - Ensuring directories exist
//! - Reading and writing files
//! - Expanding paths with ~ symbols
//! - Reading content from HTTP URLs

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::fs as async_fs;
use tracing::{debug, trace};

/// Ensure a directory exists, creating it if necessary
///
/// # Arguments
///
/// * `path` - The directory path to ensure exists
///
/// # Returns
///
/// Returns `Ok(PathBuf)` with the canonical path if successful, or an error otherwise
///
/// # Examples
///
/// ```no_run
/// use indexer_cli::utils::file::ensure_dir_exists;
/// use std::path::Path;
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let dir = ensure_dir_exists(Path::new("/tmp/test")).await?;
/// # Ok(())
/// # }
/// ```
pub async fn ensure_dir_exists(path: &Path) -> Result<PathBuf> {
    if !path.exists() {
        debug!("Creating directory: {}", path.display());
        async_fs::create_dir_all(path)
            .await
            .with_context(|| format!("Failed to create directory: {}", path.display()))?;
    } else if !path.is_dir() {
        anyhow::bail!("Path exists but is not a directory: {}", path.display());
    }

    // Return the canonical path
    let canonical = path
        .canonicalize()
        .with_context(|| format!("Failed to canonicalize path: {}", path.display()))?;

    Ok(canonical)
}

/// Synchronous version of ensure_dir_exists
///
/// # Arguments
///
/// * `path` - The directory path to ensure exists
///
/// # Returns
///
/// Returns `Ok(PathBuf)` with the canonical path if successful, or an error otherwise
///
/// # Examples
///
/// ```no_run
/// use indexer_cli::utils::file::ensure_dir_exists_sync;
/// use std::path::Path;
///
/// let dir = ensure_dir_exists_sync(Path::new("/tmp/test"))?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn ensure_dir_exists_sync(path: &Path) -> Result<PathBuf> {
    if !path.exists() {
        debug!("Creating directory: {}", path.display());
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory: {}", path.display()))?;
    } else if !path.is_dir() {
        anyhow::bail!("Path exists but is not a directory: {}", path.display());
    }

    // Return the canonical path
    let canonical = path
        .canonicalize()
        .with_context(|| format!("Failed to canonicalize path: {}", path.display()))?;

    Ok(canonical)
}

/// Read file content as a string
///
/// # Arguments
///
/// * `path` - The file path to read
///
/// # Returns
///
/// Returns the file content as a `String`, or an error if reading fails
///
/// # Examples
///
/// ```no_run
/// use indexer_cli::utils::file::read_file;
/// use std::path::Path;
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let content = read_file(Path::new("/tmp/test.txt")).await?;
/// # Ok(())
/// # }
/// ```
pub async fn read_file(path: &Path) -> Result<String> {
    trace!("Reading file: {}", path.display());

    let content = async_fs::read_to_string(path)
        .await
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    Ok(content)
}

/// Synchronous version of read_file
///
/// # Arguments
///
/// * `path` - The file path to read
///
/// # Returns
///
/// Returns the file content as a `String`, or an error if reading fails
///
/// # Examples
///
/// ```no_run
/// use indexer_cli::utils::file::read_file_sync;
/// use std::path::Path;
///
/// let content = read_file_sync(Path::new("/tmp/test.txt"))?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn read_file_sync(path: &Path) -> Result<String> {
    trace!("Reading file: {}", path.display());

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    Ok(content)
}

/// Read content from a file or HTTP URL
///
/// If the input starts with "http://" or "https://", it will be fetched as a URL.
/// Otherwise, it will be treated as a file path.
///
/// # Arguments
///
/// * `source` - The file path or URL to read
///
/// # Returns
///
/// Returns the content as a `String`, or an error if reading fails
///
/// # Examples
///
/// ```no_run
/// use indexer_cli::utils::file::read_content;
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// // Read from file
/// let file_content = read_content("/tmp/test.txt").await?;
///
/// // Read from URL
/// let url_content = read_content("https://placeholder.test/data.txt").await?;
/// # Ok(())
/// # }
/// ```
pub async fn read_content(source: &str) -> Result<String> {
    if source.starts_with("http://") || source.starts_with("https://") {
        // Read from URL
        debug!("Fetching content from URL: {}", source);

        let response = reqwest::get(source)
            .await
            .with_context(|| format!("Failed to fetch URL: {}", source))?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Failed to fetch URL: {} (status: {})",
                source,
                response.status()
            );
        }

        let content = response
            .text()
            .await
            .with_context(|| format!("Failed to read response body from: {}", source))?;

        Ok(content)
    } else {
        // Read from file
        let path = expand_path(source)?;
        read_file(&path).await
    }
}

/// Write content to a file
///
/// Creates parent directories if they don't exist.
///
/// # Arguments
///
/// * `path` - The file path to write to
/// * `content` - The content to write
///
/// # Returns
///
/// Returns `Ok(())` if successful, or an error if writing fails
///
/// # Examples
///
/// ```no_run
/// use indexer_cli::utils::file::write_file;
/// use std::path::Path;
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// write_file(Path::new("/tmp/output.txt"), "Hello, world!").await?;
/// # Ok(())
/// # }
/// ```
pub async fn write_file(path: &Path, content: &str) -> Result<()> {
    trace!("Writing file: {}", path.display());

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        ensure_dir_exists(parent).await?;
    }

    async_fs::write(path, content)
        .await
        .with_context(|| format!("Failed to write file: {}", path.display()))?;

    Ok(())
}

/// Synchronous version of write_file
///
/// # Arguments
///
/// * `path` - The file path to write to
/// * `content` - The content to write
///
/// # Returns
///
/// Returns `Ok(())` if successful, or an error if writing fails
///
/// # Examples
///
/// ```no_run
/// use indexer_cli::utils::file::write_file_sync;
/// use std::path::Path;
///
/// write_file_sync(Path::new("/tmp/output.txt"), "Hello, world!")?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn write_file_sync(path: &Path, content: &str) -> Result<()> {
    trace!("Writing file: {}", path.display());

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        ensure_dir_exists_sync(parent)?;
    }

    fs::write(path, content)
        .with_context(|| format!("Failed to write file: {}", path.display()))?;

    Ok(())
}

/// Write bytes to a file
///
/// Creates parent directories if they don't exist.
///
/// # Arguments
///
/// * `path` - The file path to write to
/// * `content` - The bytes to write
///
/// # Returns
///
/// Returns `Ok(())` if successful, or an error if writing fails
///
/// # Examples
///
/// ```no_run
/// use indexer_cli::utils::file::write_bytes;
/// use std::path::Path;
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// write_bytes(Path::new("/tmp/data.bin"), &[0, 1, 2, 3]).await?;
/// # Ok(())
/// # }
/// ```
pub async fn write_bytes(path: &Path, content: &[u8]) -> Result<()> {
    trace!("Writing bytes to file: {}", path.display());

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        ensure_dir_exists(parent).await?;
    }

    async_fs::write(path, content)
        .await
        .with_context(|| format!("Failed to write file: {}", path.display()))?;

    Ok(())
}

/// Expand a path, replacing ~ with the home directory
///
/// # Arguments
///
/// * `path_str` - The path string to expand
///
/// # Returns
///
/// Returns the expanded `PathBuf`, or an error if expansion fails
///
/// # Examples
///
/// ```no_run
/// use indexer_cli::utils::file::expand_path;
///
/// let path = expand_path("~/Documents/test.txt")?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn expand_path(path_str: &str) -> Result<PathBuf> {
    if path_str.starts_with("~/") {
        // Expand home directory
        let home = dirs::home_dir().context("Failed to determine home directory")?;

        let rest = &path_str[2..]; // Skip "~/"
        Ok(home.join(rest))
    } else if path_str == "~" {
        // Just the home directory
        dirs::home_dir().context("Failed to determine home directory")
    } else {
        // No expansion needed
        Ok(PathBuf::from(path_str))
    }
}

/// Get the file extension from a path
///
/// # Arguments
///
/// * `path` - The file path
///
/// # Returns
///
/// Returns the file extension as a lowercase string, or None if there's no extension
///
/// # Examples
///
/// ```
/// use indexer_cli::utils::file::get_file_extension;
/// use std::path::Path;
///
/// assert_eq!(get_file_extension(Path::new("test.txt")), Some("txt".to_string()));
/// assert_eq!(get_file_extension(Path::new("test.tar.gz")), Some("gz".to_string()));
/// assert_eq!(get_file_extension(Path::new("test")), None);
/// ```
pub fn get_file_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
}

/// Check if a file exists
///
/// # Arguments
///
/// * `path` - The file path to check
///
/// # Returns
///
/// Returns `true` if the file exists, `false` otherwise
///
/// # Examples
///
/// ```
/// use indexer_cli::utils::file::file_exists;
/// use std::path::Path;
///
/// if file_exists(Path::new("/tmp/test.txt")) {
///     println!("File exists!");
/// }
/// ```
pub fn file_exists(path: &Path) -> bool {
    path.exists() && path.is_file()
}

/// Check if a directory exists
///
/// # Arguments
///
/// * `path` - The directory path to check
///
/// # Returns
///
/// Returns `true` if the directory exists, `false` otherwise
///
/// # Examples
///
/// ```
/// use indexer_cli::utils::file::dir_exists;
/// use std::path::Path;
///
/// if dir_exists(Path::new("/tmp")) {
///     println!("Directory exists!");
/// }
/// ```
pub fn dir_exists(path: &Path) -> bool {
    path.exists() && path.is_dir()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_ensure_dir_exists() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("test/nested/dir");

        assert!(!test_dir.exists());

        let result = ensure_dir_exists(&test_dir).await;
        assert!(result.is_ok());
        assert!(test_dir.exists());
        assert!(test_dir.is_dir());
    }

    #[test]
    fn test_ensure_dir_exists_sync() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("test/nested/dir");

        assert!(!test_dir.exists());

        let result = ensure_dir_exists_sync(&test_dir);
        assert!(result.is_ok());
        assert!(test_dir.exists());
        assert!(test_dir.is_dir());
    }

    #[tokio::test]
    async fn test_read_write_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        let content = "Hello, world!";
        write_file(&test_file, content).await.unwrap();

        assert!(test_file.exists());

        let read_content = read_file(&test_file).await.unwrap();
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_read_write_file_sync() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        let content = "Hello, world!";
        write_file_sync(&test_file, content).unwrap();

        assert!(test_file.exists());

        let read_content = read_file_sync(&test_file).unwrap();
        assert_eq!(read_content, content);
    }

    #[tokio::test]
    async fn test_write_bytes() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("data.bin");

        let data = vec![0u8, 1, 2, 3, 4, 5];
        write_bytes(&test_file, &data).await.unwrap();

        assert!(test_file.exists());

        let read_data = async_fs::read(&test_file).await.unwrap();
        assert_eq!(read_data, data);
    }

    #[test]
    fn test_expand_path() {
        // Test without expansion
        let path = expand_path("/tmp/test.txt").unwrap();
        assert_eq!(path, PathBuf::from("/tmp/test.txt"));

        // Test with ~ (if home directory is available)
        if let Ok(home) = dirs::home_dir().ok_or("No home dir") {
            let path = expand_path("~/test.txt").unwrap();
            assert_eq!(path, home.join("test.txt"));

            let path = expand_path("~").unwrap();
            assert_eq!(path, home);
        }
    }

    #[test]
    fn test_get_file_extension() {
        assert_eq!(
            get_file_extension(Path::new("test.txt")),
            Some("txt".to_string())
        );
        assert_eq!(
            get_file_extension(Path::new("test.TAR.GZ")),
            Some("gz".to_string())
        );
        assert_eq!(get_file_extension(Path::new("test")), None);
        assert_eq!(get_file_extension(Path::new(".hidden")), None);
        assert_eq!(
            get_file_extension(Path::new("test.json")),
            Some("json".to_string())
        );
    }

    #[test]
    fn test_file_exists() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        assert!(!file_exists(&test_file));

        std::fs::write(&test_file, "test").unwrap();
        assert!(file_exists(&test_file));

        // Directory should return false for file_exists
        assert!(!file_exists(temp_dir.path()));
    }

    #[test]
    fn test_dir_exists() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("test");

        assert!(!dir_exists(&test_dir));

        std::fs::create_dir(&test_dir).unwrap();
        assert!(dir_exists(&test_dir));

        // File should return false for dir_exists
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "test").unwrap();
        assert!(!dir_exists(&test_file));
    }
}
