// Configuration module

pub mod loader;
pub mod settings;
pub mod validation;

// Re-export commonly used types and functions for convenience
pub use loader::{
    ensure_config_dir, expand_tilde, find_project_config, get_config_dir, get_global_config_path,
    load_config, load_from_file, merge_configs, save_global_config, save_project_config,
    save_to_file,
};

pub use settings::{
    GoogleConfig, HistoryConfig, IndexNowConfig, LoggingConfig, OutputConfig, QuotaConfig,
    RetryConfig, Settings, SitemapConfig, SitemapFilters,
};

pub use validation::{validate_config, validate_google_config, validate_indexnow_config, ValidationReport};
