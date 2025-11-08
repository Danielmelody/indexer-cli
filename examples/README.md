# Examples

This directory contains practical examples demonstrating how to use the indexer-cli library in your own Rust projects.

## Table of Contents

- [Basic Usage](#basic-usage)
- [Available Examples](#available-examples)
- [Running Examples](#running-examples)
- [Example Topics](#example-topics)

## Basic Usage

The examples in this directory demonstrate using indexer-cli as a library in your own Rust applications. Each example is a standalone program that can be run with `cargo run --example <name>`.

## Available Examples

### basic_usage.rs

Demonstrates basic usage of the indexer-cli library including:
- Loading configuration
- Initializing API clients
- Submitting URLs to Google Indexing API
- Submitting URLs to IndexNow API
- Basic error handling

**Run with:**
```bash
cargo run --example basic_usage
```

### Configuration

Before running the examples, you need to set up configuration:

1. **Create configuration directory:**
   ```bash
   mkdir -p ~/.indexer-cli
   ```

2. **Create config file** (`~/.indexer-cli/config.yaml`):
   ```yaml
   google:
     enabled: true
     service_account_file: ~/.indexer-cli/service-account.json
     batch_size: 100

   indexnow:
     enabled: true
     api_key: your-32-character-api-key
     key_location: https://yourdomain.com/your-api-key.txt
     endpoints:
       - https://api.indexnow.org/indexnow

   history:
     enabled: true
     database_path: ~/.indexer-cli/history.db
   ```

3. **Set up Google credentials:**
   - Download your Google service account JSON file
   - Save it to `~/.indexer-cli/service-account.json`

4. **Set up IndexNow key:**
   - Generate an API key: `indexer-cli indexnow generate-key --length 32`
   - Create key file and upload to your website root

## Running Examples

### Prerequisites

- Rust 1.70 or higher
- Valid Google service account (for Google API examples)
- Valid IndexNow API key (for IndexNow examples)

### Run an Example

```bash
# Run basic usage example
cargo run --example basic_usage

# Run with debug logging
RUST_LOG=debug cargo run --example basic_usage

# Build and run optimized version
cargo build --release --example basic_usage
./target/release/examples/basic_usage
```

### Environment Variables

Some examples support environment variables:

```bash
# Set Google service account path
export INDEXER_GOOGLE_SERVICE_ACCOUNT=/path/to/service-account.json

# Set IndexNow API key
export INDEXER_INDEXNOW_API_KEY=your-api-key

# Set config file location
export INDEXER_CONFIG=/path/to/config.yaml

# Run example
cargo run --example basic_usage
```

## Example Topics

### 1. Google Indexing API

**Examples demonstrating:**
- Client initialization with service account
- Single URL submission
- Batch URL submission
- Getting URL metadata
- Checking quota usage
- Error handling
- Rate limiting

**Key code patterns:**

```rust
// Initialize client
let client = GoogleIndexingClient::new(service_account_path).await?;

// Submit single URL
let result = client.publish_url(
    "https://example.com/page",
    NotificationType::UrlUpdated
).await?;

// Batch submission
let urls = vec![
    "https://example.com/page1".to_string(),
    "https://example.com/page2".to_string(),
];
let result = client.batch_publish_urls(
    urls,
    NotificationType::UrlUpdated
).await?;
```

### 2. IndexNow API

**Examples demonstrating:**
- Client initialization
- Single URL submission
- Batch URL submission
- Multi-endpoint submission
- Key file verification
- Error handling

**Key code patterns:**

```rust
// Initialize client
let client = IndexNowClient::new(
    api_key,
    key_location,
    endpoints,
)?;

// Submit to single endpoint
let response = client.submit_url(
    "https://example.com/page",
    "https://api.indexnow.org/indexnow"
).await?;

// Submit to all endpoints
let results = client.submit_to_all(&urls).await;
```

### 3. Sitemap Parsing

**Examples demonstrating:**
- Parsing XML sitemaps
- Handling sitemap indexes
- URL filtering
- Gzip decompression
- Recursive parsing

**Key code patterns:**

```rust
// Initialize parser
let parser = SitemapParser::new()?;

// Parse sitemap
let result = parser.parse_sitemap(
    "https://example.com/sitemap.xml",
    None
).await?;

// With filters
let filters = SitemapFilters {
    url_pattern: Some(Regex::new(r"^https://example.com/blog/")?),
    lastmod_after: Some(Utc::now() - Duration::days(7)),
    priority_min: Some(0.5),
};

let result = parser.parse_sitemap(
    "https://example.com/sitemap.xml",
    Some(&filters)
).await?;
```

### 4. Batch Submission

**Examples demonstrating:**
- Batch submission orchestration
- Progress tracking
- History management
- Concurrent processing
- Error aggregation

**Key code patterns:**

```rust
// Initialize components
let google_client = GoogleIndexingClient::new(path).await?;
let history_manager = Arc::new(HistoryManager::new(db_conn));

let submitter = BatchSubmitter::new(
    Some(Arc::new(google_client)),
    None,
    history_manager,
    BatchConfig::default(),
);

// Submit batch
let result = submitter.submit_to_google(
    urls,
    NotificationType::UrlUpdated
).await?;
```

### 5. History Management

**Examples demonstrating:**
- Recording submissions
- Querying history
- Checking for duplicates
- Filtering URLs
- Statistics gathering

**Key code patterns:**

```rust
// Initialize database
let conn = init_database(&db_path)?;

// Record submission
let record = SubmissionRecord {
    url: "https://example.com/page".to_string(),
    api: ApiType::Google,
    action: ActionType::UrlUpdated,
    status: SubmissionStatus::Success,
    // ...
};
insert_submission(&conn, &record)?;

// Check if URL was submitted
let submitted = check_url_submitted(
    &conn,
    "https://example.com/page",
    ApiType::Google,
    Utc::now() - Duration::days(1)
)?;
```

### 6. Configuration

**Examples demonstrating:**
- Loading configuration
- Merging multiple sources
- Environment variable overrides
- Validation
- Default values

**Key code patterns:**

```rust
// Load configuration
let config = load_config(None)?;

// Access settings
if let Some(google_config) = &config.google {
    println!("Google enabled: {}", google_config.enabled);
    println!("Batch size: {}", google_config.batch_size);
}

// Load with custom path
let config = load_config(Some(PathBuf::from("./custom-config.yaml")))?;
```

### 7. Error Handling

**Examples demonstrating:**
- Error types
- Error propagation
- Retry logic
- User-friendly error messages
- Recovery strategies

**Key code patterns:**

```rust
use indexer_cli::types::IndexerError;

match client.publish_url(url, NotificationType::UrlUpdated).await {
    Ok(result) => {
        if result.success {
            println!("Success!");
        } else {
            eprintln!("Failed: {}", result.message);
        }
    }
    Err(IndexerError::GoogleAuthError { message }) => {
        eprintln!("Authentication failed: {}", message);
    }
    Err(IndexerError::GoogleRateLimitExceeded) => {
        eprintln!("Rate limit exceeded, waiting...");
        tokio::time::sleep(Duration::from_secs(60)).await;
        // Retry...
    }
    Err(e) => {
        eprintln!("Unexpected error: {}", e);
    }
}
```

### 8. Async Patterns

**Examples demonstrating:**
- Tokio runtime usage
- Concurrent operations
- Task spawning
- Timeout handling
- Cancellation

**Key code patterns:**

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Concurrent submissions
    let mut tasks = vec![];

    for url in urls {
        let client = client.clone();
        let task = tokio::spawn(async move {
            client.publish_url(&url, NotificationType::UrlUpdated).await
        });
        tasks.push(task);
    }

    // Wait for all tasks
    for task in tasks {
        let result = task.await??;
        println!("Result: {:?}", result);
    }

    Ok(())
}
```

## Creating Your Own Example

To create a new example:

1. **Create a new file** in the `examples/` directory:
   ```bash
   touch examples/my_example.rs
   ```

2. **Add example code**:
   ```rust
   use indexer_cli::prelude::*;

   #[tokio::main]
   async fn main() -> Result<(), Box<dyn std::error::Error>> {
       // Your code here
       Ok(())
   }
   ```

3. **Run your example**:
   ```bash
   cargo run --example my_example
   ```

## Best Practices

When creating examples:

1. **Keep it simple** - Focus on one concept per example
2. **Add comments** - Explain what each section does
3. **Handle errors** - Show proper error handling
4. **Use realistic data** - Use example.com or similar
5. **Document requirements** - Note any prerequisites
6. **Test before committing** - Ensure examples work

## Example Template

Use this template for new examples:

```rust
//! Example: Description of what this example demonstrates
//!
//! This example shows how to:
//! - Feature 1
//! - Feature 2
//! - Feature 3
//!
//! ## Prerequisites
//!
//! - Requirement 1
//! - Requirement 2
//!
//! ## Usage
//!
//! ```bash
//! cargo run --example example_name
//! ```

use indexer_cli::prelude::*;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Your example code here
    println!("Example output");

    Ok(())
}
```

## Getting Help

If you have questions about the examples:

1. Check the [API documentation](../docs/API.md)
2. Read the [architecture docs](../docs/ARCHITECTURE.md)
3. Open an [issue](https://github.com/your-username/indexer-cli/issues)
4. Ask in [discussions](https://github.com/your-username/indexer-cli/discussions)

## Contributing

Want to add an example? See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines on contributing examples.

## License

All examples are licensed under the same terms as the main project (MIT License).
