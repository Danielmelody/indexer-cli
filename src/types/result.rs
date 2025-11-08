//! Result type aliases for the indexer-cli application.
//!
//! This module defines convenient type aliases for Result types used throughout
//! the application, making function signatures cleaner and more consistent.

use super::error::IndexerError;

/// The standard Result type for the indexer-cli application.
///
/// This is a type alias for `Result<T, IndexerError>`, which is used throughout
/// the application for any operation that can fail with an `IndexerError`.
///
/// # Examples
///
/// ```ignore
/// use indexer_cli::types::Result;
///
/// fn load_config() -> Result<Config> {
///     // Function implementation
/// }
/// ```
pub type Result<T> = std::result::Result<T, IndexerError>;

/// A unit Result type that returns nothing on success.
///
/// This is a convenience alias for `Result<()>`, commonly used for functions
/// that perform actions but don't return a meaningful value.
///
/// # Examples
///
/// ```ignore
/// use indexer_cli::types::UnitResult;
///
/// fn validate_config() -> UnitResult {
///     // Validation logic
///     Ok(())
/// }
/// ```
pub type UnitResult = Result<()>;

/// A Result type for operations that return a string.
///
/// This is a convenience alias for `Result<String>`, commonly used for
/// functions that generate or retrieve text data.
///
/// # Examples
///
/// ```ignore
/// use indexer_cli::types::StringResult;
///
/// fn generate_api_key() -> StringResult {
///     Ok("generated-key".to_string())
/// }
/// ```
pub type StringResult = Result<String>;

/// A Result type for operations that return a boolean.
///
/// This is a convenience alias for `Result<bool>`, commonly used for
/// validation and existence checks.
///
/// # Examples
///
/// ```ignore
/// use indexer_cli::types::BoolResult;
///
/// fn url_exists(url: &str) -> BoolResult {
///     Ok(true)
/// }
/// ```
pub type BoolResult = Result<bool>;

/// A Result type for operations that return an optional value.
///
/// This is useful for operations that may or may not find a value,
/// but can still fail with an error.
///
/// # Examples
///
/// ```ignore
/// use indexer_cli::types::OptionResult;
///
/// fn find_config() -> OptionResult<Config> {
///     // None if not found, Some if found, Err if failed
///     Ok(Some(config))
/// }
/// ```
pub type OptionResult<T> = Result<Option<T>>;

/// A Result type for batch operations that return a collection.
///
/// This is useful for operations that process multiple items and return
/// a vector of results.
///
/// # Examples
///
/// ```ignore
/// use indexer_cli::types::VecResult;
///
/// fn parse_urls(sitemap: &str) -> VecResult<String> {
///     Ok(vec!["url1".to_string(), "url2".to_string()])
/// }
/// ```
pub type VecResult<T> = Result<Vec<T>>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::error::IndexerError;

    #[test]
    fn test_result_ok() {
        let result: Result<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_result_err() {
        let result: Result<i32> = Err(IndexerError::OperationCancelled);
        assert!(result.is_err());
    }

    #[test]
    fn test_unit_result() {
        let result: UnitResult = Ok(());
        assert!(result.is_ok());
    }

    #[test]
    fn test_string_result() {
        let result: StringResult = Ok("test".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test");
    }

    #[test]
    fn test_bool_result() {
        let result: BoolResult = Ok(true);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_option_result() {
        let result: OptionResult<i32> = Ok(Some(42));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(42));

        let result: OptionResult<i32> = Ok(None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_vec_result() {
        let result: VecResult<String> = Ok(vec!["a".to_string(), "b".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }
}
