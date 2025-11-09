//! Authentication module for various API providers.
//!
//! This module provides OAuth 2.0 authentication and token management
//! for Google Indexing API and other services.

pub mod oauth;
pub mod token_store;

pub use oauth::{GoogleOAuthFlow, OAuthToken};
pub use token_store::TokenStore;
