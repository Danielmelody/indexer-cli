// Configuration validation

use super::loader::expand_tilde;
use super::settings::{GoogleConfig, IndexNowConfig, Settings};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Controls which parts of the configuration should be validated.
#[derive(Debug, Clone, Copy)]
pub struct ValidationOptions {
    pub validate_google: bool,
    pub validate_indexnow: bool,
    pub validate_common: bool,
}

impl ValidationOptions {
    /// Validate all sections (default behavior).
    pub fn all() -> Self {
        Self {
            validate_google: true,
            validate_indexnow: true,
            validate_common: true,
        }
    }

    /// Validate only Google related settings.
    pub fn google_only() -> Self {
        Self {
            validate_google: true,
            validate_indexnow: false,
            validate_common: false,
        }
    }

    /// Validate only IndexNow related settings.
    pub fn indexnow_only() -> Self {
        Self {
            validate_google: false,
            validate_indexnow: true,
            validate_common: false,
        }
    }
}

impl Default for ValidationOptions {
    fn default() -> Self {
        Self::all()
    }
}

/// Validate the entire configuration
pub fn validate_config(settings: &Settings) -> Result<ValidationReport> {
    let report = build_validation_report(settings, &ValidationOptions::default())?;

    if report.has_errors() {
        anyhow::bail!("Configuration validation failed:\n{}", report);
    }

    Ok(report)
}

/// Build a validation report according to the provided options.
pub fn build_validation_report(
    settings: &Settings,
    options: &ValidationOptions,
) -> Result<ValidationReport> {
    let mut report = ValidationReport::new();

    if options.validate_google {
        // Validate Google config if present
        if let Some(ref google_config) = settings.google {
            if google_config.enabled {
                match validate_google_config(google_config) {
                    Ok(_) => report.add_success("Google Indexing API configuration is valid"),
                    Err(e) => report.add_error(&format!("Google Indexing API: {}", e)),
                }
            } else {
                report.add_info("Google Indexing API is disabled");
            }
        } else {
            report.add_warning("Google Indexing API is not configured");
        }
    }

    if options.validate_indexnow {
        // Validate IndexNow config if present
        if let Some(ref indexnow_config) = settings.indexnow {
            if indexnow_config.enabled {
                match validate_indexnow_config(indexnow_config) {
                    Ok(_) => report.add_success("IndexNow API configuration is valid"),
                    Err(e) => report.add_error(&format!("IndexNow API: {}", e)),
                }
            } else {
                report.add_info("IndexNow API is disabled");
            }
        } else {
            report.add_warning("IndexNow API is not configured");
        }
    }

    if options.validate_common {
        // Validate file paths
        validate_file_paths(settings, &mut report)?;

        // Validate numeric ranges
        validate_numeric_ranges(settings, &mut report)?;

        // Validate log level
        validate_log_level(&settings.logging.level, &mut report)?;

        // Validate output format
        validate_output_format(&settings.output.format, &mut report)?;
    }

    Ok(report)
}

/// Validate Google Indexing API configuration
pub fn validate_google_config(config: &GoogleConfig) -> Result<()> {
    // Check if service account file exists
    if let Some(service_account_file) = &config.service_account_file {
        let service_account_path = expand_tilde(service_account_file);

        if !service_account_path.exists() {
            anyhow::bail!(
                "Service account file not found: {}",
                service_account_path.display()
            );
        }

        // Validate service account file is valid JSON
        validate_service_account_file(&service_account_path)?;
    }

    // Validate quota limits
    if config.quota.daily_limit == 0 {
        anyhow::bail!("Daily limit must be greater than 0");
    }

    if config.quota.rate_limit == 0 {
        anyhow::bail!("Rate limit must be greater than 0");
    }

    // Validate batch size
    if config.batch_size == 0 {
        anyhow::bail!("Batch size must be greater than 0");
    }

    if config.batch_size > 1000 {
        anyhow::bail!("Batch size should not exceed 1000 for optimal performance");
    }

    Ok(())
}

/// Validate IndexNow API configuration
pub fn validate_indexnow_config(config: &IndexNowConfig) -> Result<()> {
    // Validate API key format (8-128 characters, alphanumeric)
    if config.api_key.is_empty() {
        anyhow::bail!("IndexNow API key is required");
    }

    if config.api_key.len() < 8 || config.api_key.len() > 128 {
        anyhow::bail!(
            "IndexNow API key must be between 8 and 128 characters (current: {})",
            config.api_key.len()
        );
    }

    // Validate key is alphanumeric
    if !config.api_key.chars().all(|c| c.is_ascii_alphanumeric()) {
        anyhow::bail!("IndexNow API key must contain only alphanumeric characters");
    }

    // Validate key location URL
    if config.key_location.is_empty() {
        anyhow::bail!("IndexNow key location URL is required");
    }

    validate_url(&config.key_location).context("Invalid key location URL")?;

    // Validate endpoints
    if config.endpoints.is_empty() {
        anyhow::bail!("At least one IndexNow endpoint is required");
    }

    for endpoint in &config.endpoints {
        validate_url(endpoint).with_context(|| format!("Invalid endpoint URL: {}", endpoint))?;
    }

    // Validate batch size
    if config.batch_size == 0 {
        anyhow::bail!("Batch size must be greater than 0");
    }

    if config.batch_size > 10000 {
        anyhow::bail!("IndexNow batch size should not exceed 10,000");
    }

    Ok(())
}

/// Validate a service account JSON file
fn validate_service_account_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    let contents = fs::read_to_string(path)
        .with_context(|| format!("Failed to read service account file: {}", path.display()))?;

    let json: serde_json::Value =
        serde_json::from_str(&contents).context("Service account file is not valid JSON")?;

    // Check for required fields
    let required_fields = [
        "type",
        "project_id",
        "private_key_id",
        "private_key",
        "client_email",
        "client_id",
    ];

    for field in &required_fields {
        if !json.get(field).is_some() {
            anyhow::bail!("Service account file is missing required field: {}", field);
        }
    }

    // Verify it's a service account
    if json.get("type").and_then(|v| v.as_str()) != Some("service_account") {
        anyhow::bail!("JSON file is not a valid service account key");
    }

    Ok(())
}

/// Validate file paths in configuration
fn validate_file_paths(settings: &Settings, report: &mut ValidationReport) -> Result<()> {
    // Validate history database path parent directory
    let db_path = expand_tilde(&settings.history.database_path);
    if let Some(parent) = db_path.parent() {
        if !parent.exists() {
            report.add_warning(&format!(
                "Database directory does not exist (will be created): {}",
                parent.display()
            ));
        }
    }

    // Validate log file path parent directory
    let log_path = expand_tilde(&settings.logging.file);
    if let Some(parent) = log_path.parent() {
        if !parent.exists() {
            report.add_warning(&format!(
                "Log directory does not exist (will be created): {}",
                parent.display()
            ));
        }
    }

    Ok(())
}

/// Validate numeric ranges in configuration
fn validate_numeric_ranges(settings: &Settings, report: &mut ValidationReport) -> Result<()> {
    // Validate retention days
    if settings.history.retention_days == 0 {
        report.add_error("History retention days must be greater than 0");
    }

    // Validate logging max size
    if settings.logging.max_size_mb == 0 {
        report.add_error("Log file max size must be greater than 0");
    }

    // Validate retry settings
    if settings.retry.max_attempts == 0 {
        report.add_error("Retry max attempts must be greater than 0");
    }

    if settings.retry.backoff_factor == 0 {
        report.add_error("Retry backoff factor must be greater than 0");
    }

    if settings.retry.max_wait_seconds == 0 {
        report.add_error("Retry max wait seconds must be greater than 0");
    }

    Ok(())
}

/// Validate log level
fn validate_log_level(level: &str, report: &mut ValidationReport) -> Result<()> {
    let valid_levels = ["trace", "debug", "info", "warn", "error"];
    if !valid_levels.contains(&level.to_lowercase().as_str()) {
        report.add_error(&format!(
            "Invalid log level: {}. Must be one of: {}",
            level,
            valid_levels.join(", ")
        ));
    }
    Ok(())
}

/// Validate output format
fn validate_output_format(format: &str, report: &mut ValidationReport) -> Result<()> {
    let valid_formats = ["text", "json", "csv"];
    if !valid_formats.contains(&format.to_lowercase().as_str()) {
        report.add_error(&format!(
            "Invalid output format: {}. Must be one of: {}",
            format,
            valid_formats.join(", ")
        ));
    }
    Ok(())
}

/// Validate a URL
fn validate_url(url: &str) -> Result<()> {
    url::Url::parse(url).with_context(|| format!("Invalid URL: {}", url))?;
    Ok(())
}

/// Validation report
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub successes: Vec<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub info: Vec<String>,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            successes: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
            info: Vec::new(),
        }
    }

    pub fn add_success(&mut self, message: &str) {
        self.successes.push(message.to_string());
    }

    pub fn add_warning(&mut self, message: &str) {
        self.warnings.push(message.to_string());
    }

    pub fn add_error(&mut self, message: &str) {
        self.errors.push(message.to_string());
    }

    pub fn add_info(&mut self, message: &str) {
        self.info.push(message.to_string());
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub fn is_valid(&self) -> bool {
        !self.has_errors()
    }
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ValidationReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.successes.is_empty() {
            writeln!(f, "\nSuccesses:")?;
            for success in &self.successes {
                writeln!(f, "  ✓ {}", success)?;
            }
        }

        if !self.info.is_empty() {
            writeln!(f, "\nInformation:")?;
            for info in &self.info {
                writeln!(f, "  ℹ {}", info)?;
            }
        }

        if !self.warnings.is_empty() {
            writeln!(f, "\nWarnings:")?;
            for warning in &self.warnings {
                writeln!(f, "  ⚠ {}", warning)?;
            }
        }

        if !self.errors.is_empty() {
            writeln!(f, "\nErrors:")?;
            for error in &self.errors {
                writeln!(f, "  ✗ {}", error)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("http://example.com/path").is_ok());
        assert!(validate_url("not a url").is_err());
        assert!(validate_url("").is_err());
    }

    #[test]
    fn test_validation_report() {
        let mut report = ValidationReport::new();
        assert!(report.is_valid());

        report.add_success("Test success");
        assert!(report.is_valid());

        report.add_warning("Test warning");
        assert!(report.is_valid());
        assert!(report.has_warnings());

        report.add_error("Test error");
        assert!(!report.is_valid());
        assert!(report.has_errors());
    }

    #[test]
    fn test_validate_indexnow_api_key() {
        // Valid key
        let mut config = IndexNowConfig::default();
        config.api_key = "a1b2c3d4e5f6g7h8".to_string();
        config.key_location = "https://example.com/key.txt".to_string();
        config.endpoints = vec!["https://api.indexnow.org/indexnow".to_string()];
        assert!(validate_indexnow_config(&config).is_ok());

        // Too short
        let mut config = IndexNowConfig::default();
        config.api_key = "short".to_string();
        config.key_location = "https://example.com/key.txt".to_string();
        assert!(validate_indexnow_config(&config).is_err());

        // Non-alphanumeric
        let mut config = IndexNowConfig::default();
        config.api_key = "invalid-key-with-dashes".to_string();
        config.key_location = "https://example.com/key.txt".to_string();
        assert!(validate_indexnow_config(&config).is_err());
    }

    #[test]
    fn test_build_report_google_only_scope() {
        let mut settings = Settings::default();
        settings.google = Some(GoogleConfig::default());

        let report = build_validation_report(&settings, &ValidationOptions::google_only()).unwrap();

        assert!(report
            .successes
            .iter()
            .any(|msg| msg.contains("Google Indexing API")));
        assert!(report
            .warnings
            .iter()
            .all(|msg| !msg.contains("IndexNow API")));
    }

    #[test]
    fn test_build_report_indexnow_only_scope() {
        let mut settings = Settings::default();
        let mut indexnow = IndexNowConfig::default();
        indexnow.api_key = "abcd1234".to_string();
        indexnow.key_location = "https://example.com/abcd1234.txt".to_string();
        indexnow.endpoints = vec!["https://api.indexnow.org/indexnow".to_string()];
        settings.indexnow = Some(indexnow);

        let report =
            build_validation_report(&settings, &ValidationOptions::indexnow_only()).unwrap();

        assert!(report
            .successes
            .iter()
            .any(|msg| msg.contains("IndexNow API")));
        assert!(report
            .warnings
            .iter()
            .all(|msg| !msg.contains("Google Indexing API")));
    }
}
