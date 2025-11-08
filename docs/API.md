# API Integration Guide

This document provides comprehensive information about integrating with the Google Indexing API and IndexNow API through indexer-cli.

## Table of Contents

- [Google Indexing API](#google-indexing-api)
  - [Overview](#overview)
  - [Authentication](#authentication)
  - [API Operations](#api-operations)
  - [Rate Limits and Quotas](#rate-limits-and-quotas)
  - [Error Handling](#error-handling-google)
  - [Best Practices](#best-practices-google)
- [IndexNow API](#indexnow-api)
  - [Overview](#overview-1)
  - [Authentication](#authentication-1)
  - [API Operations](#api-operations-1)
  - [Endpoints](#endpoints)
  - [Error Handling](#error-handling-indexnow)
  - [Best Practices](#best-practices-indexnow)
- [Integration Examples](#integration-examples)
- [Troubleshooting](#troubleshooting)

## Google Indexing API

### Overview

The Google Indexing API allows you to notify Google when pages are added, updated, or removed, enabling faster indexing than waiting for Googlebot to discover changes.

**Official Documentation**: [Google Indexing API](https://developers.google.com/search/apis/indexing-api/v3/quickstart)

**Key Features**:
- Notify Google of URL updates (new or modified pages)
- Notify Google of URL deletions (removed pages)
- Get metadata about submitted URLs
- Check indexing status

**Use Cases**:
- Job postings
- Live streaming events
- News articles
- Time-sensitive content
- Any content that requires rapid indexing

**Limitations**:
- Only works for verified properties in Google Search Console
- Daily quota: 200 URL notifications
- Not suitable for bulk indexing of entire websites

### Authentication

The Google Indexing API uses OAuth2 service account authentication.

#### Setting Up Authentication

1. **Create a GCP Project**
   - Go to [Google Cloud Console](https://console.cloud.google.com/)
   - Create a new project or select existing

2. **Enable the API**
   - Navigate to APIs & Services > Library
   - Search for "Web Search Indexing API"
   - Click Enable

3. **Create Service Account**
   - Go to APIs & Services > Credentials
   - Click Create Credentials > Service Account
   - Name your service account (e.g., "indexer-cli")
   - Click Create and Continue

4. **Generate Key**
   - In the service accounts list, click on your account
   - Go to Keys tab
   - Click Add Key > Create new key
   - Select JSON format
   - Download and save securely

5. **Grant Search Console Access**
   - Go to [Google Search Console](https://search.google.com/search-console/)
   - Select your property
   - Go to Settings > Users and permissions
   - Add user with service account email
   - Grant Owner permission

#### Code Example: Initialize Client

```rust
use indexer_cli::api::google_indexing::GoogleIndexingClient;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service_account_path = PathBuf::from("./service-account.json");

    // Create client with default settings
    let client = GoogleIndexingClient::new(service_account_path).await?;

    // Or with custom configuration
    let client = GoogleIndexingClient::with_config(
        service_account_path,
        200,  // daily_publish_limit
        380,  // rate_limit_per_minute
        100,  // batch_size
    ).await?;

    Ok(())
}
```

### API Operations

#### Submit URL Update

Notify Google that a URL was added or updated:

```rust
use indexer_cli::api::google_indexing::{GoogleIndexingClient, NotificationType};

async fn submit_url_update(
    client: &GoogleIndexingClient,
    url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let result = client.publish_url(url, NotificationType::UrlUpdated).await?;

    if result.success {
        println!("Successfully submitted: {}", result.url);
    } else {
        println!("Failed: {}", result.message);
    }

    Ok(())
}
```

#### Submit URL Deletion

Notify Google that a URL was removed:

```rust
async fn submit_url_deletion(
    client: &GoogleIndexingClient,
    url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let result = client.publish_url(url, NotificationType::UrlDeleted).await?;

    if result.success {
        println!("Successfully notified deletion: {}", result.url);
    }

    Ok(())
}
```

#### Batch Submit URLs

Submit multiple URLs efficiently:

```rust
async fn batch_submit(
    client: &GoogleIndexingClient,
    urls: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let result = client.batch_publish_urls(
        urls,
        NotificationType::UrlUpdated,
    ).await?;

    println!("Total: {}", result.total);
    println!("Successful: {}", result.successful);
    println!("Failed: {}", result.failed);

    // Check individual results
    for submission in result.results {
        if !submission.success {
            println!("Failed URL: {} - {}", submission.url, submission.message);
        }
    }

    Ok(())
}
```

#### Get URL Metadata

Check the indexing status of a URL:

```rust
async fn get_url_status(
    client: &GoogleIndexingClient,
    url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let metadata = client.get_metadata(url).await?;

    if let Some(latest_update) = metadata.url_notification_metadata.latest_update {
        println!("URL: {}", latest_update.url);
        println!("Type: {}", latest_update.notification_type);
        println!("Notify Time: {:?}", latest_update.notify_time);
    } else {
        println!("No indexing information found for this URL");
    }

    Ok(())
}
```

#### Check Quota

Monitor your API quota usage:

```rust
async fn check_quota(
    client: &GoogleIndexingClient,
) -> Result<(), Box<dyn std::error::Error>> {
    let quota = client.check_quota().await?;

    println!("Daily publish limit: {}", quota.daily_publish_limit);
    println!("Daily publish used: {}", quota.daily_publish_used);
    println!("Rate limit per minute: {}", quota.rate_limit_per_minute);

    Ok(())
}
```

### Rate Limits and Quotas

#### Quota Types

1. **Daily Publish Limit**: 200 URL notifications per day
   - Includes both URL_UPDATED and URL_DELETED
   - Resets at midnight Pacific Time

2. **Request Rate Limit**: 380 requests per minute (total)
   - Covers all API methods
   - Includes publish and getMetadata calls

3. **Metadata Rate Limit**: 180 requests per minute
   - Specific to getMetadata calls

#### Automatic Rate Limiting

The client automatically handles rate limiting:

```rust
// The client waits automatically when approaching limits
for url in urls {
    // This will automatically wait if rate limit is reached
    let result = client.publish_url(url, NotificationType::UrlUpdated).await?;
}
```

#### Custom Rate Limiting

Configure custom rate limits:

```rust
let client = GoogleIndexingClient::with_config(
    service_account_path,
    200,  // daily_publish_limit
    200,  // rate_limit_per_minute (lower than default)
    50,   // batch_size (smaller batches)
).await?;
```

### Error Handling (Google)

#### Error Types

```rust
use indexer_cli::types::IndexerError;

match client.publish_url(url, NotificationType::UrlUpdated).await {
    Ok(result) => {
        if result.success {
            println!("Success!");
        } else {
            println!("Submission failed: {}", result.message);
        }
    }
    Err(IndexerError::GoogleAuthError { message }) => {
        eprintln!("Authentication failed: {}", message);
    }
    Err(IndexerError::GooglePermissionDenied { message }) => {
        eprintln!("Permission denied: {}", message);
        eprintln!("Ensure service account has Owner access in Search Console");
    }
    Err(IndexerError::GoogleRateLimitExceeded) => {
        eprintln!("Rate limit exceeded. Wait and try again.");
    }
    Err(IndexerError::GoogleInvalidRequest { message }) => {
        eprintln!("Invalid request: {}", message);
    }
    Err(e) => {
        eprintln!("Unexpected error: {}", e);
    }
}
```

#### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `GoogleAuthError` | Invalid service account key | Verify service account JSON is valid |
| `GooglePermissionDenied` | Missing Search Console access | Grant Owner permission to service account |
| `GoogleRateLimitExceeded` | Too many requests | Wait for rate limit window to reset |
| `GoogleInvalidRequest` | Malformed URL or request | Validate URL format |
| `GoogleServiceAccountNotFound` | Missing credentials file | Check file path exists |

#### Retry Strategy

The client automatically retries transient errors:

```rust
// Automatic retry with exponential backoff
let result = client.publish_url(url, NotificationType::UrlUpdated).await?;

// The client will retry:
// - Network errors
// - 5xx server errors
// - Rate limit errors (after waiting)
```

### Best Practices (Google)

#### 1. Verify Ownership First

Only submit URLs for properties you own and have verified in Google Search Console.

#### 2. Use Appropriate Notification Type

- **URL_UPDATED**: For new or modified pages
- **URL_DELETED**: For removed pages (don't use for temporary outages)

#### 3. Batch Similar Operations

Group URL submissions together for better efficiency:

```rust
// Good: Batch submission
let urls = vec![/* multiple URLs */];
client.batch_publish_urls(urls, NotificationType::UrlUpdated).await?;

// Avoid: Individual submissions in loop
for url in urls {
    client.publish_url(url, NotificationType::UrlUpdated).await?; // Slower
}
```

#### 4. Don't Resubmit Unnecessarily

Track submission history to avoid resubmitting the same URLs:

```rust
// Use history tracking to filter already-submitted URLs
let new_urls = filter_new_urls(urls, history_db);
client.batch_publish_urls(new_urls, NotificationType::UrlUpdated).await?;
```

#### 5. Monitor Quota Usage

Keep track of daily quota to avoid hitting limits:

```rust
let quota = client.check_quota().await?;
if quota.daily_publish_used >= quota.daily_publish_limit - 10 {
    println!("Warning: Approaching daily quota limit");
}
```

#### 6. Handle Errors Gracefully

Don't fail entire batch if one URL fails:

```rust
let result = client.batch_publish_urls(urls, NotificationType::UrlUpdated).await?;

// Check individual results
for submission in result.results {
    if !submission.success {
        log::warn!("Failed to submit {}: {}", submission.url, submission.message);
        // Continue with other URLs
    }
}
```

## IndexNow API

### Overview

IndexNow is an open protocol that allows website owners to instantly notify search engines about content changes. Multiple search engines support this protocol.

**Official Documentation**: [IndexNow.org](https://www.indexnow.org/)

**Key Features**:
- Instant notification to multiple search engines
- No daily quota limits
- Simple API key authentication
- Batch submission up to 10,000 URLs
- Free to use

**Supported Search Engines**:
- Microsoft Bing
- Yandex
- Seznam.cz
- Naver

### Authentication

IndexNow uses a simple API key authentication.

#### Generate API Key

```rust
use indexer_cli::api::indexnow::IndexNowClient;

// Generate a new 32-character key
let api_key = IndexNowClient::generate_key(32)?;
println!("API Key: {}", api_key);

// Example output: 3f8a9c2b1d7e6f5a4c3b2a1d7e6f5a4c
```

#### Key Requirements

- **Length**: 8-128 characters (32 recommended)
- **Characters**: Alphanumeric (a-z, A-Z, 0-9) and hyphens (-)
- **Format**: Hexadecimal is recommended

#### Host Key File

Create a text file with your API key and host it at your domain root:

1. Create file: `{api_key}.txt`
2. Content: Exactly your API key (no extra whitespace)
3. Upload to: `https://yourdomain.com/{api_key}.txt`

Example:
```bash
echo -n "3f8a9c2b1d7e6f5a4c3b2a1d7e6f5a4c" > 3f8a9c2b1d7e6f5a4c3b2a1d7e6f5a4c.txt
# Upload to web server root
```

#### Initialize Client

```rust
use indexer_cli::api::indexnow::IndexNowClient;

let client = IndexNowClient::new(
    "3f8a9c2b1d7e6f5a4c3b2a1d7e6f5a4c".to_string(),
    "https://example.com/3f8a9c2b1d7e6f5a4c3b2a1d7e6f5a4c.txt".to_string(),
    vec![
        "https://api.indexnow.org/indexnow".to_string(),
        "https://www.bing.com/indexnow".to_string(),
        "https://yandex.com/indexnow".to_string(),
    ],
)?;

// Or use default endpoints
let client = IndexNowClient::with_default_endpoints(
    "3f8a9c2b1d7e6f5a4c3b2a1d7e6f5a4c".to_string(),
    "https://example.com/3f8a9c2b1d7e6f5a4c3b2a1d7e6f5a4c.txt".to_string(),
)?;
```

### API Operations

#### Submit Single URL

```rust
async fn submit_single_url(
    client: &IndexNowClient,
    url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = client.submit_url(
        url,
        "https://api.indexnow.org/indexnow",
    ).await?;

    if response.is_success() {
        println!("Successfully submitted: {}", url);
    } else if response.is_pending_verification() {
        println!("Accepted, key verification pending");
    }

    Ok(())
}
```

#### Submit Multiple URLs

```rust
async fn submit_batch(
    client: &IndexNowClient,
    urls: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = client.submit_urls(
        &urls,
        "https://api.indexnow.org/indexnow",
    ).await?;

    println!("Status: {}", response.status_code);
    println!("Message: {}", response.message);

    Ok(())
}
```

#### Submit to All Endpoints

```rust
async fn submit_to_all(
    client: &IndexNowClient,
    urls: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let results = client.submit_to_all(&urls).await;

    for result in results {
        match result {
            Ok(response) => {
                println!("Endpoint: {} - Status: {}",
                    response.endpoint,
                    response.status_code
                );
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }

    Ok(())
}
```

#### Verify Key File

```rust
async fn verify_key(
    client: &IndexNowClient,
) -> Result<(), Box<dyn std::error::Error>> {
    client.verify_key_file("example.com").await?;
    println!("Key file verified successfully");
    Ok(())
}
```

### Endpoints

#### Primary Endpoint

**api.indexnow.org**: Main endpoint that distributes to all participating search engines.

```
GET https://api.indexnow.org/indexnow?url={url}&key={key}
```

#### Search Engine Specific

**Bing**:
```
https://www.bing.com/indexnow
```

**Yandex**:
```
https://yandex.com/indexnow
```

#### Request Format

**Single URL (GET)**:
```
GET https://api.indexnow.org/indexnow?url=https://example.com/page&key=your-key
```

**Batch (POST)**:
```json
POST https://api.indexnow.org/indexnow
Content-Type: application/json

{
  "host": "example.com",
  "key": "your-key",
  "keyLocation": "https://example.com/your-key.txt",
  "urlList": [
    "https://example.com/page1",
    "https://example.com/page2"
  ]
}
```

### Error Handling (IndexNow)

#### Response Codes

| Code | Meaning | Action |
|------|---------|--------|
| 200 | URL submitted successfully | None |
| 202 | URL received, key validation pending | Wait for verification |
| 400 | Bad request (invalid format) | Check request format |
| 403 | Forbidden (invalid key) | Verify API key and key file |
| 422 | Unprocessable (URL doesn't match host) | Ensure URLs match host |
| 429 | Too many requests | Implement rate limiting |

#### Error Handling Example

```rust
use indexer_cli::types::IndexerError;

match client.submit_url(url, endpoint).await {
    Ok(response) => {
        match response.status_code {
            200 => println!("Success!"),
            202 => println!("Pending verification"),
            _ => println!("Unexpected status: {}", response.status_code),
        }
    }
    Err(IndexerError::IndexNowInvalidKey) => {
        eprintln!("Invalid API key. Check your key configuration.");
    }
    Err(IndexerError::IndexNowBadRequest { message }) => {
        eprintln!("Bad request: {}", message);
    }
    Err(IndexerError::IndexNowUnprocessableEntity { message }) => {
        eprintln!("URL doesn't match host or key mismatch: {}", message);
    }
    Err(IndexerError::IndexNowRateLimitExceeded) => {
        eprintln!("Rate limit exceeded. Wait before retrying.");
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

### Best Practices (IndexNow)

#### 1. Keep Your Key Secure

- Store key in environment variables or config files
- Don't commit key to version control
- Regenerate if compromised

#### 2. Host Key File Properly

- Key file must be publicly accessible via HTTPS
- File content must exactly match API key
- Host at domain root, not subdirectory

#### 3. Batch URLs Efficiently

```rust
// Good: Batch submission (up to 10,000 URLs)
let urls = vec![/* multiple URLs */];
client.submit_urls(&urls, endpoint).await?;

// Avoid: Individual submissions (inefficient)
for url in urls {
    client.submit_url(url, endpoint).await?; // Slow, more requests
}
```

#### 4. Submit to Primary Endpoint

Submit to `api.indexnow.org` to reach all search engines:

```rust
// Reaches all search engines
client.submit_url(url, "https://api.indexnow.org/indexnow").await?;
```

#### 5. Don't Spam

- Only submit when content actually changes
- Don't resubmit the same URL repeatedly
- Use reasonable submission frequency

#### 6. Verify Key File

Test key file accessibility before first submission:

```rust
match client.verify_key_file("example.com").await {
    Ok(_) => println!("Key file verified"),
    Err(e) => eprintln!("Key file not accessible: {}", e),
}
```

## Integration Examples

### Complete Workflow Example

```rust
use indexer_cli::api::{google_indexing::*, indexnow::*};
use indexer_cli::services::batch_submitter::*;
use indexer_cli::database;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize database
    let db_conn = database::init_database(
        &std::path::PathBuf::from("./indexer.db")
    )?;
    let history_manager = Arc::new(HistoryManager::new(db_conn));

    // Initialize Google client
    let google_client = GoogleIndexingClient::new(
        std::path::PathBuf::from("./service-account.json")
    ).await?;

    // Initialize IndexNow client
    let indexnow_client = IndexNowClient::with_default_endpoints(
        "your-api-key".to_string(),
        "https://example.com/your-api-key.txt".to_string(),
    )?;

    // Create batch submitter
    let submitter = BatchSubmitter::new(
        Some(Arc::new(google_client)),
        Some(Arc::new(indexnow_client)),
        history_manager,
        BatchConfig::default(),
    );

    // Submit URLs to all APIs
    let urls = vec![
        "https://example.com/page1".to_string(),
        "https://example.com/page2".to_string(),
    ];

    let result = submitter.submit_to_all(
        urls,
        NotificationType::UrlUpdated,
    ).await?;

    println!("Total URLs: {}", result.total_urls);
    println!("Submitted: {}", result.submitted);
    println!("Skipped: {}", result.skipped);
    println!("Successful: {}", result.total_successful());
    println!("Failed: {}", result.total_failed());

    Ok(())
}
```

### Sitemap Processing Example

```rust
use indexer_cli::services::sitemap_parser::*;
use indexer_cli::api::google_indexing::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse sitemap
    let parser = SitemapParser::new()?;
    let result = parser.parse_sitemap(
        "https://example.com/sitemap.xml",
        None,
    ).await?;

    println!("Found {} URLs", result.urls.len());

    // Extract URL strings
    let urls: Vec<String> = result.urls
        .iter()
        .map(|u| u.loc.clone())
        .collect();

    // Submit to Google
    let client = GoogleIndexingClient::new(
        std::path::PathBuf::from("./service-account.json")
    ).await?;

    let submission_result = client.batch_publish_urls(
        urls,
        NotificationType::UrlUpdated,
    ).await?;

    println!("Submitted {} URLs successfully",
        submission_result.successful
    );

    Ok(())
}
```

## Troubleshooting

### Google API Issues

**Problem**: `GoogleAuthError`
- **Cause**: Invalid service account credentials
- **Solution**: Verify JSON file is valid and not corrupted

**Problem**: `GooglePermissionDenied`
- **Cause**: Service account not granted access
- **Solution**: Add service account email to Search Console with Owner permission

**Problem**: `GoogleRateLimitExceeded`
- **Cause**: Too many requests
- **Solution**: Client automatically handles this; ensure you're not bypassing rate limiting

### IndexNow Issues

**Problem**: `IndexNowInvalidKey`
- **Cause**: API key format is invalid
- **Solution**: Regenerate key with correct length and characters

**Problem**: `IndexNowKeyFileNotAccessible`
- **Cause**: Key file not publicly accessible
- **Solution**: Verify file is at domain root and accessible via HTTPS

**Problem**: Status 422
- **Cause**: URL doesn't match host in key_location
- **Solution**: Ensure all URLs are from the same domain as key file

### General Issues

**Problem**: Network timeouts
- **Cause**: Slow connection or large request
- **Solution**: Adjust timeout in client configuration

**Problem**: SSL/TLS errors
- **Cause**: Certificate validation failure
- **Solution**: Ensure system certificates are up to date

---

For more information, see:
- [README.md](../README.md) - General usage and setup
- [ARCHITECTURE.md](ARCHITECTURE.md) - Technical architecture details
- [Google Indexing API Docs](https://developers.google.com/search/apis/indexing-api/v3/quickstart)
- [IndexNow Protocol](https://www.indexnow.org/)
