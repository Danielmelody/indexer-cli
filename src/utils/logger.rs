//! Logging utilities using tracing framework
//!
//! This module provides logging initialization with support for:
//! - Console and file output
//! - Configurable log levels
//! - Log rotation
//! - Structured logging with tracing

use anyhow::{Context, Result};
use tracing::Level;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

/// Log output destination
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogDestination {
    /// Log to console only
    Console,
    /// Log to file only
    File,
    /// Log to both console and file
    Both,
}

/// Log rotation configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogRotation {
    /// Rotate logs daily
    Daily,
    /// Rotate logs hourly
    Hourly,
    /// Rotate logs every minute (useful for testing)
    Minutely,
    /// Never rotate logs
    Never,
}

impl From<LogRotation> for Rotation {
    fn from(rotation: LogRotation) -> Self {
        match rotation {
            LogRotation::Daily => Rotation::DAILY,
            LogRotation::Hourly => Rotation::HOURLY,
            LogRotation::Minutely => Rotation::MINUTELY,
            LogRotation::Never => Rotation::NEVER,
        }
    }
}

/// Logger configuration
#[derive(Debug, Clone)]
pub struct LoggerConfig {
    /// Log level (default: INFO)
    pub level: Level,
    /// Output destination
    pub destination: LogDestination,
    /// Log file directory (required if destination includes File)
    pub log_dir: Option<String>,
    /// Log file prefix (default: "indexer-cli")
    pub file_prefix: Option<String>,
    /// Log rotation strategy
    pub rotation: LogRotation,
    /// Whether to include span events (for tracing async operations)
    pub include_spans: bool,
    /// Whether to use ANSI colors in console output
    pub ansi: bool,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            level: Level::INFO,
            destination: LogDestination::Console,
            log_dir: None,
            file_prefix: Some("indexer-cli".to_string()),
            rotation: LogRotation::Daily,
            include_spans: false,
            ansi: true,
        }
    }
}

impl LoggerConfig {
    /// Create a new logger configuration with sensible defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the log level
    pub fn with_level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    /// Set the output destination
    pub fn with_destination(mut self, destination: LogDestination) -> Self {
        self.destination = destination;
        self
    }

    /// Set the log directory
    pub fn with_log_dir(mut self, dir: impl Into<String>) -> Self {
        self.log_dir = Some(dir.into());
        self
    }

    /// Set the log file prefix
    pub fn with_file_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.file_prefix = Some(prefix.into());
        self
    }

    /// Set the log rotation strategy
    pub fn with_rotation(mut self, rotation: LogRotation) -> Self {
        self.rotation = rotation;
        self
    }

    /// Enable span tracing
    pub fn with_spans(mut self, include_spans: bool) -> Self {
        self.include_spans = include_spans;
        self
    }

    /// Enable or disable ANSI colors
    pub fn with_ansi(mut self, ansi: bool) -> Self {
        self.ansi = ansi;
        self
    }
}

/// Initialize the tracing logger with the given configuration
///
/// # Arguments
///
/// * `config` - Logger configuration
///
/// # Returns
///
/// Returns `Ok(())` if initialization succeeds, or an error if it fails
///
/// # Examples
///
/// ```no_run
/// use indexer_cli::utils::logger::{init_logger, LoggerConfig, LogDestination};
/// use tracing::Level;
///
/// let config = LoggerConfig::new()
///     .with_level(Level::DEBUG)
///     .with_destination(LogDestination::Both)
///     .with_log_dir("./logs");
///
/// init_logger(config).expect("Failed to initialize logger");
/// ```
pub fn init_logger(config: LoggerConfig) -> Result<()> {
    // Build the environment filter
    let env_filter = EnvFilter::builder()
        .with_default_directive(config.level.into())
        .from_env_lossy();

    match config.destination {
        LogDestination::Console => {
            // Console only
            let span_events = if config.include_spans {
                FmtSpan::NEW | FmtSpan::CLOSE
            } else {
                FmtSpan::NONE
            };

            let console_layer = fmt::layer()
                .with_span_events(span_events)
                .with_ansi(config.ansi)
                .with_filter(env_filter);

            tracing_subscriber::registry().with(console_layer).init();
        }
        LogDestination::File => {
            // File only
            let span_events = if config.include_spans {
                FmtSpan::NEW | FmtSpan::CLOSE
            } else {
                FmtSpan::NONE
            };

            let log_dir = config
                .log_dir
                .context("Log directory must be specified for file logging")?;
            let file_prefix = config.file_prefix.unwrap_or_else(|| "indexer-cli".to_string());

            let file_appender =
                RollingFileAppender::new(config.rotation.into(), log_dir, file_prefix);

            let file_layer = fmt::layer()
                .with_writer(file_appender)
                .with_span_events(span_events)
                .with_ansi(false) // No ANSI colors in files
                .with_filter(env_filter);

            tracing_subscriber::registry().with(file_layer).init();
        }
        LogDestination::Both => {
            // Both console and file
            let span_events = if config.include_spans {
                FmtSpan::NEW | FmtSpan::CLOSE
            } else {
                FmtSpan::NONE
            };

            let log_dir = config
                .log_dir
                .context("Log directory must be specified for file logging")?;
            let file_prefix = config.file_prefix.unwrap_or_else(|| "indexer-cli".to_string());

            let file_appender =
                RollingFileAppender::new(config.rotation.into(), log_dir, file_prefix);

            // Clone the env_filter for both layers
            let console_filter = env_filter.clone();
            let file_filter = env_filter;

            let console_layer = fmt::layer()
                .with_span_events(span_events.clone())
                .with_ansi(config.ansi)
                .with_filter(console_filter);

            let file_layer = fmt::layer()
                .with_writer(file_appender)
                .with_span_events(span_events)
                .with_ansi(false)
                .with_filter(file_filter);

            tracing_subscriber::registry()
                .with(console_layer)
                .with(file_layer)
                .init();
        }
    }

    Ok(())
}

/// Initialize a simple console logger with default settings
///
/// This is a convenience function for quick logger setup during development
///
/// # Examples
///
/// ```no_run
/// use indexer_cli::utils::logger::init_simple_logger;
///
/// init_simple_logger().expect("Failed to initialize logger");
/// ```
pub fn init_simple_logger() -> Result<()> {
    init_logger(LoggerConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_config_default() {
        let config = LoggerConfig::default();
        assert_eq!(config.level, Level::INFO);
        assert_eq!(config.destination, LogDestination::Console);
        assert!(config.log_dir.is_none());
        assert_eq!(config.file_prefix, Some("indexer-cli".to_string()));
        assert_eq!(config.rotation, LogRotation::Daily);
        assert!(!config.include_spans);
        assert!(config.ansi);
    }

    #[test]
    fn test_logger_config_builder() {
        let config = LoggerConfig::new()
            .with_level(Level::DEBUG)
            .with_destination(LogDestination::Both)
            .with_log_dir("/tmp/logs")
            .with_file_prefix("test")
            .with_rotation(LogRotation::Hourly)
            .with_spans(true)
            .with_ansi(false);

        assert_eq!(config.level, Level::DEBUG);
        assert_eq!(config.destination, LogDestination::Both);
        assert_eq!(config.log_dir, Some("/tmp/logs".to_string()));
        assert_eq!(config.file_prefix, Some("test".to_string()));
        assert_eq!(config.rotation, LogRotation::Hourly);
        assert!(config.include_spans);
        assert!(!config.ansi);
    }

    #[test]
    fn test_log_rotation_conversion() {
        assert_eq!(Rotation::from(LogRotation::Daily), Rotation::DAILY);
        assert_eq!(Rotation::from(LogRotation::Hourly), Rotation::HOURLY);
        assert_eq!(Rotation::from(LogRotation::Minutely), Rotation::MINUTELY);
        assert_eq!(Rotation::from(LogRotation::Never), Rotation::NEVER);
    }
}
