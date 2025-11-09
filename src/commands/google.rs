//! Google command - Google Indexing API operations.

use crate::api::google_indexing::{GoogleIndexingClient, MetadataResponse};
use crate::auth::oauth::GoogleOAuthFlow;
use crate::cli::args::{Cli, GoogleArgs, GoogleCommand, GoogleAction, GoogleAuthArgs, GoogleSetupArgs, GoogleSubmitArgs, GoogleStatusArgs, SubmitArgs, ApiTarget, OutputFormat};
use crate::config::loader::{load_config, save_global_config, save_project_config};
use crate::config::settings::{Settings, GoogleConfig, GoogleAuthConfig, GoogleAuthMethod, QuotaConfig};
use crate::types::error::IndexerError;
use crate::types::Result;
use colored::Colorize;
use dialoguer::{Input, Confirm};
use std::path::PathBuf;

pub async fn run(args: GoogleArgs, cli: &Cli) -> Result<()> {
    match args.command {
        GoogleCommand::Auth(auth_args) => auth(auth_args, cli).await,
        GoogleCommand::Logout => logout(cli).await,
        GoogleCommand::Setup(setup_args) => setup(setup_args, cli).await,
        GoogleCommand::Submit(submit_args) => submit(submit_args, cli).await,
        GoogleCommand::Status(status_args) => status(status_args, cli).await,
        GoogleCommand::Quota => quota(cli).await,
        GoogleCommand::Verify => verify(cli).await,
    }
}

/// Authenticate with Google using OAuth 2.0
pub async fn auth(args: GoogleAuthArgs, _cli: &Cli) -> Result<()> {
    println!("{}", "Google OAuth 2.0 Authentication".cyan().bold());
    println!("{}", "=".repeat(60).dimmed());
    println!();

    // Determine client credentials in order of precedence:
    // 1. Command line arguments (--client-id, --client-secret)
    // 2. Configuration file (indexer.yaml)
    // 3. Environment variables (GOOGLE_OAUTH_CLIENT_ID, GOOGLE_OAUTH_CLIENT_SECRET)
    // 4. Default placeholders (will trigger error)

    let (client_id, client_secret) = if let (Some(id), Some(secret)) =
        (args.client_id.clone(), args.client_secret.clone())
    {
        // Option 1: Use command line arguments
        println!("{}", "Using OAuth credentials from command line arguments".dimmed());
        (Some(id), Some(secret))
    } else if let Ok(config) = load_config() {
        // Option 2: Try to load from configuration file
        if let Some(google_config) = &config.google {
            if let (Some(id), Some(secret)) =
                (&google_config.auth.oauth_client_id, &google_config.auth.oauth_client_secret)
            {
                println!("{}", "Using OAuth credentials from configuration file".dimmed());
                (Some(id.clone()), Some(secret.clone()))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    // Create OAuth flow
    let oauth_flow = if let (Some(id), Some(secret)) = (client_id, client_secret) {
        println!("{}", "Using custom OAuth client credentials".green());
        println!();
        GoogleOAuthFlow::with_credentials(id, secret)?
    } else {
        // Option 3 & 4: Will try environment variables, then fall back to placeholders
        println!("{}", "Checking for OAuth credentials in environment variables...".dimmed());
        let has_env_vars = std::env::var("GOOGLE_OAUTH_CLIENT_ID").is_ok() &&
                           std::env::var("GOOGLE_OAUTH_CLIENT_SECRET").is_ok();
        if has_env_vars {
            println!("{}", "Using OAuth credentials from environment variables".green());
        } else {
            println!("{}", "No custom credentials found, using defaults".yellow());
            println!("{}", "WARNING: Default credentials are placeholders and will not work!".yellow().bold());
        }
        println!();
        GoogleOAuthFlow::new()?
    };

    // Check if already authenticated
    if oauth_flow.is_authenticated() && !args.force {
        println!("{}", "Already authenticated!".green());
        println!("Token location: {}", oauth_flow.token_store().token_file_path().display());
        println!();
        println!("Use --force to re-authenticate");
        return Ok(());
    }

    if args.force && oauth_flow.is_authenticated() {
        println!("{}", "Force re-authentication requested".yellow());
        println!();
    }

    // Start OAuth flow
    oauth_flow.authorize().await?;

    println!();
    println!("Next steps:");
    println!("  • Run 'indexer-cli google verify' to verify the setup");
    println!("  • Run 'indexer-cli google submit <url>' to submit URLs");
    println!("  • Run 'indexer-cli google quota' to check your quota");

    Ok(())
}

/// Logout and revoke Google OAuth credentials
pub async fn logout(_cli: &Cli) -> Result<()> {
    println!("{}", "Google OAuth Logout".cyan().bold());
    println!("{}", "=".repeat(60).dimmed());
    println!();

    let oauth_flow = GoogleOAuthFlow::new()?;

    if !oauth_flow.is_authenticated() {
        println!("{}", "Not currently authenticated".yellow());
        return Ok(());
    }

    // Confirm logout
    let confirm = Confirm::new()
        .with_prompt("Are you sure you want to logout and revoke credentials?")
        .default(false)
        .interact()
        .map_err(|e| IndexerError::InternalError {
            message: format!("Failed to read input: {}", e),
        })?;

    if !confirm {
        println!("Logout cancelled");
        return Ok(());
    }

    // Logout
    oauth_flow.logout().await?;

    println!();
    println!("{}", "Successfully logged out".green());
    println!("Run 'indexer-cli google auth' to authenticate again");

    Ok(())
}

/// Interactive setup for Google service account (simplified wizard, IndexGuru-style)
pub async fn setup(args: GoogleSetupArgs, _cli: &Cli) -> Result<()> {
    println!("{}", "Google Indexing API Setup Wizard".cyan().bold());
    println!("{}", "=".repeat(60));
    println!();

    println!("This wizard will help you set up Google Indexing API (Service Account method)");
    println!("Estimated time: 5-10 minutes (one-time setup)");
    println!();

    // Step 1: Create Google Cloud Project
    println!("{}", "─── Step 1/5: Create Google Cloud Project ───".bold());
    println!();
    println!("1. Visit Google Cloud Console:");
    println!("   {}", "https://console.cloud.google.com/projectcreate".blue().underline());
    println!();
    println!("2. Create a new project:");
    println!("   • Project name: any name (e.g., 'my-indexing-api')");
    println!("   • Click 'Create'");
    println!();

    if !args.non_interactive {
        prompt_continue("Project creation complete? Press Enter to continue")?;
    } else {
        println!("(Skipping confirmation in non-interactive mode)");
        println!();
    }

    // Step 2: Enable API
    println!("{}", "─── Step 2/5: Enable Google Indexing API ───".bold());
    println!();
    println!("1. Visit Indexing API page:");
    println!("   {}", "https://console.cloud.google.com/apis/library/indexing.googleapis.com".blue().underline());
    println!();
    println!("2. Click the 'Enable' button");
    println!();
    println!("   {} Important: Wait 3-5 minutes for API to fully activate!", "⏰".yellow());
    println!();

    if !args.non_interactive {
        prompt_continue("API enabled? Press Enter to continue")?;
    } else {
        println!("(Skipping confirmation in non-interactive mode)");
        println!();
    }

    // Step 3: Create Service Account
    println!("{}", "─── Step 3/5: Create Service Account ───".bold());
    println!();
    println!("1. Visit Service Accounts page:");
    println!("   {}", "https://console.cloud.google.com/iam-admin/serviceaccounts".blue().underline());
    println!();
    println!("2. Make sure you selected the correct project");
    println!();
    println!("3. Click 'Create Service Account'");
    println!();
    println!("4. Fill in the details:");
    println!("   • Service account name: indexer-cli (or any name)");
    println!("   • Description: optional");
    println!("   • Click 'Create and continue'");
    println!();
    println!("5. Skip optional steps, click 'Done'");
    println!();
    println!("   {} IMPORTANT: Note the email address!", "📝".yellow());
    println!("   Format: something@project-id.iam.gserviceaccount.com");
    println!();

    if !args.non_interactive {
        prompt_continue("Service Account created? Press Enter to continue")?;
    } else {
        println!("(Skipping confirmation in non-interactive mode)");
        println!();
    }

    // Step 4: Download JSON Key
    println!("{}", "─── Step 4/5: Download JSON Key ───".bold());
    println!();
    println!("1. In Service Accounts list, find your newly created account");
    println!();
    println!("2. Click the account name → Select the 'Keys' tab");
    println!();
    println!("3. Click 'Add Key' → 'Create new key'");
    println!();
    println!("4. Select {} as key type", "JSON".green().bold());
    println!();
    println!("5. Click 'Create' - JSON file will auto-download");
    println!();

    if !args.non_interactive {
        prompt_continue("JSON key downloaded? Press Enter to continue")?;
    } else {
        println!("(Skipping confirmation in non-interactive mode)");
        println!();
    }

    // Step 5: Add to Search Console (CRITICAL!)
    println!("{}", "─── Step 5/5: Add Service Account to Google Search Console ───".bold());
    println!();
    println!("{}", "⚠️  CRITICAL STEP! Must be completed for API to work!".red().bold());
    println!();
    println!("1. Visit Google Search Console:");
    println!("   {}", "https://search.google.com/search-console".blue().underline());
    println!();
    println!("2. Select your website property");
    println!();
    println!("3. Left menu: Settings → Users and permissions");
    println!();
    println!("4. Click 'Add user'");
    println!();
    println!("5. Paste the Service Account email");
    println!("   (something@project-id.iam.gserviceaccount.com)");
    println!();
    println!("6. {} Set permission level to: {}", "⚠️ ".red(), "Owner".green().bold());
    println!("   Other levels will cause API errors!");
    println!();
    println!("7. Click 'Add'");
    println!();

    if !args.non_interactive {
        prompt_continue("Service Account added to Search Console? Press Enter to continue")?;
    } else {
        println!("(Skipping confirmation in non-interactive mode)");
        println!();
    }

    // Configuration step
    println!();
    println!("{}", "─── Configure indexer-cli ───".bold());
    println!();

    // Get JSON file path
    let json_path = if let Some(path_buf) = &args.service_account {
        // Argument provided
        path_buf.clone()
    } else {
        // Interactive input
        let input: String = Input::new()
            .with_prompt("Enter full path to your JSON key file (e.g., /Users/name/Downloads/my-project-abc123.json)")
            .interact_text()
            .map_err(|e| IndexerError::InternalError {
                message: format!("Failed to read input: {}", e),
            })?;
        PathBuf::from(input.trim())
    };

    // Validate file exists
    if !json_path.exists() {
        println!();
        println!("{}", "✗ File not found!".red());
        println!("  Path: {}", json_path.display());
        println!();
        println!("Please check the file path and try again");
        return Err(IndexerError::GoogleServiceAccountNotFound {
            path: json_path,
        });
    }

    println!();
    println!("{}", "Validating JSON file...".dimmed());

    // Read and validate JSON
    let key = yup_oauth2::read_service_account_key(&json_path)
        .await
        .map_err(|e| {
            let error_msg = e.to_string();
            println!();
            println!("{}", "✗ Failed to read JSON file".red());
            println!("  Error: {}", error_msg.red());
            println!();

            if error_msg.contains("Not enough private keys in PEM")
                || error_msg.contains("private_key")
                || error_msg.contains("key") {
                println!("{}", "This error usually means:".yellow());
                println!("  1. The JSON file is corrupted or incomplete");
                println!("  2. The 'private_key' field is missing or malformed");
                println!("  3. You downloaded P12 format instead of JSON");
                println!();
                println!("{}", "Solution:".green());
                println!("  1. Visit: https://console.cloud.google.com/iam-admin/serviceaccounts");
                println!("  2. Select your service account");
                println!("  3. Keys → Add Key → Create new key");
                println!("  4. {} Choose JSON format (NOT P12!)", "IMPORTANT:".red().bold());
                println!("  5. Click Create");
                println!();
            } else {
                println!("{}", "Troubleshooting:".yellow());
                println!("  • Ensure the file is valid JSON");
                println!("  • Check that the file is not corrupted");
                println!("  • Verify you have the correct file");
            }

            IndexerError::GoogleServiceAccountInvalid {
                message: error_msg,
            }
        })?;

    println!("{}", "✓ JSON file is valid".green());
    println!();
    println!("Service Account information:");
    println!("  • Email: {}", key.client_email.cyan());
    if let Some(project_id) = &key.project_id {
        println!("  • Project ID: {}", project_id.cyan());
    }
    println!();

    // Test API connection
    println!("{}", "Testing API connection...".dimmed());

    match GoogleIndexingClient::from_service_account(json_path.clone()).await {
        Ok(_) => {
            println!("{}", "✓ API connection successful!".green());
        }
        Err(e) => {
            println!("{}", "✗ API connection failed".red());
            println!("  Error: {}", e);
            println!();
            println!("Possible causes:");
            println!("  1. Indexing API is still activating (wait 3-5 minutes)");
            println!("  2. Service Account not added to Search Console");
            println!("  3. Search Console permission is not 'Owner'");
            println!();
            println!("Suggestion: Wait a few minutes, then run 'indexer-cli google verify' to recheck");
            println!();

            // Ask to continue anyway
            let should_continue = if args.non_interactive {
                true // Always continue in non-interactive mode
            } else {
                Confirm::new()
                    .with_prompt("Continue saving configuration anyway? (can verify later)")
                    .default(true)
                    .interact()
                    .map_err(|e| IndexerError::InternalError {
                        message: format!("Failed to read input: {}", e),
                    })?
            };

            if !should_continue {
                return Ok(());
            }
        }
    }

    // Save configuration
    println!();
    println!("{}", "Saving configuration...".dimmed());

    let use_global = if args.global {
        true
    } else if args.non_interactive {
        false // Default to project config in non-interactive mode
    } else {
        Confirm::new()
            .with_prompt("Save to global configuration? (Yes: ~/.indexer-cli/config.yaml, No: ./indexer.yaml)")
            .default(false)
            .interact()
            .map_err(|e| IndexerError::InternalError {
                message: format!("Failed to read input: {}", e),
            })?
    };

    let mut config = load_config().unwrap_or_default();
    config.google = Some(GoogleConfig {
        enabled: true,
        auth: GoogleAuthConfig {
            method: GoogleAuthMethod::ServiceAccount,
            oauth_client_id: None,
            oauth_client_secret: None,
            service_account_file: Some(json_path),
        },
        service_account_file: None,
        quota: QuotaConfig::default(),
        batch_size: 100,
    });

    let config_path = if use_global {
        save_global_config(&config)?
    } else {
        save_project_config(&config)?
    };

    println!("{}", "✓ Configuration saved!".green());
    println!("  Location: {}", config_path.display().to_string().cyan());
    println!();

    // Success summary
    println!();
    println!("{}", "🎉 Setup complete!".green().bold());
    println!("{}", "=".repeat(60));
    println!();
    println!("Google Indexing API is now configured and ready to use!");
    println!();
    println!("Next steps:");
    println!();
    println!("  1. {} Verify configuration", "indexer-cli google verify".cyan());
    println!("     Check that everything is set up correctly");
    println!();
    println!("  2. {} Check your quota", "indexer-cli google quota".cyan());
    println!("     See remaining daily submissions");
    println!();
    println!("  3. {} Start submitting URLs", "indexer-cli submit --sitemap https://your-site.com/sitemap.xml".cyan());
    println!("     Begin submitting URLs to Google");
    println!();

    Ok(())
}

/// Helper function: Wait for user to press Enter
fn prompt_continue(message: &str) -> Result<()> {
    use std::io::{self, Write};

    println!();
    print!("{} ", message.dimmed());
    io::stdout().flush().map_err(|e| IndexerError::InternalError {
        message: format!("Output failed: {}", e),
    })?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| IndexerError::InternalError {
        message: format!("Failed to read input: {}", e),
    })?;

    println!();
    Ok(())
}

/// Submit URLs to Google (wrapper around main submit)
pub async fn submit(args: GoogleSubmitArgs, cli: &Cli) -> Result<()> {
    // Convert GoogleAction to the main GoogleAction type
    let google_action = match args.action {
        GoogleAction::UrlUpdated => crate::cli::args::GoogleAction::UrlUpdated,
        GoogleAction::UrlDeleted => crate::cli::args::GoogleAction::UrlDeleted,
    };

    // Convert to main SubmitArgs
    let submit_args = SubmitArgs {
        urls: args.urls,
        file: args.file,
        sitemap: args.sitemap,
        api: ApiTarget::Google, // Google only
        filter: args.filter,
        since: args.since,
        google_action,
        batch_size: args.batch_size,
        dry_run: args.dry_run,
        skip_history: args.skip_history,
        format: OutputFormat::Text,
    };

    // Delegate to main submit command
    crate::commands::submit::run(submit_args, cli).await
}

/// Check indexing status of URLs
pub async fn status(args: GoogleStatusArgs, _cli: &Cli) -> Result<()> {
    println!("{}", "Google Indexing Status".cyan().bold());
    println!("{}", "=".repeat(60).dimmed());
    println!();

    // Load config
    let config = load_config().map_err(|_| IndexerError::ConfigMissingField {
        field: "configuration".to_string(),
    })?;

    // Create client
    let client = create_google_client(&config).await?;

    // Collect URLs
    let urls = collect_urls(&args.urls, &args.file)?;

    if urls.is_empty() {
        println!("{}", "No URLs to check".yellow());
        return Ok(());
    }

    println!("Checking status for {} URL(s)...", urls.len());
    println!();

    // Query status for each URL
    let mut results = Vec::new();
    for url in &urls {
        match client.get_metadata(url).await {
            Ok(metadata) => results.push((url, Some(metadata))),
            Err(_) => results.push((url, None)),
        }
    }

    // Display results
    match args.format {
        OutputFormat::Text => display_status_table(&results),
        OutputFormat::Json => display_status_json(&results)?,
        OutputFormat::Csv => display_status_csv(&results)?,
    }

    Ok(())
}

/// Show quota usage
pub async fn quota(_cli: &Cli) -> Result<()> {
    println!("{}", "Google Indexing API Quota".cyan().bold());
    println!("{}", "=".repeat(60).dimmed());
    println!();

    // Load config
    let config = load_config().map_err(|_| IndexerError::ConfigMissingField {
        field: "configuration".to_string(),
    })?;

    let google_config = require_google_config(&config)?;

    // Create client
    let client = create_google_client(&config).await?;

    // Get quota info
    let quota_info = client.check_quota().await?;

    // Calculate remaining
    let remaining = quota_info.daily_publish_limit.saturating_sub(quota_info.daily_publish_used);

    println!("{:<20} {}", "Daily Limit:", google_config.quota.daily_limit.to_string().cyan());
    println!("{:<20} {}", "Used Today:", quota_info.daily_publish_used.to_string().yellow());
    println!("{:<20} {}", "Remaining:", remaining.to_string().green());
    println!("{:<20} {} req/min", "Rate Limit:", google_config.quota.rate_limit.to_string().cyan());
    println!();

    if remaining == 0 {
        println!("{}", "⚠ Quota exhausted! Quota resets at midnight Pacific Time.".yellow().bold());
    } else {
        let percent_used = (quota_info.daily_publish_used as f64 / quota_info.daily_publish_limit as f64) * 100.0;
        if percent_used >= 90.0 {
            println!("{}", format!("⚠ Warning: {:.1}% of daily quota used", percent_used).yellow());
        } else {
            println!("{}", format!("✓ {:.1}% of daily quota used", percent_used).green());
        }
    }

    Ok(())
}

/// Verify configuration and connectivity
pub async fn verify(_cli: &Cli) -> Result<()> {
    println!("{}", "Verifying Google Indexing API Configuration".cyan().bold());
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
            println!("  {}", "Run 'indexer-cli google auth' or 'indexer-cli google setup' to configure".dimmed());
            return Err(IndexerError::ConfigMissingField {
                field: "configuration".to_string(),
            });
        }
    };

    // Check Google config
    let google_config = match config.google {
        Some(ref g) if g.enabled => {
            println!("{}", "✓ Google configuration enabled".green());
            g
        }
        _ => {
            println!("{}", "✗ Google not configured or disabled".red());
            println!("  {}", "Run 'indexer-cli google auth' or 'indexer-cli google setup' to configure".dimmed());
            return Err(IndexerError::ConfigMissingField {
                field: "google".to_string(),
            });
        }
    };

    // Check authentication method
    match google_config.auth.method {
        GoogleAuthMethod::OAuth => {
            println!("  Authentication Method: {}", "OAuth 2.0".cyan());

            // Check if OAuth token exists
            let oauth_flow = GoogleOAuthFlow::new()?;
            if oauth_flow.is_authenticated() {
                println!("{}", "✓ OAuth credentials found".green());
            } else {
                println!("{}", "✗ OAuth credentials not found".red());
                println!("  {}", "Run 'indexer-cli google auth' to authenticate".dimmed());
                return Err(IndexerError::GoogleAuthError {
                    message: "Not authenticated".to_string(),
                });
            }
        }
        GoogleAuthMethod::ServiceAccount => {
            println!("  Authentication Method: {}", "Service Account".cyan());

            // Check service account file
            let service_account_file = google_config
                .auth
                .service_account_file
                .as_ref()
                .or(google_config.service_account_file.as_ref())
                .ok_or_else(|| IndexerError::ConfigMissingField {
                    field: "google.auth.service_account_file".to_string(),
                })?;

            if !service_account_file.exists() {
                println!("{}", "✗ Service account file not found".red());
                println!("  Expected: {}", service_account_file.display().to_string().dimmed());
                return Err(IndexerError::GoogleServiceAccountNotFound {
                    path: service_account_file.clone(),
                });
            }
            println!("{}", "✓ Service account file exists".green());

            // Validate JSON file format before authentication test
            print!("{}", "  Validating JSON format... ".dimmed());
            match yup_oauth2::read_service_account_key(&service_account_file).await {
                Ok(key) => {
                    println!("{}", "✓".green());
                    println!("    Service Account: {}", key.client_email.cyan());

                    // Check for RSA PRIVATE KEY format issue
                    if let Ok(content) = std::fs::read_to_string(&service_account_file) {
                        if content.contains("BEGIN RSA PRIVATE KEY") {
                            println!();
                            println!("{}", "✗ Invalid private key format detected!".red().bold());
                            println!();
                            println!("The JSON contains {} instead of {}",
                                "RSA PRIVATE KEY (PKCS#1)".red(),
                                "PRIVATE KEY (PKCS#8)".green());
                            println!();
                            println!("{}", "Why this happens:".yellow());
                            println!("  • Old key creation method was used");
                            println!("  • Key was manually edited or converted");
                            println!("  • Downloaded from wrong source");
                            println!();
                            println!("{}", "How to fix:".green().bold());
                            println!("  1. Go to Google Cloud Console:");
                            println!("     {}", "https://console.cloud.google.com/iam-admin/serviceaccounts".blue());
                            println!();
                            println!("  2. Delete the current key:");
                            println!("     • Click on service account: {}", key.client_email.cyan());
                            println!("     • Keys tab → Find key → Delete");
                            println!();
                            println!("  3. Create a NEW key:");
                            println!("     • Add Key → Create new key");
                            println!("     • Select {} format", "JSON".green().bold());
                            println!("     • Download the file");
                            println!();
                            println!("  4. Re-run setup:");
                            println!("     {}", "indexer-cli google setup --service-account <new-file.json>".cyan());
                            println!();
                            return Err(IndexerError::GoogleServiceAccountInvalid {
                                message: format!("Private key is in PKCS#1 format (RSA PRIVATE KEY). Google requires PKCS#8 format (PRIVATE KEY). File: {}", service_account_file.display()),
                            });
                        }
                    }
                }
                Err(e) => {
                    println!("{}", "✗".red());
                    println!();
                    println!("{}", format!("JSON validation error: {}", e).red());
                    println!();

                    if e.to_string().contains("Not enough private keys in PEM")
                        || e.to_string().contains("private_key")
                        || e.to_string().contains("key") {
                        println!("{}", "Common causes:".yellow());
                        println!("  • Private key field is missing or corrupted");
                        println!("  • Downloaded P12 format instead of JSON");
                        println!("  • JSON file is truncated or damaged");
                        println!();
                        println!("{}", "Fix:".green());
                        println!("  1. Delete the current key from Google Cloud Console");
                        println!("  2. Create a new key and download as JSON format");
                        println!("  3. Run: indexer-cli google setup --service-account <path>");
                    }

                    return Err(IndexerError::GoogleServiceAccountInvalid {
                        message: e.to_string(),
                    });
                }
            }
        }
    }

    // Test authentication
    print!("{}", "  Testing authentication... ".dimmed());
    let client = create_google_client(&config).await?;
    println!("{}", "✓".green());

    // Test API connectivity (try to get metadata for a test URL)
    print!("{}", "  Testing API connectivity... ".dimmed());
    match client.get_metadata("https://example.com").await {
        Ok(_) => {
            println!("{}", "✓".green());
        }
        Err(IndexerError::GoogleApiError { status_code: 404, .. }) => {
            // 404 is expected for non-existent URLs, but proves API connectivity
            println!("{}", "✓".green());
        }
        Err(IndexerError::GooglePermissionDenied { .. }) => {
            // Permission denied means auth works but URL not owned
            println!("{}", "✓".green());
            println!();
            println!("{}", "⚠ Note: Permission denied for example.com (expected)".yellow());
            match google_config.auth.method {
                GoogleAuthMethod::OAuth => {
                    println!("  {}", "Make sure you have access to the property in Search Console".dimmed());
                }
                GoogleAuthMethod::ServiceAccount => {
                    println!("  {}", "Make sure to add the service account as an owner in Search Console".dimmed());
                }
            }
        }
        Err(e) => {
            println!("{}", "✗".red());
            println!();
            println!("{}", format!("Error: {}", e).red());
            return Err(e);
        }
    }

    println!();
    println!("{}", "✓ All checks passed!".green().bold());
    println!("  {}", "Google Indexing API is ready to use".green());
    println!();
    println!("Configuration details:");
    println!("  Authentication: {}", format!("{:?}", google_config.auth.method).dimmed());
    println!("  Daily Limit: {}", google_config.quota.daily_limit.to_string().dimmed());
    println!("  Rate Limit: {} req/min", google_config.quota.rate_limit.to_string().dimmed());
    println!("  Batch Size: {}", google_config.batch_size.to_string().dimmed());

    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Create Google Indexing client based on configuration
async fn create_google_client(config: &Settings) -> Result<GoogleIndexingClient> {
    let google_config = require_google_config(config)?;

    // Check authentication method
    match google_config.auth.method {
        GoogleAuthMethod::OAuth => {
            // Use OAuth authentication
            GoogleIndexingClient::from_oauth().await
        }
        GoogleAuthMethod::ServiceAccount => {
            // Use service account authentication
            let service_account_file = google_config
                .auth
                .service_account_file
                .as_ref()
                .or(google_config.service_account_file.as_ref()) // Fallback to legacy field
                .ok_or_else(|| IndexerError::ConfigMissingField {
                    field: "google.auth.service_account_file".to_string(),
                })?;

            GoogleIndexingClient::from_service_account(service_account_file.clone()).await
        }
    }
}

/// Require Google config or return error
fn require_google_config(config: &Settings) -> Result<&GoogleConfig> {
    config
        .google
        .as_ref()
        .filter(|g| g.enabled)
        .ok_or_else(|| IndexerError::ConfigMissingField {
            field: "google".to_string(),
        })
}

/// Collect URLs from command line args and file
fn collect_urls(urls: &[String], file: &Option<PathBuf>) -> Result<Vec<String>> {
    let mut all_urls = urls.to_vec();

    if let Some(path) = file {
        if !path.exists() {
            return Err(IndexerError::FileNotFound {
                path: path.clone(),
            });
        }

        let file_content = std::fs::read_to_string(path).map_err(|e| IndexerError::FileReadError {
            path: path.clone(),
            message: e.to_string(),
        })?;

        for line in file_content.lines() {
            let url = line.trim();
            // Skip empty lines and comments
            if !url.is_empty() && !url.starts_with('#') {
                all_urls.push(url.to_string());
            }
        }
    }

    Ok(all_urls)
}

/// Display status results in table format
fn display_status_table(results: &[(&String, Option<MetadataResponse>)]) {
    println!("{:<50} {:<15} {:<20}", "URL".bold(), "Status".bold(), "Last Updated".bold());
    println!("{}", "―".repeat(85).dimmed());

    for (url, metadata) in results {
        match metadata {
            Some(m) => {
                let status = "Indexed".green();
                let last_update = m
                    .url_notification_metadata
                    .latest_update
                    .as_ref()
                    .and_then(|u| u.notify_time.clone())
                    .unwrap_or_else(|| "Unknown".to_string());
                let last_update_display = if last_update == "Unknown" {
                    last_update.dimmed().to_string()
                } else {
                    last_update.cyan().to_string()
                };
                println!(
                    "{:<50} {:<15} {:<20}",
                    truncate(url, 50),
                    status,
                    last_update_display
                );
            }
            None => {
                println!(
                    "{:<50} {:<15} {:<20}",
                    truncate(url, 50),
                    "Not found".yellow(),
                    "-".dimmed()
                );
            }
        }
    }
}

/// Display status results in JSON format
fn display_status_json(results: &[(&String, Option<MetadataResponse>)]) -> Result<()> {
    let json_results: Vec<serde_json::Value> = results
        .iter()
        .map(|(url, metadata)| {
            match metadata {
                Some(m) => serde_json::json!({
                    "url": url,
                    "status": "indexed",
                    "last_updated": m.url_notification_metadata.latest_update.as_ref()
                        .and_then(|u| u.notify_time.clone())
                        .unwrap_or_else(|| "unknown".to_string()),
                    "notification_type": m.url_notification_metadata.latest_update.as_ref()
                        .map(|u| u.notification_type.clone())
                        .unwrap_or_else(|| "unknown".to_string()),
                }),
                None => serde_json::json!({
                    "url": url,
                    "status": "not_found",
                    "last_updated": null,
                    "notification_type": null,
                }),
            }
        })
        .collect();

    let json_output = serde_json::json!({
        "results": json_results,
        "total": results.len(),
        "indexed": results.iter().filter(|(_, m)| m.is_some()).count(),
        "not_found": results.iter().filter(|(_, m)| m.is_none()).count(),
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&json_output).map_err(|e| {
            IndexerError::JsonSerializationError {
                message: e.to_string(),
            }
        })?
    );

    Ok(())
}

/// Display status results in CSV format
fn display_status_csv(results: &[(&String, Option<MetadataResponse>)]) -> Result<()> {
    use std::io;

    let mut wtr = csv::Writer::from_writer(io::stdout());

    // Write header
    wtr.write_record(&["url", "status", "last_updated", "notification_type"])
        .map_err(|e| IndexerError::InternalError {
            message: format!("CSV write error: {}", e),
        })?;

    // Write data
    for (url, metadata) in results {
        match metadata {
            Some(m) => {
                let last_update = m
                    .url_notification_metadata
                    .latest_update
                    .as_ref()
                    .and_then(|u| u.notify_time.clone())
                    .unwrap_or_else(|| "unknown".to_string());
                let notification_type = m
                    .url_notification_metadata
                    .latest_update
                    .as_ref()
                    .map(|u| u.notification_type.clone())
                    .unwrap_or_else(|| "unknown".to_string());

                wtr.write_record(&[url, "indexed", &last_update, &notification_type])
                    .map_err(|e| IndexerError::InternalError {
                        message: format!("CSV write error: {}", e),
                    })?;
            }
            None => {
                wtr.write_record(&[url, "not_found", "", ""])
                    .map_err(|e| IndexerError::InternalError {
                        message: format!("CSV write error: {}", e),
                    })?;
            }
        }
    }

    wtr.flush().map_err(|e| IndexerError::InternalError {
        message: format!("CSV flush error: {}", e),
    })?;

    Ok(())
}

/// Truncate string to specified length with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a very long string", 10), "this is...");
    }

    #[test]
    fn test_collect_urls() {
        let urls = vec!["https://example.com".to_string()];
        let result = collect_urls(&urls, &None).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "https://example.com");
    }
}
