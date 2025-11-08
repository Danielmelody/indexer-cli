//! Retry logic with exponential backoff
//!
//! This module provides utilities for retrying operations with configurable
//! backoff strategies, retry conditions, and limits.

use anyhow::{anyhow, Result};
use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};

/// Default maximum number of retry attempts
pub const DEFAULT_MAX_RETRIES: usize = 3;

/// Default initial backoff duration (in milliseconds)
pub const DEFAULT_INITIAL_BACKOFF_MS: u64 = 100;

/// Default maximum backoff duration (in seconds)
pub const DEFAULT_MAX_BACKOFF_SECS: u64 = 60;

/// Default backoff multiplier for exponential backoff
pub const DEFAULT_BACKOFF_MULTIPLIER: f64 = 2.0;

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (0 means no retries)
    pub max_retries: usize,
    /// Initial backoff duration
    pub initial_backoff: Duration,
    /// Maximum backoff duration
    pub max_backoff: Duration,
    /// Backoff multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Whether to add jitter to backoff durations
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: DEFAULT_MAX_RETRIES,
            initial_backoff: Duration::from_millis(DEFAULT_INITIAL_BACKOFF_MS),
            max_backoff: Duration::from_secs(DEFAULT_MAX_BACKOFF_SECS),
            backoff_multiplier: DEFAULT_BACKOFF_MULTIPLIER,
            jitter: true,
        }
    }
}

impl RetryConfig {
    /// Create a new retry configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum number of retry attempts
    pub fn with_max_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Set the initial backoff duration
    pub fn with_initial_backoff(mut self, duration: Duration) -> Self {
        self.initial_backoff = duration;
        self
    }

    /// Set the maximum backoff duration
    pub fn with_max_backoff(mut self, duration: Duration) -> Self {
        self.max_backoff = duration;
        self
    }

    /// Set the backoff multiplier
    pub fn with_backoff_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    /// Enable or disable jitter
    pub fn with_jitter(mut self, jitter: bool) -> Self {
        self.jitter = jitter;
        self
    }

    /// Calculate the backoff duration for a given attempt
    fn calculate_backoff(&self, attempt: usize) -> Duration {
        let base_duration = self.initial_backoff.as_millis() as f64
            * self.backoff_multiplier.powi(attempt as i32);

        let duration_ms = base_duration.min(self.max_backoff.as_millis() as f64) as u64;

        let mut duration = Duration::from_millis(duration_ms);

        // Add jitter if enabled (randomize ±25%)
        if self.jitter {
            use rand::Rng;
            let mut rng = rand::rng();
            let jitter_factor = rng.gen_range(0.75..=1.25);
            duration = Duration::from_millis((duration.as_millis() as f64 * jitter_factor) as u64);
        }

        duration
    }
}

/// Trait for determining if an error should trigger a retry
pub trait ShouldRetry {
    /// Returns true if the operation should be retried
    fn should_retry(&self) -> bool;
}

// Implement ShouldRetry for common error types
impl ShouldRetry for anyhow::Error {
    fn should_retry(&self) -> bool {
        // By default, retry on network errors
        if let Some(err) = self.downcast_ref::<reqwest::Error>() {
            err.should_retry()
        } else {
            false
        }
    }
}

impl ShouldRetry for reqwest::Error {
    fn should_retry(&self) -> bool {
        // Retry on timeout, connection errors, and certain status codes
        self.is_timeout()
            || self.is_connect()
            || self
                .status()
                .map(|s| s.is_server_error() || s == 429)
                .unwrap_or(false)
    }
}

/// Retry an operation with exponential backoff
///
/// # Arguments
///
/// * `config` - Retry configuration
/// * `operation` - The operation to retry (a closure that returns a Future)
///
/// # Returns
///
/// Returns the result of the operation if successful, or the last error if all retries fail
///
/// # Examples
///
/// ```no_run
/// use indexer_cli::utils::retry::{retry_with_backoff, RetryConfig};
/// use anyhow::Result;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let config = RetryConfig::new().with_max_retries(5);
///
///     let result = retry_with_backoff(config, || async {
///         // Your async operation here
///         Ok::<_, anyhow::Error>("Success")
///     }).await?;
///
///     Ok(())
/// }
/// ```
pub async fn retry_with_backoff<F, Fut, T, E>(config: RetryConfig, mut operation: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::error::Error + Send + Sync + ShouldRetry + 'static,
{
    let mut last_error = None;

    for attempt in 0..=config.max_retries {
        match operation().await {
            Ok(result) => {
                if attempt > 0 {
                    debug!("Operation succeeded after {} retries", attempt);
                }
                return Ok(result);
            }
            Err(err) => {
                last_error = Some(err);
                let err_ref = last_error.as_ref().unwrap();

                // Check if we should retry
                if attempt < config.max_retries && err_ref.should_retry() {
                    let backoff = config.calculate_backoff(attempt);
                    warn!(
                        "Operation failed (attempt {}/{}): {}. Retrying in {:?}",
                        attempt + 1,
                        config.max_retries + 1,
                        err_ref,
                        backoff
                    );
                    sleep(backoff).await;
                } else {
                    break;
                }
            }
        }
    }

    Err(anyhow!(
        "Operation failed after {} attempts: {}",
        config.max_retries + 1,
        last_error.unwrap()
    ))
}

/// Retry an operation with a custom retry condition
///
/// # Arguments
///
/// * `config` - Retry configuration
/// * `should_retry_fn` - Function that determines if a retry should occur based on the error
/// * `operation` - The operation to retry
///
/// # Returns
///
/// Returns the result of the operation if successful, or the last error if all retries fail
///
/// # Examples
///
/// ```no_run
/// use indexer_cli::utils::retry::{retry_with_condition, RetryConfig};
/// use anyhow::Result;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let config = RetryConfig::new();
///
///     let result = retry_with_condition(
///         config,
///         |err: &anyhow::Error| {
///             // Custom retry logic
///             err.to_string().contains("temporary")
///         },
///         || async {
///             // Your async operation here
///             Ok::<_, anyhow::Error>("Success")
///         }
///     ).await?;
///
///     Ok(())
/// }
/// ```
pub async fn retry_with_condition<F, Fut, T, E, P>(
    config: RetryConfig,
    should_retry_fn: P,
    mut operation: F,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::error::Error + Send + Sync + 'static,
    P: Fn(&E) -> bool,
{
    let mut last_error = None;

    for attempt in 0..=config.max_retries {
        match operation().await {
            Ok(result) => {
                if attempt > 0 {
                    debug!("Operation succeeded after {} retries", attempt);
                }
                return Ok(result);
            }
            Err(err) => {
                last_error = Some(err);
                let err_ref = last_error.as_ref().unwrap();

                // Check if we should retry using custom condition
                if attempt < config.max_retries && should_retry_fn(err_ref) {
                    let backoff = config.calculate_backoff(attempt);
                    warn!(
                        "Operation failed (attempt {}/{}): {}. Retrying in {:?}",
                        attempt + 1,
                        config.max_retries + 1,
                        err_ref,
                        backoff
                    );
                    sleep(backoff).await;
                } else {
                    break;
                }
            }
        }
    }

    Err(anyhow!(
        "Operation failed after {} attempts: {}",
        config.max_retries + 1,
        last_error.unwrap()
    ))
}

/// Simple retry with default configuration
///
/// This is a convenience function that uses default retry settings
///
/// # Arguments
///
/// * `operation` - The operation to retry
///
/// # Returns
///
/// Returns the result of the operation if successful, or the last error if all retries fail
///
/// # Examples
///
/// ```no_run
/// use indexer_cli::utils::retry::retry;
/// use anyhow::Result;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let result = retry(|| async {
///         // Your async operation here
///         Ok::<_, anyhow::Error>("Success")
///     }).await?;
///
///     Ok(())
/// }
/// ```
pub async fn retry<F, Fut, T, E>(operation: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::error::Error + Send + Sync + ShouldRetry + 'static,
{
    retry_with_backoff(RetryConfig::default(), operation).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, DEFAULT_MAX_RETRIES);
        assert_eq!(
            config.initial_backoff,
            Duration::from_millis(DEFAULT_INITIAL_BACKOFF_MS)
        );
        assert_eq!(
            config.max_backoff,
            Duration::from_secs(DEFAULT_MAX_BACKOFF_SECS)
        );
        assert_eq!(config.backoff_multiplier, DEFAULT_BACKOFF_MULTIPLIER);
        assert!(config.jitter);
    }

    #[test]
    fn test_retry_config_builder() {
        let config = RetryConfig::new()
            .with_max_retries(5)
            .with_initial_backoff(Duration::from_millis(200))
            .with_max_backoff(Duration::from_secs(30))
            .with_backoff_multiplier(3.0)
            .with_jitter(false);

        assert_eq!(config.max_retries, 5);
        assert_eq!(config.initial_backoff, Duration::from_millis(200));
        assert_eq!(config.max_backoff, Duration::from_secs(30));
        assert_eq!(config.backoff_multiplier, 3.0);
        assert!(!config.jitter);
    }

    #[test]
    fn test_calculate_backoff() {
        let config = RetryConfig::new()
            .with_initial_backoff(Duration::from_millis(100))
            .with_backoff_multiplier(2.0)
            .with_jitter(false);

        assert_eq!(config.calculate_backoff(0), Duration::from_millis(100));
        assert_eq!(config.calculate_backoff(1), Duration::from_millis(200));
        assert_eq!(config.calculate_backoff(2), Duration::from_millis(400));
        assert_eq!(config.calculate_backoff(3), Duration::from_millis(800));
    }

    #[test]
    fn test_calculate_backoff_with_max() {
        let config = RetryConfig::new()
            .with_initial_backoff(Duration::from_millis(100))
            .with_max_backoff(Duration::from_millis(300))
            .with_backoff_multiplier(2.0)
            .with_jitter(false);

        assert_eq!(config.calculate_backoff(0), Duration::from_millis(100));
        assert_eq!(config.calculate_backoff(1), Duration::from_millis(200));
        assert_eq!(config.calculate_backoff(2), Duration::from_millis(300));
        assert_eq!(config.calculate_backoff(3), Duration::from_millis(300));
    }

    #[tokio::test]
    async fn test_retry_success() {
        let mut attempt = 0;
        let config = RetryConfig::new().with_max_retries(3);

        let result = retry_with_condition(
            config,
            |_: &anyhow::Error| true,
            || async {
                attempt += 1;
                if attempt < 3 {
                    Err(anyhow!("Temporary error"))
                } else {
                    Ok("Success")
                }
            },
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success");
        assert_eq!(attempt, 3);
    }

    #[tokio::test]
    async fn test_retry_failure() {
        let mut attempt = 0;
        let config = RetryConfig::new().with_max_retries(2);

        let result = retry_with_condition(
            config,
            |_: &anyhow::Error| true,
            || async {
                attempt += 1;
                Err::<(), _>(anyhow!("Permanent error"))
            },
        )
        .await;

        assert!(result.is_err());
        assert_eq!(attempt, 3); // Initial attempt + 2 retries
    }

    #[tokio::test]
    async fn test_retry_no_retry_on_condition() {
        let mut attempt = 0;
        let config = RetryConfig::new().with_max_retries(3);

        let result = retry_with_condition(
            config,
            |err: &anyhow::Error| !err.to_string().contains("permanent"),
            || async {
                attempt += 1;
                Err::<(), _>(anyhow!("permanent error"))
            },
        )
        .await;

        assert!(result.is_err());
        assert_eq!(attempt, 1); // Only initial attempt, no retries
    }
}
