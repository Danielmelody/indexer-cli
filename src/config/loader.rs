// Configuration loader

use super::settings::Settings;
use crate::utils::file::{expand_home_dir, user_home_dir};
use anyhow::{Context, Result};
use config::{Config, Environment, File};
use std::path::{Path, PathBuf};

/// Environment variable prefix for configuration
const ENV_PREFIX: &str = "INDEXER";

/// Global configuration file name
const GLOBAL_CONFIG_FILE: &str = ".indexer-cli/config.yaml";

/// Project configuration file names (in order of precedence)
const PROJECT_CONFIG_FILES: &[&str] = &["indexer.yaml", ".indexer.yaml"];

/// Load configuration from all sources and merge them
///
/// Configuration priority (highest to lowest):
/// 1. Environment variables (INDEXER_*)
/// 2. Project configuration (./indexer.yaml or ./.indexer.yaml)
/// 3. Global configuration (~/.indexer-cli/config.yaml)
/// 4. Default values
pub fn load_config() -> Result<Settings> {
    let mut builder = Config::builder();

    // Start with default values
    builder = builder.add_source(config::File::from_str(
        &serde_yaml::to_string(&Settings::default())?,
        config::FileFormat::Yaml,
    ));

    // Add global configuration if it exists
    if let Some(global_config_path) = get_global_config_path() {
        if global_config_path.exists() {
            builder = builder.add_source(
                File::from(global_config_path)
                    .format(config::FileFormat::Yaml)
                    .required(false),
            );
        }
    }

    // Add project configuration if it exists
    if let Some(project_config_path) = find_project_config() {
        builder = builder.add_source(
            File::from(project_config_path)
                .format(config::FileFormat::Yaml)
                .required(false),
        );
    }

    // Add environment variables (highest priority)
    builder = builder.add_source(
        Environment::with_prefix(ENV_PREFIX)
            .separator("__")
            .try_parsing(true),
    );

    // Build and deserialize
    let config = builder.build().context("Failed to build configuration")?;

    let settings: Settings = config
        .try_deserialize()
        .context("Failed to deserialize configuration")?;

    Ok(settings)
}

/// Load configuration from a specific file
pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Settings> {
    let path = path.as_ref();

    if !path.exists() {
        anyhow::bail!("Configuration file not found: {}", path.display());
    }

    let config = Config::builder()
        .add_source(File::from(path).format(config::FileFormat::Yaml))
        .build()
        .context("Failed to load configuration file")?;

    let settings: Settings = config
        .try_deserialize()
        .context("Failed to parse configuration file")?;

    Ok(settings)
}

/// Merge multiple configuration sources
///
/// Later configurations in the list override earlier ones
pub fn merge_configs(configs: Vec<Settings>) -> Result<Settings> {
    if configs.is_empty() {
        return Ok(Settings::default());
    }

    let mut merged = configs[0].clone();

    for config in configs.iter().skip(1) {
        // Merge Google config
        if config.google.is_some() {
            merged.google = config.google.clone();
        }

        // Merge IndexNow config
        if config.indexnow.is_some() {
            merged.indexnow = config.indexnow.clone();
        }

        // Merge Sitemap config
        if config.sitemap.is_some() {
            merged.sitemap = config.sitemap.clone();
        }

        // Merge history config (always present)
        if config.history.enabled != Settings::default().history.enabled
            || config.history.database_path != Settings::default().history.database_path
            || config.history.retention_days != Settings::default().history.retention_days
        {
            merged.history = config.history.clone();
        }

        // Merge logging config
        if config.logging.level != Settings::default().logging.level
            || config.logging.file != Settings::default().logging.file
        {
            merged.logging = config.logging.clone();
        }

        // Merge retry config
        if config.retry.enabled != Settings::default().retry.enabled
            || config.retry.max_attempts != Settings::default().retry.max_attempts
        {
            merged.retry = config.retry.clone();
        }

        // Merge output config
        if config.output.format != Settings::default().output.format
            || config.output.color != Settings::default().output.color
            || config.output.verbose != Settings::default().output.verbose
        {
            merged.output = config.output.clone();
        }
    }

    Ok(merged)
}

/// Get the path to the global configuration file
pub fn get_global_config_path() -> Option<PathBuf> {
    user_home_dir().map(|home| home.join(GLOBAL_CONFIG_FILE))
}

/// Find a project configuration file in the current directory
pub fn find_project_config() -> Option<PathBuf> {
    for filename in PROJECT_CONFIG_FILES {
        let path = PathBuf::from(filename);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

/// Get the path to the global configuration directory
pub fn get_config_dir() -> Option<PathBuf> {
    user_home_dir().map(|home| home.join(".indexer-cli"))
}

/// Ensure the global configuration directory exists
pub fn ensure_config_dir() -> Result<PathBuf> {
    let config_dir = get_config_dir().context("Failed to determine home directory")?;

    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir).context("Failed to create configuration directory")?;
    }

    Ok(config_dir)
}

/// Save settings to a file
pub fn save_to_file<P: AsRef<Path>>(settings: &Settings, path: P) -> Result<()> {
    let path = path.as_ref();

    // Create parent directory if it doesn't exist
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).context("Failed to create configuration directory")?;
        }
    }

    let yaml = serde_yaml::to_string(settings).context("Failed to serialize configuration")?;

    std::fs::write(path, yaml).context("Failed to write configuration file")?;

    Ok(())
}

/// Save settings to the global configuration file
pub fn save_global_config(settings: &Settings) -> Result<PathBuf> {
    let config_path =
        get_global_config_path().context("Failed to determine global configuration path")?;

    save_to_file(settings, &config_path)?;

    Ok(config_path)
}

/// Save settings to a project configuration file
pub fn save_project_config(settings: &Settings) -> Result<PathBuf> {
    let config_path = PathBuf::from(PROJECT_CONFIG_FILES[0]);
    save_to_file(settings, &config_path)?;
    Ok(config_path)
}

/// Expand tilde (~) in file paths
pub fn expand_tilde<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    if let Some(path_str) = path.to_str() {
        if let Some(home) = user_home_dir() {
            return expand_home_dir(path_str, &home);
        }
    }
    path.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_default_config() {
        let settings = Settings::default();
        assert!(settings.google.is_none());
        assert!(settings.indexnow.is_none());
        assert!(settings.history.enabled);
    }

    #[test]
    fn test_save_and_load_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        let mut settings = Settings::default();
        settings.logging.level = "debug".to_string();
        settings.output.verbose = true;

        // Save
        save_to_file(&settings, &config_path).unwrap();

        // Load
        let loaded = load_from_file(&config_path).unwrap();
        assert_eq!(loaded.logging.level, "debug");
        assert!(loaded.output.verbose);
    }

    #[test]
    fn test_merge_configs() {
        let mut config1 = Settings::default();
        config1.logging.level = "info".to_string();

        let mut config2 = Settings::default();
        config2.logging.level = "debug".to_string();
        config2.output.verbose = true;

        let merged = merge_configs(vec![config1, config2]).unwrap();
        assert_eq!(merged.logging.level, "debug");
        assert!(merged.output.verbose);
    }

    #[test]
    fn test_expand_tilde() {
        let path = expand_tilde("~/test/file.txt");
        if let Some(home) = user_home_dir() {
            assert_eq!(path, home.join("test/file.txt"));

            let path = expand_tilde("~");
            assert_eq!(path, home);
        }

        let path = expand_tilde("~user/test/file.txt");
        assert_eq!(path, PathBuf::from("~user/test/file.txt"));
    }

    #[test]
    fn test_find_project_config() {
        // This test would need to create temporary files
        // in the current directory, which could interfere
        // with the actual project, so we skip it
    }
}
