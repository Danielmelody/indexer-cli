//! IndexNow command - IndexNow API operations.

use crate::api::indexnow::IndexNowClient;
use crate::cli::args::{
    Cli, IndexNowArgs, IndexNowCommand, IndexNowEndpoint, IndexNowGenerateKeyArgs,
    IndexNowSetupArgs, IndexNowSubmitArgs, SubmitArgs, ApiTarget, OutputFormat,
};
use crate::config::loader::{load_config, save_global_config, save_project_config};
use crate::config::settings::IndexNowConfig;
use crate::types::error::IndexerError;
use crate::types::Result;
use crate::utils::file::write_file_sync;
use colored::Colorize;
use dialoguer::Input;


pub async fn run(args: IndexNowArgs, cli: &Cli) -> Result<()> {
    match args.command {
        IndexNowCommand::Setup(setup_args) => setup(setup_args, cli).await,
        IndexNowCommand::GenerateKey(gen_args) => generate_key(gen_args, cli).await,
        IndexNowCommand::Submit(submit_args) => submit(submit_args, cli).await,
        IndexNowCommand::Verify => verify(cli).await,
    }
}

/// Interactive setup for IndexNow API
pub async fn setup(args: IndexNowSetupArgs, _cli: &Cli) -> Result<()> {
    println!("{}", "IndexNow API Setup".cyan().bold());
    println!("{}", "=".repeat(60).dimmed());
    println!();

    // Use provided key or prompt for it
    let api_key = args.key;

    // Validate key
    if api_key.len() < 8 || api_key.len() > 128 {
        return Err(IndexerError::InvalidApiKey {
            message: format!(
                "API key length must be between 8 and 128 characters (got: {})",
                api_key.len()
            ),
        });
    }

    if !api_key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err(IndexerError::InvalidApiKey {
            message: "API key must contain only alphanumeric characters and hyphens".to_string(),
        });
    }

    println!("{}", "✓ API key validated".green());
    println!();

    // Get key location
    let key_location = if let Some(location) = args.key_location {
        location
    } else {
        Input::new()
            .with_prompt("Key file location URL (e.g., https://example.com/your-key.txt)")
            .interact()
            .map_err(|e| IndexerError::InternalError {
                message: format!("Failed to read input: {}", e),
            })?
    };

    // Validate key_location is a valid URL
    url::Url::parse(&key_location).map_err(|_| IndexerError::InvalidUrl {
        url: key_location.clone(),
    })?;

    println!("{}", "✓ Key location validated".green());
    println!();

    // Load existing config or create new
    let mut config = load_config().unwrap_or_default();

    // Update IndexNow config
    config.indexnow = Some(IndexNowConfig {
        enabled: true,
        api_key: api_key.clone(),
        key_location,
        endpoints: vec![
            "https://api.indexnow.org/indexnow".to_string(),
            "https://www.bing.com/indexnow".to_string(),
            "https://yandex.com/indexnow".to_string(),
        ],
        batch_size: 10000,
    });

    // Save config
    let config_path = if args.global {
        save_global_config(&config)?
    } else {
        save_project_config(&config)?
    };

    println!("{}", "✓ Configuration saved!".green());
    println!("  Location: {}", config_path.display().to_string().dimmed());
    println!();

    // Test connectivity
    println!("{}", "Testing connectivity to IndexNow endpoints...".dimmed());
    println!();

    let client = IndexNowClient::new(
        config.indexnow.as_ref().unwrap().api_key.clone(),
        config.indexnow.as_ref().unwrap().key_location.clone(),
        config.indexnow.as_ref().unwrap().endpoints.clone(),
    )?;

    // Extract host from key_location
    let key_location_url = url::Url::parse(&config.indexnow.as_ref().unwrap().key_location)?;
    let host = key_location_url
        .host_str()
        .ok_or_else(|| IndexerError::InvalidUrl {
            url: config.indexnow.as_ref().unwrap().key_location.clone(),
        })?;

    // Test with a simple URL
    let test_url = format!("https://{}/", host);
    let results = client.submit_to_all(&[test_url]).await;

    let mut all_ok = true;
    for result in &results {
        match result {
            Ok(resp) if resp.is_success() => {
                println!("  {} {} - {}", "✓".green(), resp.endpoint, "OK".green());
            }
            Ok(resp) => {
                println!(
                    "  {} {} - {} (HTTP {})",
                    "✗".red(),
                    resp.endpoint,
                    resp.message.yellow(),
                    resp.status_code
                );
                all_ok = false;
            }
            Err(e) => {
                println!("  {} Error - {}", "✗".red(), e.to_string().red());
                all_ok = false;
            }
        }
    }

    println!();
    if all_ok {
        println!("{}", "✓ Setup complete! IndexNow is ready to use.".green().bold());
    } else {
        println!(
            "{}",
            "⚠ Setup complete but some endpoints failed. Check the errors above.".yellow().bold()
        );
    }
    println!();
    println!("Next steps:");
    println!("  • Run 'indexer indexnow verify' to verify the setup");
    println!("  • Run 'indexer indexnow submit <url>' to submit URLs");
    println!(
        "  • Make sure the key file is accessible at: {}",
        config.indexnow.as_ref().unwrap().key_location.dimmed()
    );

    Ok(())
}

/// Generate a new IndexNow API key
pub async fn generate_key(args: IndexNowGenerateKeyArgs, _cli: &Cli) -> Result<()> {
    println!("{}", "Generating IndexNow API Key".cyan().bold());
    println!("{}", "=".repeat(60).dimmed());
    println!();

    // Validate length
    if args.length < 8 || args.length > 128 {
        return Err(IndexerError::ValueOutOfRange {
            field: "length".to_string(),
            value: args.length.to_string(),
            min: "8".to_string(),
            max: "128".to_string(),
        });
    }

    // Generate key
    let api_key = IndexNowClient::generate_key(args.length)?;

    println!("Generated API Key (length: {}):", args.length);
    println!();
    println!("  {}", api_key.cyan().bold());
    println!();

    // Save to file if output directory is specified
    if let Some(output_dir) = &args.output {
        let key_file_path = output_dir.join(format!("{}.txt", api_key));
        write_file_sync(&key_file_path, &api_key)?;
        println!("{}", "✓ Key file saved!".green());
        println!("  Location: {}", key_file_path.display().to_string().dimmed());
        println!();
    }

    // Save to configuration if requested
    if args.save {
        let key_location: String = Input::new()
            .with_prompt("Key file location URL (e.g., https://example.com/your-key.txt)")
            .interact()
            .map_err(|e| IndexerError::InternalError {
                message: format!("Failed to read input: {}", e),
            })?;

        // Validate key_location
        url::Url::parse(&key_location).map_err(|_| IndexerError::InvalidUrl {
            url: key_location.clone(),
        })?;

        let mut config = load_config().unwrap_or_default();

        config.indexnow = Some(IndexNowConfig {
            enabled: true,
            api_key: api_key.clone(),
            key_location,
            endpoints: vec![
                "https://api.indexnow.org/indexnow".to_string(),
                "https://www.bing.com/indexnow".to_string(),
                "https://yandex.com/indexnow".to_string(),
            ],
            batch_size: 10000,
        });

        let config_path = if args.global {
            save_global_config(&config)?
        } else {
            save_project_config(&config)?
        };

        println!("{}", "✓ Configuration saved!".green());
        println!("  Location: {}", config_path.display().to_string().dimmed());
        println!();
    }

    println!("Important:");
    println!("  • Upload the key file to your web server");
    println!(
        "  • Make it accessible at: https://yourdomain.com/{}.txt",
        api_key.dimmed()
    );
    println!("  • The file should contain only the key (no extra characters)");

    Ok(())
}

/// Submit URLs to IndexNow (wrapper around main submit)
pub async fn submit(args: IndexNowSubmitArgs, cli: &Cli) -> Result<()> {
    // Map endpoint argument
    let api_target = match args.endpoint {
        IndexNowEndpoint::All
        | IndexNowEndpoint::Bing
        | IndexNowEndpoint::Yandex
        | IndexNowEndpoint::Seznam
        | IndexNowEndpoint::Naver
        | IndexNowEndpoint::IndexNow => ApiTarget::IndexNow, // All use IndexNow protocol
    };

    // Convert to main SubmitArgs
    let submit_args = SubmitArgs {
        urls: args.urls,
        file: args.file,
        sitemap: args.sitemap,
        api: api_target,
        filter: args.filter,
        since: args.since,
        google_action: crate::cli::args::GoogleAction::UrlUpdated, // Not used for IndexNow
        batch_size: args.batch_size,
        dry_run: args.dry_run,
        skip_history: args.skip_history,
        format: OutputFormat::Text,
    };

    // Delegate to main submit command
    crate::commands::submit::run(submit_args, cli).await
}

/// Verify IndexNow configuration and connectivity
pub async fn verify(_cli: &Cli) -> Result<()> {
    println!("{}", "Verifying IndexNow Configuration".cyan().bold());
    println!("{}", "=".repeat(60).dimmed());
    println!();

    // Check config exists
    let config = match load_config() {
        Ok(c) => {
            println!("{}", "✓ Configuration found".green());
            c
        }
        Err(_) => {
            println!("{}", "✗ Configuration not found".red());
            println!("  {}", "Run 'indexer indexnow setup' to configure".dimmed());
            return Err(IndexerError::ConfigMissingField {
                field: "configuration".to_string(),
            });
        }
    };

    // Check IndexNow config
    let indexnow_config = match config.indexnow {
        Some(ref cfg) if cfg.enabled => {
            println!("{}", "✓ IndexNow configuration enabled".green());
            cfg
        }
        _ => {
            println!("{}", "✗ IndexNow not configured or disabled".red());
            println!("  {}", "Run 'indexer indexnow setup' to configure".dimmed());
            return Err(IndexerError::ConfigMissingField {
                field: "indexnow".to_string(),
            });
        }
    };

    println!();
    println!("Configuration Details:");
    println!("  API Key: {}...{}",
        &indexnow_config.api_key[..8.min(indexnow_config.api_key.len())],
        if indexnow_config.api_key.len() > 16 {
            &indexnow_config.api_key[indexnow_config.api_key.len()-8..]
        } else {
            ""
        }
    );
    println!("  Key Length: {} characters", indexnow_config.api_key.len());
    println!("  Key Location: {}", indexnow_config.key_location);
    println!("  Endpoints: {} configured", indexnow_config.endpoints.len());
    println!("  Batch Size: {}", indexnow_config.batch_size);

    // Validate key format
    println!();
    print!("{}", "Validating API key format... ".dimmed());
    if indexnow_config.api_key.len() >= 8
        && indexnow_config.api_key.len() <= 128
        && indexnow_config.api_key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        println!("{}", "✓".green());
    } else {
        println!("{}", "✗".red());
        println!();
        println!("{}", "⚠ Invalid key format".yellow());
        println!("  Key must be 8-128 alphanumeric characters (hyphens allowed)");
        return Err(IndexerError::InvalidApiKey {
            message: "Key format validation failed".to_string(),
        });
    }

    // Test key file accessibility
    println!();
    println!("{}", "Testing key file accessibility...".dimmed());

    let client = IndexNowClient::new(
        indexnow_config.api_key.clone(),
        indexnow_config.key_location.clone(),
        indexnow_config.endpoints.clone(),
    )?;

    // Extract host from key_location
    let key_location_url = url::Url::parse(&indexnow_config.key_location)?;
    let host = key_location_url
        .host_str()
        .ok_or_else(|| IndexerError::InvalidUrl {
            url: indexnow_config.key_location.clone(),
        })?;

    match client.verify_key_file(host).await {
        Ok(_) => {
            println!("  {} Key file verified successfully", "✓".green());
        }
        Err(e) => {
            println!("  {} Key file verification failed: {}", "✗".red(), e);
            println!();
            println!("{}", "⚠ Key file is not accessible or doesn't match the API key".yellow());
            println!("  Make sure to upload the key file to: {}", indexnow_config.key_location);
        }
    }

    // Test connectivity to endpoints
    println!();
    println!("{}", "Testing connectivity to IndexNow endpoints...".dimmed());
    println!();

    let test_url = format!("https://{}/", host);
    let results = client.submit_to_all(&[test_url]).await;

    let mut all_ok = true;
    let mut successful_endpoints = 0;

    for result in &results {
        match result {
            Ok(resp) if resp.is_success() => {
                println!("  {} {} - {}", "✓".green(), resp.endpoint, "OK".green());
                successful_endpoints += 1;
            }
            Ok(resp) => {
                println!(
                    "  {} {} - {} (HTTP {})",
                    "✗".red(),
                    resp.endpoint,
                    resp.message.yellow(),
                    resp.status_code
                );
                all_ok = false;
            }
            Err(e) => {
                println!("  {} Error - {}", "✗".red(), e.to_string().red());
                all_ok = false;
            }
        }
    }

    println!();
    println!("Summary:");
    println!("  Successful endpoints: {}/{}", successful_endpoints, results.len());

    if all_ok {
        println!();
        println!("{}", "✓ All checks passed!".green().bold());
        println!("  {}", "IndexNow is ready to use".green());
    } else {
        println!();
        println!("{}", "⚠ Some checks failed".yellow().bold());
        println!("  {}", "IndexNow may work with some endpoints".yellow());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_validation() {
        // Valid keys
        assert!(validate_key("12345678").is_ok());
        assert!(validate_key("a1b2c3d4e5f6g7h8").is_ok());
        assert!(validate_key("key-with-hyphens-123").is_ok());

        // Invalid - too short
        assert!(validate_key("1234567").is_err());

        // Invalid - too long
        let long_key = "a".repeat(129);
        assert!(validate_key(&long_key).is_err());

        // Invalid - special characters
        assert!(validate_key("key_with_underscore").is_err());
        assert!(validate_key("key with spaces").is_err());
    }

    fn validate_key(key: &str) -> Result<()> {
        if key.len() < 8 || key.len() > 128 {
            return Err(IndexerError::InvalidApiKey {
                message: "Invalid length".to_string(),
            });
        }

        if !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(IndexerError::InvalidApiKey {
                message: "Invalid characters".to_string(),
            });
        }

        Ok(())
    }
}
