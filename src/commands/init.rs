//! Init command - Interactive configuration wizard.

use crate::cli::args::{Cli, InitArgs};
use crate::config::loader::{get_global_config_path, save_global_config, save_project_config};
use crate::config::settings::{
    GoogleConfig, HistoryConfig, IndexNowConfig, LoggingConfig, Settings, SitemapConfig,
};
use crate::types::Result;
use anyhow::Context;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use std::path::{Path, PathBuf};

pub async fn run(args: InitArgs, _cli: &Cli) -> Result<()> {
    // Welcome message
    print_welcome();

    // Check for existing configuration
    let config_path = if args.global {
        get_global_config_path().context("Failed to determine global configuration path")?
    } else {
        PathBuf::from("indexer.yaml")
    };

    if config_path.exists() && !args.force {
        return Err(anyhow::anyhow!(
            "Configuration file already exists at: {}\nUse --force to overwrite",
            config_path.display()
        )
        .into());
    }

    let settings = if args.non_interactive {
        println!("{}", "\nCreating default configuration...\n".cyan());
        create_default_settings()
    } else {
        // Interactive wizard
        run_interactive_wizard(&args)?
    };

    // Save configuration
    let saved_path = if args.global {
        save_global_config(&settings).context("Failed to save global configuration")?
    } else {
        save_project_config(&settings).context("Failed to save project configuration")?
    };

    // Success message
    print_success(&saved_path, &settings);

    Ok(())
}

fn print_welcome() {
    println!();
    println!(
        "{}",
        "╔════════════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        "║                                                    ║".cyan()
    );
    println!(
        "{}",
        "║            Welcome to indexer-cli!                 ║"
            .cyan()
            .bold()
    );
    println!(
        "{}",
        "║                                                    ║".cyan()
    );
    println!(
        "{}",
        "╚════════════════════════════════════════════════════╝".cyan()
    );
    println!();
    println!("This wizard will help you set up your configuration.");
    println!();
}

fn run_interactive_wizard(args: &InitArgs) -> Result<Settings> {
    let theme = ColorfulTheme::default();
    let mut settings = Settings::default();

    // Ask for configuration scope (if not already specified)
    if !args.global {
        println!("{}", "Configuration Scope".cyan().bold());
        println!("{}", "━".repeat(60).dimmed());

        let scope_options = vec![
            "Project (./indexer.yaml) - Configuration for this project only",
            "Global (~/.indexer-cli/config.yaml) - Configuration for all projects",
        ];

        let scope = Select::with_theme(&theme)
            .with_prompt("Where should the configuration be saved?")
            .items(&scope_options)
            .default(0)
            .interact()
            .map_err(|e| anyhow::anyhow!("Failed to get user input: {}", e))?;

        if scope == 1 {
            // This is a bit hacky, but we'll handle it at the save stage
            println!(
                "{}",
                "Note: Use --global flag next time for global config".dimmed()
            );
        }
        println!();
    }

    // Google Indexing API setup
    println!("{}", "Google Indexing API Setup".cyan().bold());
    println!("{}", "━".repeat(60).dimmed());

    let setup_google = Confirm::with_theme(&theme)
        .with_prompt("Do you want to configure Google Indexing API?")
        .default(false)
        .interact()
        .map_err(|e| anyhow::anyhow!("Failed to get user input: {}", e))?;

    if setup_google {
        settings.google = Some(configure_google(&theme)?);
    }
    println!();

    // IndexNow API setup
    println!("{}", "IndexNow API Setup".cyan().bold());
    println!("{}", "━".repeat(60).dimmed());

    let setup_indexnow = Confirm::with_theme(&theme)
        .with_prompt("Do you want to configure IndexNow API?")
        .default(false)
        .interact()
        .map_err(|e| anyhow::anyhow!("Failed to get user input: {}", e))?;

    if setup_indexnow {
        settings.indexnow = Some(configure_indexnow(&theme)?);
    }
    println!();

    // Sitemap configuration
    println!("{}", "Sitemap Configuration".cyan().bold());
    println!("{}", "━".repeat(60).dimmed());

    let setup_sitemap = Confirm::with_theme(&theme)
        .with_prompt("Do you want to configure a default sitemap URL?")
        .default(false)
        .interact()
        .map_err(|e| anyhow::anyhow!("Failed to get user input: {}", e))?;

    if setup_sitemap {
        settings.sitemap = Some(configure_sitemap(&theme)?);
    }
    println!();

    // History settings
    println!("{}", "History Settings".cyan().bold());
    println!("{}", "━".repeat(60).dimmed());
    settings.history = configure_history(&theme)?;
    println!();

    // Logging preferences
    println!("{}", "Logging Preferences".cyan().bold());
    println!("{}", "━".repeat(60).dimmed());
    settings.logging = configure_logging(&theme)?;
    println!();

    Ok(settings)
}

fn configure_google(theme: &ColorfulTheme) -> Result<GoogleConfig> {
    println!(
        "{}",
        "  Google Indexing API requires a service account JSON file.".dimmed()
    );
    println!(
        "{}",
        "  Run 'indexer-cli google setup' for step-by-step guide".dimmed()
    );
    println!();

    let service_account_path: String = Input::with_theme(theme)
        .with_prompt("Path to Google service account JSON file")
        .validate_with(|input: &String| -> std::result::Result<(), &str> {
            let path = Path::new(input);
            if !path.exists() {
                Err("File does not exist")
            } else if path.extension().and_then(|s| s.to_str()) != Some("json") {
                Err("File must be a JSON file")
            } else {
                Ok(())
            }
        })
        .interact_text()
        .map_err(|e| anyhow::anyhow!("Failed to get user input: {}", e))?;

    let batch_size: String = Input::with_theme(theme)
        .with_prompt("Batch size for Google API requests")
        .default("100".to_string())
        .validate_with(|input: &String| -> std::result::Result<(), &str> {
            if input.parse::<usize>().is_ok() {
                Ok(())
            } else {
                Err("Must be a valid number")
            }
        })
        .interact_text()
        .map_err(|e| anyhow::anyhow!("Failed to get user input: {}", e))?;

    Ok(GoogleConfig {
        enabled: true,
        auth: crate::config::settings::GoogleAuthConfig {
            service_account_file: Some(PathBuf::from(&service_account_path)),
        },
        service_account_file: Some(PathBuf::from(service_account_path)),
        quota: Default::default(),
        batch_size: batch_size.parse().unwrap(),
    })
}

fn configure_indexnow(theme: &ColorfulTheme) -> Result<IndexNowConfig> {
    println!(
        "{}",
        "  IndexNow API requires an API key (8-128 characters).".dimmed()
    );
    println!(
        "{}",
        "  You can generate one with: indexer indexnow generate-key".dimmed()
    );
    println!();

    let has_key = Confirm::with_theme(theme)
        .with_prompt("Do you already have an IndexNow API key?")
        .default(false)
        .interact()
        .map_err(|e| anyhow::anyhow!("Failed to get user input: {}", e))?;

    let api_key = if has_key {
        Input::with_theme(theme)
            .with_prompt("IndexNow API key")
            .validate_with(|input: &String| -> std::result::Result<(), &str> {
                let len = input.len();
                if len < 8 {
                    Err("API key must be at least 8 characters")
                } else if len > 128 {
                    Err("API key must be at most 128 characters")
                } else {
                    Ok(())
                }
            })
            .interact_text()
            .map_err(|e| anyhow::anyhow!("Failed to get user input: {}", e))?
    } else {
        println!("{}", "  Generating a random API key...".yellow());
        generate_indexnow_key(32)
    };

    let key_location: String = Input::with_theme(theme)
        .with_prompt("Key file location URL (e.g., https://placeholder.test/YOUR_KEY.txt)")
        .validate_with(|input: &String| -> std::result::Result<(), &str> {
            if input.starts_with("http://") || input.starts_with("https://") {
                Ok(())
            } else {
                Err("Must be a valid HTTP/HTTPS URL")
            }
        })
        .interact_text()
        .map_err(|e| anyhow::anyhow!("Failed to get user input: {}", e))?;

    if !has_key {
        println!();
        println!(
            "{}",
            "  Important: Remember to upload the key file!"
                .yellow()
                .bold()
        );
        println!(
            "  Create a file named {}.txt with content: {}",
            api_key.bright_cyan(),
            api_key.bright_cyan()
        );
        println!("  Upload it to: {}", key_location.bright_cyan());
        println!();
    }

    let batch_size: String = Input::with_theme(theme)
        .with_prompt("Batch size for IndexNow requests")
        .default("1000".to_string())
        .validate_with(|input: &String| -> std::result::Result<(), &str> {
            match input.parse::<usize>() {
                Ok(n) if n > 0 && n <= 10000 => Ok(()),
                _ => Err("Must be between 1 and 10000"),
            }
        })
        .interact_text()
        .map_err(|e| anyhow::anyhow!("Failed to get user input: {}", e))?;

    Ok(IndexNowConfig {
        enabled: true,
        api_key,
        key_location,
        endpoints: vec![
            "https://api.indexnow.org/indexnow".to_string(),
            "https://www.bing.com/indexnow".to_string(),
            "https://yandex.com/indexnow".to_string(),
        ],
        batch_size: batch_size.parse().unwrap(),
    })
}

fn configure_sitemap(theme: &ColorfulTheme) -> Result<SitemapConfig> {
    let sitemap_url: String = Input::with_theme(theme)
        .with_prompt("Default sitemap URL")
        .validate_with(|input: &String| -> std::result::Result<(), &str> {
            if input.starts_with("http://") || input.starts_with("https://") {
                Ok(())
            } else {
                Err("Must be a valid HTTP/HTTPS URL")
            }
        })
        .interact_text()
        .map_err(|e| anyhow::anyhow!("Failed to get user input: {}", e))?;

    let follow_index = Confirm::with_theme(theme)
        .with_prompt("Follow sitemap index files automatically?")
        .default(true)
        .interact()
        .map_err(|e| anyhow::anyhow!("Failed to get user input: {}", e))?;

    Ok(SitemapConfig {
        url: sitemap_url,
        follow_index,
        filters: Default::default(),
    })
}

fn configure_history(theme: &ColorfulTheme) -> Result<HistoryConfig> {
    let enabled = Confirm::with_theme(theme)
        .with_prompt("Enable submission history tracking?")
        .default(true)
        .interact()
        .map_err(|e| anyhow::anyhow!("Failed to get user input: {}", e))?;

    if !enabled {
        return Ok(HistoryConfig {
            enabled: false,
            ..Default::default()
        });
    }

    let database_path: String = Input::with_theme(theme)
        .with_prompt("History database path")
        .default(
            crate::constants::default_database_file_path()
                .to_string_lossy()
                .to_string(),
        )
        .interact_text()
        .map_err(|e| anyhow::anyhow!("Failed to get user input: {}", e))?;

    let retention_days: String = Input::with_theme(theme)
        .with_prompt("History retention period (days)")
        .default("365".to_string())
        .validate_with(|input: &String| -> std::result::Result<(), &str> {
            if input.parse::<u32>().is_ok() {
                Ok(())
            } else {
                Err("Must be a valid number")
            }
        })
        .interact_text()
        .map_err(|e| anyhow::anyhow!("Failed to get user input: {}", e))?;

    Ok(HistoryConfig {
        enabled: true,
        database_path,
        retention_days: retention_days.parse().unwrap(),
    })
}

fn configure_logging(theme: &ColorfulTheme) -> Result<LoggingConfig> {
    let log_levels = vec!["error", "warn", "info", "debug", "trace"];

    let level_index = Select::with_theme(theme)
        .with_prompt("Select log level")
        .items(&log_levels)
        .default(2) // "info"
        .interact()
        .map_err(|e| anyhow::anyhow!("Failed to get user input: {}", e))?;

    let log_file: String = Input::with_theme(theme)
        .with_prompt("Log file path")
        .default(
            crate::constants::default_log_file_path()
                .to_string_lossy()
                .to_string(),
        )
        .interact_text()
        .map_err(|e| anyhow::anyhow!("Failed to get user input: {}", e))?;

    let max_size_mb: String = Input::with_theme(theme)
        .with_prompt("Maximum log file size (MB)")
        .default("10".to_string())
        .validate_with(|input: &String| -> std::result::Result<(), &str> {
            if input.parse::<u32>().is_ok() {
                Ok(())
            } else {
                Err("Must be a valid number")
            }
        })
        .interact_text()
        .map_err(|e| anyhow::anyhow!("Failed to get user input: {}", e))?;

    let max_backups: String = Input::with_theme(theme)
        .with_prompt("Maximum number of log backups")
        .default("5".to_string())
        .validate_with(|input: &String| -> std::result::Result<(), &str> {
            if input.parse::<u32>().is_ok() {
                Ok(())
            } else {
                Err("Must be a valid number")
            }
        })
        .interact_text()
        .map_err(|e| anyhow::anyhow!("Failed to get user input: {}", e))?;

    Ok(LoggingConfig {
        level: log_levels[level_index].to_string(),
        file: log_file,
        max_size_mb: max_size_mb.parse().unwrap(),
        max_backups: max_backups.parse().unwrap(),
    })
}

fn create_default_settings() -> Settings {
    Settings::default()
}

fn generate_indexnow_key(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-";
    let mut rng = rand::rng();

    (0..length)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

fn print_success(config_path: &Path, settings: &Settings) {
    println!();
    println!(
        "{}",
        "╔════════════════════════════════════════════════════╗".green()
    );
    println!(
        "{}",
        "║                                                    ║".green()
    );
    println!(
        "{}",
        "║          Configuration Created Successfully!       ║"
            .green()
            .bold()
    );
    println!(
        "{}",
        "║                                                    ║".green()
    );
    println!(
        "{}",
        "╚════════════════════════════════════════════════════╝".green()
    );
    println!();
    println!(
        "{} {}",
        "Configuration saved to:".bold(),
        config_path.display().to_string().cyan()
    );
    println!();

    // Show what was configured
    println!("{}", "Configured services:".bold());
    if settings.google.is_some() {
        println!("  {} Google Indexing API", "✓".green());
    }
    if settings.indexnow.is_some() {
        println!("  {} IndexNow API", "✓".green());
    }
    if settings.sitemap.is_some() {
        println!("  {} Default Sitemap", "✓".green());
    }
    println!();

    // Next steps
    println!("{}", "Next Steps:".cyan().bold());
    println!("{}", "━".repeat(60).dimmed());

    if settings.google.is_some() {
        println!("  1. Verify Google setup:");
        println!("     {}", "indexer google verify".bright_white());
        println!();
    }

    if settings.indexnow.is_some() {
        println!(
            "  {}. Upload your IndexNow key file:",
            if settings.google.is_some() { "2" } else { "1" }
        );
        if let Some(ref indexnow) = settings.indexnow {
            println!("     Key: {}", indexnow.api_key.bright_cyan());
            println!("     Location: {}", indexnow.key_location.bright_cyan());
        }
        println!();
        println!(
            "  {}. Verify IndexNow setup:",
            if settings.google.is_some() { "3" } else { "2" }
        );
        println!("     {}", "indexer indexnow verify".bright_white());
        println!();
    }

    let next_num = match (settings.google.is_some(), settings.indexnow.is_some()) {
        (true, true) => "4",
        (true, false) | (false, true) => "3",
        (false, false) => "1",
    };

    println!("  {}. Start submitting URLs:", next_num);
    println!(
        "     {}",
        "indexer submit --sitemap https://placeholder.test/sitemap.xml".bright_white()
    );
    println!(
        "     {}",
        "indexer submit https://placeholder.test/page1 https://placeholder.test/page2"
            .bright_white()
    );
    println!();

    println!("{}", "For more information, run: indexer --help".dimmed());
    println!();
}
