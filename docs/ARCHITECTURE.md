# Architecture Documentation

This document provides a detailed overview of the indexer-cli architecture, design decisions, and implementation details.

## Table of Contents

- [System Overview](#system-overview)
- [Architecture Diagram](#architecture-diagram)
- [Module Organization](#module-organization)
- [Data Flow](#data-flow)
- [Core Components](#core-components)
- [Database Design](#database-design)
- [API Integration](#api-integration)
- [Error Handling](#error-handling)
- [Async/Concurrency Model](#asyncconcurrency-model)
- [Configuration System](#configuration-system)
- [Testing Strategy](#testing-strategy)
- [Performance Considerations](#performance-considerations)
- [Security Considerations](#security-considerations)
- [Design Decisions](#design-decisions)

## System Overview

indexer-cli is a production-ready command-line tool designed to automate URL submissions to search engines through multiple APIs. The system is built with Rust for performance, safety, and reliability.

### Key Design Goals

1. **Reliability** - Robust error handling, automatic retries, persistent state
2. **Performance** - Concurrent processing, efficient batching, minimal overhead
3. **Usability** - Clear CLI interface, comprehensive documentation, helpful errors
4. **Maintainability** - Modular design, clear separation of concerns, extensive tests
5. **Extensibility** - Easy to add new APIs, commands, and features

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        User Interface                        │
│                    (CLI - clap-based)                       │
└────────────────────────┬────────────────────────────────────┘
                         │
                         │ Commands
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                     Command Layer                            │
│   init │ config │ google │ indexnow │ submit │ sitemap...  │
└────────┬────────────────────────────────┬───────────────────┘
         │                                │
         │ Business Logic                 │ Data Access
         ▼                                ▼
┌────────────────────────────┐  ┌──────────────────────────┐
│    Services Layer          │  │    Database Layer        │
│  - BatchSubmitter          │  │  - Schema Management     │
│  - SitemapParser           │  │  - Query Operations      │
│  - HistoryManager          │  │  - Model Definitions     │
│  - URLProcessor            │  │  - SQLite (WAL mode)     │
└────────┬───────────────────┘  └──────────────────────────┘
         │
         │ External APIs
         ▼
┌─────────────────────────────────────────────────────────────┐
│                      API Clients                             │
│  - GoogleIndexingClient (OAuth2)                            │
│  - IndexNowClient (API Key)                                 │
└─────────────────────────────────────────────────────────────┘
         │
         │ HTTP/HTTPS
         ▼
┌─────────────────────────────────────────────────────────────┐
│                   External Services                          │
│  - Google Indexing API                                      │
│  - IndexNow (Bing, Yandex, Seznam, Naver)                 │
└─────────────────────────────────────────────────────────────┘
```

## Architecture Diagram

### Component Interaction

```
┌──────────────┐
│     User     │
└──────┬───────┘
       │ Commands
       ▼
┌──────────────────────────────────────────────────────────────┐
│                         CLI Layer                             │
│  ┌───────────┐  ┌──────────┐  ┌──────────────────────────┐ │
│  │   Args    │→│ Handler  │→│   Command Dispatcher      │ │
│  │  Parser   │  │ Validator│  │                          │ │
│  └───────────┘  └──────────┘  └──────────────────────────┘ │
└───────────────────────────┬──────────────────────────────────┘
                            │
                            ▼
┌──────────────────────────────────────────────────────────────┐
│                    Configuration Layer                        │
│  ┌────────────┐  ┌──────────┐  ┌────────────────────────┐  │
│  │  Loader    │→│ Validator│→│   Settings Struct       │  │
│  │ (YAML/Env) │  │          │  │                        │  │
│  └────────────┘  └──────────┘  └────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌──────────────────────────────────────────────────────────────┐
│                   Services Layer                              │
│  ┌──────────────────┐  ┌────────────────┐  ┌─────────────┐ │
│  │ BatchSubmitter   │  │ SitemapParser  │  │   History   │ │
│  │                  │  │                │  │   Manager   │ │
│  │ - Filter URLs    │  │ - Parse XML    │  │             │ │
│  │ - Batch Process  │  │ - Extract URLs │  │ - Check Dups│ │
│  │ - Progress Track │  │ - Apply Filters│  │ - Record    │ │
│  │ - Error Handling │  │ - Recursion    │  │ - Query     │ │
│  └────────┬─────────┘  └────────────────┘  └──────┬──────┘ │
└───────────┼─────────────────────────────────────────┼────────┘
            │                                          │
            ▼                                          ▼
┌────────────────────────┐                  ┌──────────────────┐
│    API Clients         │                  │    Database      │
│                        │                  │                  │
│ ┌────────────────────┐ │                  │  ┌────────────┐ │
│ │ GoogleIndexing     │ │                  │  │  SQLite    │ │
│ │ - Authenticate     │ │                  │  │  (WAL)     │ │
│ │ - Rate Limit       │ │                  │  │            │ │
│ │ - Submit           │ │                  │  │ Tables:    │ │
│ │ - Get Metadata     │ │                  │  │ - history  │ │
│ └────────────────────┘ │                  │  │ - schema   │ │
│                        │                  │  └────────────┘ │
│ ┌────────────────────┐ │                  │                  │
│ │ IndexNow           │ │                  │  Indexes:       │
│ │ - Validate Key     │ │                  │  - url          │
│ │ - Submit Single    │ │                  │  - api          │
│ │ - Submit Batch     │ │                  │  - status       │
│ │ - Multi-Endpoint   │ │                  │  - submitted_at │
│ └────────────────────┘ │                  └──────────────────┘
└────────────────────────┘
            │
            ▼
    External APIs
```

## Module Organization

### Source Tree Structure

```
src/
├── main.rs                 # Binary entry point
├── lib.rs                  # Library root, public API exports
├── constants.rs            # Global constants
│
├── cli/                    # Command-line interface
│   ├── mod.rs              # Module exports
│   ├── args.rs             # Argument definitions (clap)
│   └── handler.rs          # Command routing and execution
│
├── commands/               # Command implementations
│   ├── mod.rs
│   ├── init.rs             # Initialize configuration
│   ├── config.rs           # Configuration management
│   ├── google.rs           # Google API commands
│   ├── indexnow.rs         # IndexNow API commands
│   ├── submit.rs           # Unified submission
│   ├── sitemap.rs          # Sitemap operations
│   ├── history.rs          # History management
│   ├── watch.rs            # Watch mode
│   └── validate.rs         # Validation commands
│
├── api/                    # External API clients
│   ├── mod.rs
│   ├── google_indexing.rs  # Google Indexing API client
│   └── indexnow.rs         # IndexNow API client
│
├── services/               # Business logic layer
│   ├── mod.rs
│   ├── batch_submitter.rs  # Batch submission orchestration
│   ├── sitemap_parser.rs   # XML sitemap parsing
│   ├── history_manager.rs  # History tracking
│   └── url_processor.rs    # URL validation and processing
│
├── database/               # Data persistence layer
│   ├── mod.rs
│   ├── schema.rs           # Schema definition and migrations
│   ├── models.rs           # Data models
│   └── queries.rs          # SQL queries
│
├── config/                 # Configuration management
│   ├── mod.rs
│   ├── settings.rs         # Settings structures
│   ├── loader.rs           # Config file loading
│   └── validation.rs       # Config validation
│
├── types/                  # Type definitions
│   ├── mod.rs
│   ├── error.rs            # Error types
│   └── result.rs           # Result type alias
│
└── utils/                  # Utility functions
    ├── mod.rs
    ├── retry.rs            # Retry logic with backoff
    ├── logger.rs           # Logging setup
    ├── file.rs             # File operations
    └── validators.rs       # Input validation
```

### Module Responsibilities

#### CLI Layer (`cli/`)

- **Purpose**: User interface and command parsing
- **Responsibilities**:
  - Parse command-line arguments using clap
  - Validate user input
  - Route commands to appropriate handlers
  - Format and display output
- **Dependencies**: Minimal, only on types and commands

#### Commands Layer (`commands/`)

- **Purpose**: Command implementation and orchestration
- **Responsibilities**:
  - Implement business logic for each command
  - Coordinate between services and APIs
  - Handle command-specific error cases
  - Format command-specific output
- **Dependencies**: Services, APIs, Database, Config

#### API Layer (`api/`)

- **Purpose**: External API integration
- **Responsibilities**:
  - Implement API client logic
  - Handle authentication (OAuth2, API keys)
  - Manage rate limiting and quotas
  - Parse API responses
  - Implement retry logic
- **Dependencies**: HTTP client (reqwest), Auth libraries

#### Services Layer (`services/`)

- **Purpose**: Core business logic
- **Responsibilities**:
  - Batch processing orchestration
  - Sitemap parsing and URL extraction
  - Submission history management
  - URL filtering and validation
  - Progress tracking
- **Dependencies**: API clients, Database

#### Database Layer (`database/`)

- **Purpose**: Data persistence
- **Responsibilities**:
  - Database schema management
  - SQL query execution
  - Data model definitions
  - Migrations
- **Dependencies**: rusqlite

#### Configuration Layer (`config/`)

- **Purpose**: Application configuration
- **Responsibilities**:
  - Load configuration from files and environment
  - Validate configuration values
  - Provide defaults
  - Merge configuration sources
- **Dependencies**: serde, serde_yaml

#### Types Layer (`types/`)

- **Purpose**: Shared type definitions
- **Responsibilities**:
  - Define error types
  - Define result types
  - Shared enums and structs
- **Dependencies**: Minimal

#### Utils Layer (`utils/`)

- **Purpose**: Reusable utilities
- **Responsibilities**:
  - Retry logic with exponential backoff
  - Logging configuration
  - File I/O helpers
  - Validation helpers
- **Dependencies**: Various, depending on utility

## Data Flow

### Submission Flow (Google API)

```
User Command
    │
    ▼
Parse Arguments (URLs, file, or sitemap)
    │
    ▼
Load Configuration
    │
    ├─→ Google credentials
    ├─→ Batch size
    ├─→ Quota limits
    └─→ Retry settings
    │
    ▼
Initialize Google Client
    │
    └─→ Authenticate with OAuth2
    │
    ▼
Collect URLs
    │
    ├─→ Direct URLs: validate and prepare
    ├─→ From file: read and parse
    └─→ From sitemap: download, parse, extract
    │
    ▼
Filter URLs (if history check enabled)
    │
    └─→ Query database for recent submissions
    │
    ▼
Split into Batches (batch_size = 100)
    │
    ▼
Process Batches Concurrently
    │
    ├─→ Rate Limiting: wait if needed
    ├─→ Submit URL to Google API
    │   ├─→ Get access token
    │   ├─→ Make HTTP POST request
    │   ├─→ Handle response
    │   └─→ Retry on failure (exponential backoff)
    │
    ▼
Record Results to Database
    │
    ├─→ URL
    ├─→ API (google)
    ├─→ Status (success/failed)
    ├─→ Response code
    ├─→ Response message
    └─→ Timestamp
    │
    ▼
Display Results
    │
    ├─→ Progress bars during processing
    ├─→ Summary (success/failed counts)
    └─→ Format output (text, JSON, CSV)
```

### Submission Flow (IndexNow)

```
User Command
    │
    ▼
Parse Arguments
    │
    ▼
Load Configuration
    │
    ├─→ API key
    ├─→ Key location URL
    ├─→ Endpoints
    └─→ Batch size
    │
    ▼
Initialize IndexNow Client
    │
    ├─→ Validate API key format
    └─→ Validate key location URL
    │
    ▼
Collect URLs
    │
    ▼
Filter URLs (if history check enabled)
    │
    ▼
Split into Batches (batch_size = 10,000)
    │
    ▼
For Each Batch:
    │
    ├─→ Submit to All Endpoints Concurrently
    │   ├─→ api.indexnow.org
    │   ├─→ www.bing.com/indexnow
    │   └─→ yandex.com/indexnow
    │
    ├─→ Each Endpoint:
    │   ├─→ Build request (GET for 1 URL, POST for batch)
    │   ├─→ Make HTTP request
    │   ├─→ Handle response (200, 202, 400, 403, 422, 429)
    │   └─→ Retry on failure
    │
    ▼
Record Results to Database
    │
    ▼
Display Results
```

### Sitemap Parsing Flow

```
Sitemap URL
    │
    ▼
Validate URL
    │
    ▼
Download Content (HTTP GET)
    │
    ├─→ Check Content-Length (< 50MB)
    ├─→ Download with timeout
    └─→ Detect gzip compression
    │
    ▼
Decompress if Gzipped
    │
    ▼
Parse XML
    │
    ├─→ Detect type (regular sitemap vs index)
    │
    ├─→ If Regular Sitemap:
    │   ├─→ Extract <url> elements
    │   ├─→ Parse <loc>, <lastmod>, <priority>, <changefreq>
    │   ├─→ Deduplicate URLs
    │   └─→ Apply filters (pattern, date, priority)
    │
    └─→ If Sitemap Index:
        ├─→ Extract <sitemap> <loc> elements
        ├─→ For each child sitemap:
        │   ├─→ Recursively download and parse
        │   ├─→ Check recursion depth limit
        │   └─→ Aggregate results
        └─→ Apply filters to aggregated URLs
    │
    ▼
Return URLs
```

## Core Components

### BatchSubmitter

**Purpose**: Orchestrate batch submission to APIs with progress tracking and error handling.

**Key Features**:
- Concurrent batch processing
- History-based URL filtering
- Progress bar display
- Error aggregation
- Retry handling

**Implementation Details**:
```rust
pub struct BatchSubmitter {
    google_client: Option<Arc<GoogleIndexingClient>>,
    indexnow_client: Option<Arc<IndexNowClient>>,
    history_manager: Arc<HistoryManager>,
    config: BatchConfig,
}

impl BatchSubmitter {
    // Submit to Google with automatic batching
    pub async fn submit_to_google(
        &self,
        urls: Vec<String>,
        action: NotificationType,
    ) -> Result<BatchResult, IndexerError>

    // Submit to IndexNow with multi-endpoint support
    pub async fn submit_to_indexnow(
        &self,
        urls: Vec<String>,
    ) -> Result<BatchResult, IndexerError>

    // Submit to all enabled APIs concurrently
    pub async fn submit_to_all(
        &self,
        urls: Vec<String>,
        action: NotificationType,
    ) -> Result<BatchResult, IndexerError>
}
```

### SitemapParser

**Purpose**: Parse XML sitemaps with support for compression, indexes, and filtering.

**Key Features**:
- Gzip decompression
- Recursive index parsing
- URL filtering (pattern, date, priority)
- Size validation
- Deduplication

**Implementation Details**:
```rust
pub struct SitemapParser {
    client: Client,
    max_recursion_depth: usize,
    max_urls: usize,
}

impl SitemapParser {
    // Parse sitemap from URL
    pub async fn parse_sitemap(
        &self,
        url: &str,
        filters: Option<&SitemapFilters>,
    ) -> Result<ParseResult, IndexerError>

    // Parse from XML string
    pub fn parse_sitemap_xml(
        &self,
        xml_content: &str,
    ) -> Result<Vec<SitemapUrl>, IndexerError>
}
```

### GoogleIndexingClient

**Purpose**: Interact with Google Indexing API v3.

**Key Features**:
- OAuth2 service account authentication
- Automatic token refresh
- Rate limiting (380 req/min)
- Quota tracking (200 publish/day)
- Retry with exponential backoff

**Implementation Details**:
```rust
pub struct GoogleIndexingClient {
    client: reqwest::Client,
    auth: Arc<Mutex<Authenticator<HttpsConnector<HttpConnector>>>>,
    rate_limiter: Arc<Mutex<RateLimiter>>,
    daily_publish_limit: usize,
    batch_size: usize,
}

impl GoogleIndexingClient {
    // Authenticate and get access token
    pub async fn authenticate(&self) -> Result<String, IndexerError>

    // Submit single URL
    pub async fn publish_url(
        &self,
        url: &str,
        notification_type: NotificationType,
    ) -> Result<SubmissionResult, IndexerError>

    // Submit batch of URLs
    pub async fn batch_publish_urls(
        &self,
        urls: Vec<String>,
        notification_type: NotificationType,
    ) -> Result<BatchSubmissionResult, IndexerError>

    // Get URL metadata
    pub async fn get_metadata(
        &self,
        url: &str,
    ) -> Result<MetadataResponse, IndexerError>
}
```

### IndexNowClient

**Purpose**: Interact with IndexNow API protocol.

**Key Features**:
- Multiple endpoint support
- Batch submission (up to 10,000 URLs)
- Key validation
- Concurrent endpoint submission

**Implementation Details**:
```rust
pub struct IndexNowClient {
    client: Client,
    api_key: String,
    key_location: String,
    endpoints: Vec<String>,
}

impl IndexNowClient {
    // Submit single URL
    pub async fn submit_url(
        &self,
        url: &str,
        endpoint: &str,
    ) -> Result<IndexNowResponse, IndexerError>

    // Submit batch of URLs
    pub async fn submit_urls(
        &self,
        urls: &[String],
        endpoint: &str,
    ) -> Result<IndexNowResponse, IndexerError>

    // Submit to all configured endpoints
    pub async fn submit_to_all(
        &self,
        urls: &[String],
    ) -> Vec<Result<IndexNowResponse, IndexerError>>

    // Verify key file accessibility
    pub async fn verify_key_file(
        &self,
        host: &str,
    ) -> Result<(), IndexerError>
}
```

## Database Design

### Schema

**submission_history table**:
```sql
CREATE TABLE submission_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL,
    api TEXT NOT NULL,           -- 'google' or 'indexnow'
    action TEXT NOT NULL,         -- 'url-updated' or 'url-deleted'
    status TEXT NOT NULL,         -- 'success' or 'failed'
    response_code INTEGER,
    response_message TEXT,
    submitted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    metadata TEXT                 -- JSON for additional data
);

CREATE INDEX idx_url ON submission_history(url);
CREATE INDEX idx_api ON submission_history(api);
CREATE INDEX idx_status ON submission_history(status);
CREATE INDEX idx_submitted_at ON submission_history(submitted_at);
```

**schema_version table**:
```sql
CREATE TABLE schema_version (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    version INTEGER NOT NULL,
    applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### Models

```rust
pub struct SubmissionRecord {
    pub id: Option<i64>,
    pub url: String,
    pub api: ApiType,
    pub action: ActionType,
    pub status: SubmissionStatus,
    pub response_code: Option<i32>,
    pub response_message: Option<String>,
    pub submitted_at: DateTime<Utc>,
    pub metadata: Option<String>,
}

pub enum ApiType {
    Google,
    IndexNow,
}

pub enum ActionType {
    UrlUpdated,
    UrlDeleted,
}

pub enum SubmissionStatus {
    Success,
    Failed,
}
```

### Query Patterns

**Check if URL was submitted recently**:
```rust
pub fn check_url_submitted(
    conn: &Connection,
    url: &str,
    api: ApiType,
    since: DateTime<Utc>,
) -> Result<bool, IndexerError>
```

**Insert submission record**:
```rust
pub fn insert_submission(
    conn: &Connection,
    record: &SubmissionRecord,
) -> Result<i64, IndexerError>
```

**Query history with filters**:
```rust
pub fn query_history(
    conn: &Connection,
    filters: HistoryFilters,
    limit: usize,
) -> Result<Vec<SubmissionRecord>, IndexerError>
```

**Get statistics**:
```rust
pub fn get_statistics(
    conn: &Connection,
    since: Option<DateTime<Utc>>,
    until: Option<DateTime<Utc>>,
) -> Result<Statistics, IndexerError>
```

### Migration Strategy

- Schema version tracked in `schema_version` table
- Migrations applied sequentially on startup
- Current version: 1 (initial schema)
- Future migrations will be added to `schema.rs`

## API Integration

### Google Indexing API

**Authentication Flow**:
1. Load service account JSON key
2. Create OAuth2 authenticator
3. Request access token with scope: `https://www.googleapis.com/auth/indexing`
4. Cache token until expiration
5. Automatically refresh when needed

**Request Format**:
```json
POST https://indexing.googleapis.com/v3/urlNotifications:publish
Authorization: Bearer {access_token}
Content-Type: application/json

{
  "url": "https://example.com/page",
  "type": "URL_UPDATED"
}
```

**Rate Limiting Strategy**:
- Track request timestamps in memory
- Before each request, check if limit would be exceeded
- If exceeded, calculate wait time and sleep
- Remove expired timestamps from tracking

### IndexNow API

**Single URL Submission**:
```
GET https://api.indexnow.org/indexnow?url={url}&key={key}
```

**Batch Submission**:
```json
POST https://api.indexnow.org/indexnow
Content-Type: application/json

{
  "host": "example.com",
  "key": "your-api-key",
  "keyLocation": "https://example.com/your-api-key.txt",
  "urlList": [
    "https://example.com/page1",
    "https://example.com/page2"
  ]
}
```

**Multi-Endpoint Strategy**:
- Submit to all endpoints concurrently using `tokio::spawn`
- Each endpoint gets same URL list
- Success if any endpoint returns 200/202
- Log warnings for failed endpoints
- Don't fail entire operation if one endpoint fails

## Error Handling

### Error Type Hierarchy

```rust
#[derive(Debug, thiserror::Error)]
pub enum IndexerError {
    // Configuration errors
    #[error("Configuration file not found: {path}")]
    ConfigFileNotFound { path: PathBuf },

    #[error("Invalid configuration: {message}")]
    ConfigInvalid { message: String },

    // Google API errors
    #[error("Google authentication failed: {message}")]
    GoogleAuthError { message: String },

    #[error("Google API error (status {status_code}): {message}")]
    GoogleApiError { status_code: u16, message: String },

    // IndexNow API errors
    #[error("IndexNow invalid API key")]
    IndexNowInvalidKey,

    #[error("IndexNow rate limit exceeded")]
    IndexNowRateLimitExceeded,

    // Sitemap errors
    #[error("Sitemap download failed: {message}")]
    SitemapDownloadFailed { url: String, message: String },

    #[error("Sitemap too large: {size} bytes (limit: {limit})")]
    SitemapTooLarge { size: usize, limit: usize },

    // Database errors
    #[error("Database query failed: {message}")]
    DatabaseQueryFailed { message: String },

    // HTTP errors
    #[error("HTTP request failed: {message}")]
    HttpRequestFailed { message: String },

    // Validation errors
    #[error("Invalid URL: {url}")]
    InvalidUrl { url: String },

    // ... more variants
}

impl IndexerError {
    /// Check if this error should trigger a retry
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::HttpRequestFailed { .. } => true,
            Self::GoogleApiError { status_code, .. } => {
                matches!(status_code, 500 | 502 | 503 | 504)
            }
            Self::IndexNowRateLimitExceeded => true,
            _ => false,
        }
    }
}
```

### Error Handling Strategy

1. **Propagate errors up the stack** using `?` operator
2. **Convert errors at boundaries** using `map_err`
3. **Add context** to errors when caught
4. **Retry transient errors** with exponential backoff
5. **Log errors** at appropriate levels
6. **Present user-friendly messages** in CLI

### Retry Logic

```rust
pub async fn retry_with_backoff<F, Fut, T>(
    config: RetryConfig,
    mut operation: F,
) -> Result<T, IndexerError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, IndexerError>>,
{
    let mut attempt = 0;
    let mut delay = config.initial_backoff;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt >= config.max_retries => return Err(e),
            Err(e) if !e.is_retryable() => return Err(e),
            Err(e) => {
                attempt += 1;
                warn!("Attempt {} failed: {}. Retrying in {:?}", attempt, e, delay);
                tokio::time::sleep(delay).await;
                delay = std::cmp::min(
                    delay * config.backoff_multiplier as u32,
                    config.max_backoff,
                );
            }
        }
    }
}
```

## Async/Concurrency Model

### Runtime: Tokio

All async operations use the Tokio runtime with the `multi-threaded` scheduler.

### Concurrency Patterns

**1. Concurrent Batch Processing**:
```rust
use futures::stream::{self, StreamExt};

let mut batch_stream = stream::iter(batches)
    .map(|batch| async move {
        // Process batch
    })
    .buffer_unordered(concurrent_batches);

while let Some(result) = batch_stream.next().await {
    // Handle result
}
```

**2. Multi-Endpoint Submission**:
```rust
let mut tasks = Vec::new();

for endpoint in endpoints {
    let task = tokio::spawn(async move {
        client.submit_url(url, endpoint).await
    });
    tasks.push(task);
}

// Wait for all tasks
for task in tasks {
    let result = task.await?;
    // Handle result
}
```

**3. Rate Limiting with Async**:
```rust
pub struct RateLimiter {
    max_requests_per_minute: usize,
    request_times: Vec<DateTime<Utc>>,
}

impl RateLimiter {
    pub async fn wait_if_needed(&mut self) {
        // Calculate wait time
        if self.should_wait() {
            let duration = self.calculate_wait_duration();
            tokio::time::sleep(duration).await;
        }
        self.record_request();
    }
}
```

### Thread Safety

- **Arc** for shared ownership across tasks
- **Mutex** for synchronized access to shared state
- **RwLock** for read-heavy workloads (not currently used)

Example:
```rust
pub struct BatchSubmitter {
    google_client: Option<Arc<GoogleIndexingClient>>,
    history_manager: Arc<HistoryManager>,
    // ...
}
```

## Configuration System

### Configuration Precedence

1. **Command-line arguments** (highest priority)
2. **Environment variables**
3. **Project config file** (`./.indexer-cli/config.yaml`)
4. **Global config file** (`~/.indexer-cli/config.yaml`)
5. **Default values** (lowest priority)

### Configuration Loading

```rust
pub fn load_config(path: Option<PathBuf>) -> Result<Settings, IndexerError> {
    let mut builder = Config::builder();

    // Add default values
    builder = builder.add_source(Config::try_from(&Settings::default())?);

    // Add global config
    if let Ok(global_path) = global_config_path() {
        if global_path.exists() {
            builder = builder.add_source(File::from(global_path));
        }
    }

    // Add project config
    if let Ok(project_path) = project_config_path() {
        if project_path.exists() {
            builder = builder.add_source(File::from(project_path));
        }
    }

    // Add custom config if specified
    if let Some(path) = path {
        builder = builder.add_source(File::from(path));
    }

    // Add environment variables
    builder = builder.add_source(
        Environment::with_prefix("INDEXER").separator("__")
    );

    // Build and deserialize
    let config = builder.build()?;
    let settings: Settings = config.try_deserialize()?;

    Ok(settings)
}
```

## Testing Strategy

### Unit Tests

- Located in module files using `#[cfg(test)]`
- Test individual functions and methods
- Use mocks where appropriate
- Coverage goal: >80%

### Integration Tests

- Located in `tests/` directory
- Test complete workflows
- Use real HTTP clients with mock servers (wiremock)
- Test CLI commands end-to-end

### Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Helper functions
    fn create_test_client() -> TestClient {
        // ...
    }

    // Unit tests
    #[test]
    fn test_function_name() {
        // ...
    }

    #[tokio::test]
    async fn test_async_function() {
        // ...
    }
}
```

## Performance Considerations

### Optimizations

1. **Concurrent Processing**: Use tokio for async I/O and concurrent requests
2. **Connection Pooling**: Reuse HTTP connections via reqwest
3. **Streaming**: Process large sitemaps without loading entirely into memory
4. **Indexing**: Database indexes on frequently queried columns
5. **Batch Operations**: Group operations to reduce overhead
6. **Efficient Parsing**: Use fast XML parser (roxmltree)

### Benchmarks

Benchmarks can be added in `benches/` directory:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn sitemap_parsing_benchmark(c: &mut Criterion) {
    c.bench_function("parse sitemap 1000 urls", |b| {
        b.iter(|| {
            // Benchmark code
        });
    });
}
```

## Security Considerations

### Credentials Management

- **Never hardcode credentials** in source code
- Store Google service account keys securely
- Use environment variables for sensitive data
- Validate all user input

### Network Security

- **HTTPS only** for API communications
- Use `rustls` instead of OpenSSL for TLS
- Validate SSL certificates
- Set appropriate timeouts

### Input Validation

- Validate all URLs before submission
- Sanitize file paths
- Limit sitemap size and URL count
- Validate API keys and tokens

### SQL Injection Prevention

- Use parameterized queries exclusively
- Never construct SQL with string concatenation
- Validate database inputs

## Design Decisions

### Why Rust?

- **Performance**: Comparable to C/C++, no garbage collection
- **Safety**: Memory safety without runtime overhead
- **Concurrency**: Excellent async support with tokio
- **Reliability**: Strong type system catches errors at compile time
- **Ecosystem**: Great libraries for CLI, HTTP, databases

### Why SQLite?

- **Embedded**: No separate database server needed
- **Reliable**: ACID compliant, battle-tested
- **Fast**: Efficient for read-heavy workloads
- **Portable**: Single file, easy to backup

### Why clap?

- **Derive API**: Clean, declarative command definitions
- **Auto-generated help**: Automatic help text and error messages
- **Validation**: Built-in validation and parsing
- **Completions**: Can generate shell completions

### Why Tokio?

- **Mature**: Industry-standard async runtime
- **Performance**: Highly optimized
- **Ecosystem**: Most async libraries built on tokio
- **Features**: Timers, I/O, task scheduling

### Why reqwest?

- **High-level**: Easy-to-use HTTP client
- **Async**: Native async/await support
- **Features**: Connection pooling, compression, redirects
- **Reliable**: Well-maintained, widely used

---

This architecture is designed to be maintainable, extensible, and production-ready. For questions or suggestions, please open an issue or discussion on GitHub.
