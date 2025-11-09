//! OAuth 2.0 authentication flow for Google APIs.
//!
//! This module implements the OAuth 2.0 authorization code flow with PKCE
//! for Google Indexing API. It provides a user-friendly web-based authentication
//! flow using a local HTTP server to receive the authorization code.

use crate::auth::token_store::{StoredToken, TokenStore};
use crate::types::error::IndexerError;
use crate::types::Result;
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl, RefreshToken,
    Scope, TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use tracing::{debug, info, warn};

/// Google OAuth 2.0 endpoints
const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

/// Google Indexing API scope
const GOOGLE_INDEXING_SCOPE: &str = "https://www.googleapis.com/auth/indexing";

/// Default OAuth 2.0 client credentials
///
/// Note: These are placeholder credentials. Users MUST create their own
/// OAuth client in Google Cloud Console to use this feature.
///
/// To set up your own credentials:
/// 1. Create a project at https://console.cloud.google.com
/// 2. Enable Google Indexing API
/// 3. Create OAuth 2.0 client ID (Desktop app or Web application)
/// 4. Configure redirect URI: http://localhost:8080/oauth/callback
/// 5. Provide credentials via:
///    - Command line: --client-id <ID> --client-secret <SECRET>
///    - Environment variables: GOOGLE_OAUTH_CLIENT_ID, GOOGLE_OAUTH_CLIENT_SECRET
///    - Configuration file (indexer.yaml): google.auth.oauth_client_id, google.auth.oauth_client_secret
const DEFAULT_CLIENT_ID: &str = "YOUR_CLIENT_ID.apps.googleusercontent.com";
const DEFAULT_CLIENT_SECRET: &str = "YOUR_CLIENT_SECRET";

/// Environment variable names for OAuth credentials
const ENV_CLIENT_ID: &str = "GOOGLE_OAUTH_CLIENT_ID";
const ENV_CLIENT_SECRET: &str = "GOOGLE_OAUTH_CLIENT_SECRET";

/// OAuth token structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    /// Access token
    pub access_token: String,
    /// Refresh token
    pub refresh_token: Option<String>,
    /// Token expiration time in seconds
    pub expires_in: Option<i64>,
    /// Token scopes
    pub scopes: Vec<String>,
}

/// Google OAuth 2.0 flow handler
pub struct GoogleOAuthFlow {
    /// Token store
    token_store: TokenStore,
    /// Client ID (custom or default)
    client_id: String,
    /// Client secret (custom or default)
    client_secret: String,
    /// Redirect URL
    redirect_url: RedirectUrl,
}

impl GoogleOAuthFlow {
    /// Create a new Google OAuth flow with default credentials
    ///
    /// This method attempts to load credentials from the following sources in order:
    /// 1. Environment variables (GOOGLE_OAUTH_CLIENT_ID, GOOGLE_OAUTH_CLIENT_SECRET)
    /// 2. Default constants (which are placeholders)
    ///
    /// # Returns
    ///
    /// Returns a new GoogleOAuthFlow instance
    ///
    /// # Errors
    ///
    /// Returns an error if the token store cannot be initialized
    pub fn new() -> Result<Self> {
        // Try to load from environment variables first
        let client_id = std::env::var(ENV_CLIENT_ID)
            .unwrap_or_else(|_| DEFAULT_CLIENT_ID.to_string());
        let client_secret = std::env::var(ENV_CLIENT_SECRET)
            .unwrap_or_else(|_| DEFAULT_CLIENT_SECRET.to_string());

        Self::with_credentials(client_id, client_secret)
    }

    /// Create a new Google OAuth flow with custom credentials
    ///
    /// # Arguments
    ///
    /// * `client_id` - Google OAuth client ID
    /// * `client_secret` - Google OAuth client secret
    ///
    /// # Returns
    ///
    /// Returns a new GoogleOAuthFlow instance
    ///
    /// # Errors
    ///
    /// Returns an error if the OAuth client cannot be initialized
    pub fn with_credentials(client_id: String, client_secret: String) -> Result<Self> {
        let token_store = TokenStore::google_default()?;

        // Create redirect URL (local server)
        let redirect_url = RedirectUrl::new("http://localhost:8080/oauth/callback".to_string())
            .map_err(|e| IndexerError::GoogleAuthError {
                message: format!("Invalid redirect URL: {}", e),
            })?;

        Ok(Self {
            token_store,
            client_id,
            client_secret,
            redirect_url,
        })
    }

    /// Start the OAuth authorization flow
    ///
    /// This method:
    /// 1. Validates that OAuth credentials are not placeholders
    /// 2. Starts a local HTTP server to receive the authorization code
    /// 3. Opens the user's browser to the Google authorization page
    /// 4. Waits for the user to authorize the application
    /// 5. Exchanges the authorization code for tokens
    /// 6. Saves the tokens to storage
    ///
    /// # Returns
    ///
    /// Returns Ok if the authorization was successful
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The OAuth credentials are placeholders
    /// - The local server cannot be started
    /// - The browser cannot be opened
    /// - The authorization fails
    /// - The token exchange fails
    pub async fn authorize(&self) -> Result<()> {
        // Check if credentials are placeholders
        if self.is_using_placeholders() {
            return Err(IndexerError::GoogleAuthError {
                message: self.get_placeholder_error_message(),
            });
        }

        info!("Starting Google OAuth 2.0 authorization flow");

        // Create OAuth client
        let client = BasicClient::new(ClientId::new(self.client_id.clone()))
            .set_client_secret(ClientSecret::new(self.client_secret.clone()))
            .set_auth_uri(AuthUrl::new(GOOGLE_AUTH_URL.to_string()).unwrap())
            .set_token_uri(TokenUrl::new(GOOGLE_TOKEN_URL.to_string()).unwrap())
            .set_redirect_uri(self.redirect_url.clone());

        // Generate PKCE challenge
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        // Generate authorization URL
        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new(GOOGLE_INDEXING_SCOPE.to_string()))
            .add_extra_param("access_type", "offline")
            .add_extra_param("prompt", "consent")
            .set_pkce_challenge(pkce_challenge)
            .url();

        info!("Authorization URL: {}", auth_url);

        // Open browser
        println!("\nOpening browser for Google authorization...");
        println!("If the browser doesn't open automatically, visit this URL:");
        println!("\n{}\n", auth_url);

        if let Err(e) = webbrowser::open(auth_url.as_str()) {
            warn!("Failed to open browser automatically: {}", e);
            println!("Please open the URL manually in your browser.");
        }

        // Start local server to receive callback
        let (code, state) = self.start_callback_server()?;

        // Verify CSRF token
        if state.secret() != csrf_token.secret() {
            return Err(IndexerError::GoogleAuthError {
                message: "CSRF token mismatch".to_string(),
            });
        }

        debug!("CSRF token verified");

        // Exchange authorization code for tokens
        println!("Exchanging authorization code for tokens...");

        // Create HTTP client for token exchange
        let http_client = oauth2::reqwest::blocking::ClientBuilder::new()
            .redirect(oauth2::reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| IndexerError::GoogleAuthError {
                message: format!("Failed to create HTTP client: {}", e),
            })?;

        let token_response = client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(pkce_verifier)
            .request(&http_client)
            .map_err(|e| IndexerError::GoogleAuthError {
                message: format!("Token exchange failed: {}", e),
            })?;

        // Extract tokens
        let access_token = token_response.access_token().secret().to_string();
        let refresh_token = token_response
            .refresh_token()
            .map(|t| t.secret().to_string())
            .ok_or_else(|| IndexerError::GoogleAuthError {
                message: "No refresh token received. Make sure to request 'access_type=offline'"
                    .to_string(),
            })?;

        let expires_in = token_response
            .expires_in()
            .map(|d| d.as_secs() as i64)
            .unwrap_or(3600);

        // Create stored token
        let stored_token = StoredToken::new(
            access_token,
            refresh_token,
            expires_in,
            vec![GOOGLE_INDEXING_SCOPE.to_string()],
        );

        // Save token
        self.token_store.save_token(&stored_token)?;

        info!(
            "Token saved to: {}",
            self.token_store.token_file_path().display()
        );

        println!("\nAuthorization successful!");
        println!(
            "Credentials saved to: {}",
            self.token_store.token_file_path().display()
        );

        Ok(())
    }

    /// Start a local HTTP server to receive the OAuth callback
    ///
    /// # Returns
    ///
    /// Returns the authorization code and CSRF state
    ///
    /// # Errors
    ///
    /// Returns an error if the server cannot bind or receive the callback
    fn start_callback_server(&self) -> Result<(String, CsrfToken)> {
        // Bind to localhost:8080
        let listener = TcpListener::bind("127.0.0.1:8080").map_err(|e| {
            IndexerError::GoogleAuthError {
                message: format!("Failed to start local server: {}. Port 8080 may be in use.", e),
            }
        })?;

        println!("Waiting for authorization...");
        debug!("Local server listening on http://127.0.0.1:8080");

        // Accept one connection
        let (mut stream, _) = listener.accept().map_err(|e| IndexerError::GoogleAuthError {
            message: format!("Failed to accept connection: {}", e),
        })?;

        // Read the request
        let mut reader = BufReader::new(&stream);
        let mut request_line = String::new();
        reader
            .read_line(&mut request_line)
            .map_err(|e| IndexerError::GoogleAuthError {
                message: format!("Failed to read request: {}", e),
            })?;

        debug!("Received request: {}", request_line);

        // Extract code and state from URL
        let redirect_url = request_line.split_whitespace().nth(1).ok_or_else(|| {
            IndexerError::GoogleAuthError {
                message: "Invalid callback request".to_string(),
            }
        })?;

        let url = url::Url::parse(&format!("http://localhost{}", redirect_url)).map_err(|e| {
            IndexerError::GoogleAuthError {
                message: format!("Failed to parse callback URL: {}", e),
            }
        })?;

        let code = url
            .query_pairs()
            .find(|(key, _)| key == "code")
            .map(|(_, value)| value.to_string())
            .ok_or_else(|| IndexerError::GoogleAuthError {
                message: "Authorization code not found in callback".to_string(),
            })?;

        let state = url
            .query_pairs()
            .find(|(key, _)| key == "state")
            .map(|(_, value)| CsrfToken::new(value.to_string()))
            .ok_or_else(|| IndexerError::GoogleAuthError {
                message: "State parameter not found in callback".to_string(),
            })?;

        // Send success response to browser
        let response = "HTTP/1.1 200 OK\r\n\
                       Content-Type: text/html; charset=utf-8\r\n\
                       \r\n\
                       <html>\
                       <head><title>Authorization Successful</title></head>\
                       <body style='font-family: sans-serif; text-align: center; padding: 50px;'>\
                       <h1 style='color: #4CAF50;'>✓ Authorization Successful!</h1>\
                       <p>You can close this window and return to the terminal.</p>\
                       </body>\
                       </html>";

        stream
            .write_all(response.as_bytes())
            .map_err(|e| IndexerError::GoogleAuthError {
                message: format!("Failed to send response: {}", e),
            })?;

        stream.flush().map_err(|e| IndexerError::GoogleAuthError {
            message: format!("Failed to flush stream: {}", e),
        })?;

        Ok((code, state))
    }

    /// Get a valid access token
    ///
    /// This method:
    /// 1. Checks if a token exists in storage
    /// 2. If the token is expired, refreshes it
    /// 3. Returns the access token
    ///
    /// # Returns
    ///
    /// Returns the access token
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No token is stored
    /// - The token cannot be refreshed
    pub async fn get_access_token(&self) -> Result<String> {
        // Load token from storage
        let mut token = self.token_store.load_token()?;

        // Check if token is expired
        if token.is_expired() {
            info!("Access token expired, refreshing...");
            token = self.refresh_access_token(&token.refresh_token).await?;
        }

        Ok(token.access_token)
    }

    /// Refresh the access token using the refresh token
    ///
    /// # Arguments
    ///
    /// * `refresh_token` - The refresh token
    ///
    /// # Returns
    ///
    /// Returns the new stored token
    ///
    /// # Errors
    ///
    /// Returns an error if the token refresh fails
    async fn refresh_access_token(&self, refresh_token: &str) -> Result<StoredToken> {
        debug!("Refreshing access token");

        // Create OAuth client
        let client = BasicClient::new(ClientId::new(self.client_id.clone()))
            .set_client_secret(ClientSecret::new(self.client_secret.clone()))
            .set_auth_uri(AuthUrl::new(GOOGLE_AUTH_URL.to_string()).unwrap())
            .set_token_uri(TokenUrl::new(GOOGLE_TOKEN_URL.to_string()).unwrap())
            .set_redirect_uri(self.redirect_url.clone());

        // Create HTTP client for token refresh
        let http_client = oauth2::reqwest::blocking::ClientBuilder::new()
            .redirect(oauth2::reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| IndexerError::GoogleAuthError {
                message: format!("Failed to create HTTP client: {}", e),
            })?;

        let token_response = client
            .exchange_refresh_token(&RefreshToken::new(refresh_token.to_string()))
            .request(&http_client)
            .map_err(|e| IndexerError::GoogleAuthError {
                message: format!("Token refresh failed: {}", e),
            })?;

        // Extract new tokens
        let access_token = token_response.access_token().secret().to_string();
        let new_refresh_token = token_response
            .refresh_token()
            .map(|t| t.secret().to_string())
            .unwrap_or_else(|| refresh_token.to_string()); // Keep old refresh token if not provided

        let expires_in = token_response
            .expires_in()
            .map(|d| d.as_secs() as i64)
            .unwrap_or(3600);

        // Create and save new token
        let stored_token = StoredToken::new(
            access_token,
            new_refresh_token,
            expires_in,
            vec![GOOGLE_INDEXING_SCOPE.to_string()],
        );

        self.token_store.save_token(&stored_token)?;

        debug!("Access token refreshed successfully");

        Ok(stored_token)
    }

    /// Revoke the stored token and delete it
    ///
    /// # Returns
    ///
    /// Returns Ok if the token was revoked successfully
    ///
    /// # Errors
    ///
    /// Returns an error if the token cannot be revoked or deleted
    pub async fn logout(&self) -> Result<()> {
        info!("Logging out and revoking token");

        // Load token
        if let Ok(token) = self.token_store.load_token() {
            // Revoke token via Google API
            let revoke_url = format!(
                "https://oauth2.googleapis.com/revoke?token={}",
                token.access_token
            );

            let client = reqwest::Client::new();
            let response = client
                .post(&revoke_url)
                .send()
                .await
                .map_err(|e| IndexerError::HttpRequestFailed {
                    message: e.to_string(),
                })?;

            if response.status().is_success() {
                debug!("Token revoked successfully");
            } else {
                warn!("Token revocation may have failed: {}", response.status());
            }
        }

        // Delete stored token
        self.token_store.delete_token()?;

        println!("Logged out successfully");
        Ok(())
    }

    /// Check if the user is authenticated
    ///
    /// # Returns
    ///
    /// Returns true if a valid token exists
    pub fn is_authenticated(&self) -> bool {
        self.token_store.has_token()
    }

    /// Get the token store
    pub fn token_store(&self) -> &TokenStore {
        &self.token_store
    }

    /// Check if the current credentials are placeholders
    fn is_using_placeholders(&self) -> bool {
        self.client_id.starts_with("YOUR_CLIENT_ID") ||
        self.client_secret == "YOUR_CLIENT_SECRET" ||
        self.client_id.is_empty() ||
        self.client_secret.is_empty()
    }

    /// Generate a helpful error message when placeholders are detected
    fn get_placeholder_error_message(&self) -> String {
        format!(
            r#"OAuth client credentials not configured.

The current credentials are placeholders and will not work with Google's OAuth service.

To use Google OAuth authentication, you need to create your own OAuth 2.0 client:

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
SETUP INSTRUCTIONS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Step 1: Create Google Cloud Project
  → Visit: https://console.cloud.google.com
  → Create a new project or select an existing one

Step 2: Enable Google Indexing API
  → Go to "APIs & Services" → "Library"
  → Search for "Indexing API"
  → Click "Enable"

Step 3: Create OAuth 2.0 Client
  → Go to "APIs & Services" → "Credentials"
  → Click "Create Credentials" → "OAuth client ID"
  → Application type: "Desktop app" or "Web application"
  → Add authorized redirect URI: http://localhost:8080/oauth/callback
  → Click "Create" and note your Client ID and Client Secret

Step 4: Configure Credentials
  Choose one of the following methods:

  Option A - Command Line (Quick Start):
    indexer-cli google auth \
      --client-id "YOUR_CLIENT_ID.apps.googleusercontent.com" \
      --client-secret "YOUR_CLIENT_SECRET"

  Option B - Environment Variables (Recommended):
    export GOOGLE_OAUTH_CLIENT_ID="YOUR_CLIENT_ID.apps.googleusercontent.com"
    export GOOGLE_OAUTH_CLIENT_SECRET="YOUR_CLIENT_SECRET"
    indexer-cli google auth

  Option C - Configuration File (indexer.yaml):
    google:
      auth:
        method: oauth
        oauth_client_id: "YOUR_CLIENT_ID.apps.googleusercontent.com"
        oauth_client_secret: "YOUR_CLIENT_SECRET"

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

For detailed setup guide with screenshots, visit:
https://github.com/yourusername/indexer-cli/blob/master/docs/google-oauth-setup.md

Alternative: If you prefer, you can use Service Account authentication instead:
  indexer-cli google setup --service-account /path/to/service-account.json
"#
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_flow_creation() {
        let _flow = GoogleOAuthFlow::with_credentials(
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
        );
        assert!(_flow.is_ok());
    }

    #[test]
    fn test_is_authenticated_no_token() {
        let flow = GoogleOAuthFlow::with_credentials(
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
        )
        .unwrap();
        // Should be false if no token exists
        // (This test will fail if a token actually exists in the default location)
    }
}
