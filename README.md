# indexer-cli

[English](README.md) | [简体中文](README_zh.md)

> A production-ready CLI tool for automating website indexing workflows with Google Indexing API and IndexNow

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Build Status](https://img.shields.io/github/actions/workflow/status/your-username/indexer-cli/ci.yml?branch=master)](https://github.com/your-username/indexer-cli/actions)
[![Crates.io](https://img.shields.io/crates/v/indexer-cli.svg)](https://crates.io/crates/indexer-cli)
[![Downloads](https://img.shields.io/crates/d/indexer-cli.svg)](https://crates.io/crates/indexer-cli)

**indexer-cli** is a powerful command-line tool that automates the process of submitting URLs to search engines. It seamlessly integrates with Google Indexing API and IndexNow protocol to help you get your content indexed faster and more efficiently.

## Features

- **Google Indexing API Integration**
  - Google service account authentication (JSON key)
  - URL submission with UPDATE/DELETE actions
  - Metadata retrieval and status checking
  - Intelligent rate limiting and quota management
  - Exponential backoff retry logic

- **IndexNow API Support**
  - Submit to multiple search engines (Bing, Yandex, Seznam, Naver)
  - Batch submission up to 10,000 URLs
  - API key generation and verification
  - Key file hosting validation

- **Sitemap Processing**
  - Parse XML sitemaps and sitemap indexes
  - Recursive sitemap index traversal
  - Support for gzip-compressed sitemaps
  - URL filtering by pattern, date, and priority
  - Automatic URL extraction and deduplication

- **Submission History Tracking**
  - SQLite database for persistent storage
  - Prevent duplicate submissions
  - Query and export submission history
  - Statistics and reporting

- **Advanced Features**
  - Concurrent batch processing with progress bars
  - Configurable retry strategies
  - URL validation and filtering
  - Watch mode for continuous monitoring
  - Comprehensive logging with rotation
  - Dry-run mode for testing

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Configuration](#configuration)
- [Usage](#usage)
  - [Initialize Configuration](#initialize-configuration)
  - [Google Indexing API](#google-indexing-api)
  - [IndexNow API](#indexnow-api)
  - [Sitemap Operations](#sitemap-operations)
  - [Submission History](#submission-history)
  - [Watch Mode](#watch-mode)
- [Google Setup Guide](#google-setup-guide)
- [IndexNow Setup Guide](#indexnow-setup-guide)
- [Advanced Usage](#advanced-usage)
- [Architecture](#architecture)
- [Development](#development)
- [Troubleshooting](#troubleshooting)
- [FAQ](#faq)
- [Comparison](#comparison)
- [License](#license)

## Installation

### Prerequisites

- **Rust 1.70 or higher** - Install from [rustup.rs](https://rustup.rs/)
- **SQLite 3** - Usually pre-installed on most systems

### From Source

Clone the repository and build:

```bash
git clone https://github.com/your-username/indexer-cli.git
cd indexer-cli
cargo build --release
```

The binary will be available at `target/release/indexer-cli`.

### Install Globally

To install the binary to your system:

```bash
cargo install --path .
```

### From crates.io (Coming Soon)

```bash
cargo install indexer-cli
```

## Quick Start

### 1. Initialize Configuration

Create a configuration file with the interactive wizard:

```bash
indexer-cli init
```

This creates `~/.indexer-cli/config.yaml` with default settings.

### 2. Configure Google Service Account

Set up Google Indexing API credentials:

```bash
indexer-cli google setup --service-account /path/to/service-account.json
```

See the [Google Setup Guide](#google-setup-guide) for detailed instructions.

### 3. Configure IndexNow API Key

Generate and configure an IndexNow API key:

```bash
# Generate a new key
indexer-cli indexnow generate-key --length 32 --save

# Or set an existing key
indexer-cli indexnow setup --key your-api-key-here
```

See the [IndexNow Setup Guide](#indexnow-setup-guide) for detailed instructions.

### 4. Submit Your First URL

Submit a URL to all configured APIs:

```bash
indexer-cli submit https://your-site.com/your-page
```

Or submit to a specific API:

```bash
# Google only
indexer-cli google submit https://your-site.com/your-page

# IndexNow only
indexer-cli indexnow submit https://your-site.com/your-page
```

### 5. Submit from Sitemap

Extract URLs from a sitemap and submit them:

```bash
indexer-cli sitemap parse https://your-site.com/sitemap.xml | \
  indexer-cli submit --file -
```

## Configuration

### Configuration File Location

The configuration file is located at:
- **Global**: `~/.indexer-cli/config.yaml`
- **Project**: `./.indexer-cli/config.yaml` (overrides global)

You can specify a custom location with the `--config` flag:

```bash
indexer-cli --config /path/to/config.yaml <command>
```

### Configuration Format

The configuration file uses YAML format:

```yaml
# Google Indexing API Configuration
google:
  enabled: true
  service_account_file: ~/.indexer-cli/service-account.json
  quota:
    daily_limit: 200
    rate_limit: 380
  batch_size: 100

# IndexNow API Configuration
indexnow:
  enabled: true
  api_key: your-32-character-api-key-here
  key_location: https://your-site.com/your-api-key.txt
  endpoints:
    - https://api.indexnow.org/indexnow
    - https://www.bing.com/indexnow
    - https://yandex.com/indexnow
  batch_size: 10000

# Sitemap Configuration
sitemap:
  url: https://your-site.com/sitemap.xml
  follow_index: true
  filters:
    url_pattern: ".*"
    lastmod_after: null
    priority_min: 0.0

# History Tracking
history:
  enabled: true
  database_path: ~/.indexer-cli/history.db
  retention_days: 365

# Logging Configuration
logging:
  level: info
  file: ~/.indexer-cli/indexer.log
  max_size_mb: 10
  max_backups: 5

# Retry Configuration
retry:
  enabled: true
  max_attempts: 3
  backoff_factor: 2
  max_wait_seconds: 60

# Output Configuration
output:
  format: text
  color: true
  verbose: false
```

### Environment Variables

Configuration can also be set via environment variables:

```bash
export INDEXER_CONFIG=/path/to/config.yaml
export INDEXER_GOOGLE_SERVICE_ACCOUNT=/path/to/service-account.json
export INDEXER_INDEXNOW_API_KEY=your-api-key
```

## Usage

### Initialize Configuration

Create a new configuration file:

```bash
# Interactive wizard (default)
indexer-cli init

# Create global configuration
indexer-cli init --global

# Overwrite existing configuration
indexer-cli init --force

# Non-interactive with defaults
indexer-cli init --non-interactive
```

### Google Indexing API

#### Setup

Configure Google service account credentials:

```bash
# Set up service account
indexer-cli google setup --service-account /path/to/service-account.json

# Save to global configuration
indexer-cli google setup --service-account /path/to/service-account.json --global

# Verify configuration
indexer-cli google verify
```

#### Submit URLs

Submit one or more URLs:

```bash
# Single URL
indexer-cli google submit https://your-site.com/page1

# Multiple URLs
indexer-cli google submit https://your-site.com/page1 https://your-site.com/page2

# From file (one URL per line)
indexer-cli google submit --file urls.txt

# From sitemap
indexer-cli google submit --sitemap https://your-site.com/sitemap.xml

# With DELETE action
indexer-cli google submit https://your-site.com/old-page --action url-deleted

# With filters
indexer-cli google submit --sitemap https://your-site.com/sitemap.xml \
  --filter "^https://your-site.com/blog/" \
  --since 2024-01-01

# Dry run (don't actually submit)
indexer-cli google submit https://your-site.com/page1 --dry-run
```

#### Check Status

Check the indexing status of URLs:

```bash
# Check single URL
indexer-cli google status https://your-site.com/page1

# Check multiple URLs
indexer-cli google status --file urls.txt

# Output as JSON
indexer-cli google status https://your-site.com/page1 --format json
```

#### Quota Management

View your API quota usage:

```bash
indexer-cli google quota
```

### IndexNow API

#### Setup

Configure IndexNow API key:

```bash
# Generate a new key
indexer-cli indexnow generate-key --length 32

# Generate and save to configuration
indexer-cli indexnow generate-key --length 32 --save

# Generate and output key file
indexer-cli indexnow generate-key --length 32 --output /var/www/html/

# Set an existing key
indexer-cli indexnow setup --key your-api-key-here \
  --key-location https://your-site.com/your-api-key.txt

# Verify key file is accessible
indexer-cli indexnow verify
```

#### Submit URLs

Submit URLs to IndexNow:

```bash
# Single URL
indexer-cli indexnow submit https://your-site.com/page1

# Multiple URLs
indexer-cli indexnow submit https://your-site.com/page1 https://your-site.com/page2

# From file
indexer-cli indexnow submit --file urls.txt

# From sitemap
indexer-cli indexnow submit --sitemap https://your-site.com/sitemap.xml

# To specific endpoint only
indexer-cli indexnow submit https://your-site.com/page1 --endpoint bing

# With filters
indexer-cli indexnow submit --sitemap https://your-site.com/sitemap.xml \
  --filter "^https://your-site.com/products/" \
  --since 2024-01-01

# Batch size control
indexer-cli indexnow submit --file urls.txt --batch-size 1000
```

### Unified Submit Command

Submit to all configured APIs at once:

```bash
# Submit to all APIs
indexer-cli submit https://your-site.com/page1

# From file
indexer-cli submit --file urls.txt

# From sitemap
indexer-cli submit --sitemap https://your-site.com/sitemap.xml

# To specific API
indexer-cli submit https://your-site.com/page1 --api google
indexer-cli submit https://your-site.com/page1 --api indexnow

# With options
indexer-cli submit --sitemap https://your-site.com/sitemap.xml \
  --filter "^https://your-site.com/" \
  --since 2024-01-01 \
  --batch-size 50 \
  --format json
```

### Sitemap Operations

#### Parse Sitemap

Parse and display sitemap contents:

```bash
# Parse sitemap
indexer-cli sitemap parse https://your-site.com/sitemap.xml

# Follow sitemap indexes
indexer-cli sitemap parse https://your-site.com/sitemap.xml --follow-index

# Output as JSON
indexer-cli sitemap parse https://your-site.com/sitemap.xml --format json
```

#### List URLs

List all URLs from a sitemap:

```bash
# List all URLs
indexer-cli sitemap list https://your-site.com/sitemap.xml

# With filter
indexer-cli sitemap list https://your-site.com/sitemap.xml \
  --filter "^https://your-site.com/blog/"

# Modified after date
indexer-cli sitemap list https://your-site.com/sitemap.xml \
  --since 2024-01-01

# Limit results
indexer-cli sitemap list https://your-site.com/sitemap.xml --limit 100
```

#### Export URLs

Export sitemap URLs to a file:

```bash
# Export to text file
indexer-cli sitemap export https://your-site.com/sitemap.xml --output urls.txt

# With filters
indexer-cli sitemap export https://your-site.com/sitemap.xml \
  --output urls.txt \
  --filter "^https://your-site.com/products/" \
  --since 2024-01-01
```

#### Sitemap Statistics

Show sitemap statistics:

```bash
# Display stats
indexer-cli sitemap stats https://your-site.com/sitemap.xml

# Output as JSON
indexer-cli sitemap stats https://your-site.com/sitemap.xml --format json
```

#### Validate Sitemap

Validate sitemap format and structure:

```bash
indexer-cli sitemap validate https://your-site.com/sitemap.xml
```

### Submission History

#### List History

View recent submission history:

```bash
# List last 20 submissions
indexer-cli history list

# List last 50 submissions
indexer-cli history list --limit 50

# Output as JSON
indexer-cli history list --format json
```

#### Search History

Search submission history with filters:

```bash
# Search by URL pattern
indexer-cli history search --url "example.com/blog"

# Search by API
indexer-cli history search --api google

# Search by status
indexer-cli history search --status success

# Search by date range
indexer-cli history search --since 2024-01-01 --until 2024-01-31

# Combine filters
indexer-cli history search \
  --url "example.com" \
  --api indexnow \
  --status success \
  --since 2024-01-01 \
  --limit 100
```

#### History Statistics

View submission statistics:

```bash
# Overall stats
indexer-cli history stats

# Stats for date range
indexer-cli history stats --since 2024-01-01 --until 2024-01-31

# Output as JSON
indexer-cli history stats --format json
```

#### Export History

Export submission history:

```bash
# Export as CSV
indexer-cli history export --output history.csv --format csv

# Export as JSON
indexer-cli history export --output history.json --format json

# Export date range
indexer-cli history export --output history.csv \
  --since 2024-01-01 --until 2024-01-31
```

#### Clean History

Clean old history records:

```bash
# Delete records older than 90 days
indexer-cli history clean --older-than 90

# Delete all records
indexer-cli history clean --all

# Skip confirmation prompt
indexer-cli history clean --older-than 90 --yes
```

### Watch Mode

Continuously monitor a sitemap for changes and auto-submit new URLs:

```bash
# Watch sitemap (check every hour)
indexer-cli watch --sitemap https://your-site.com/sitemap.xml

# Custom check interval (in seconds)
indexer-cli watch --sitemap https://your-site.com/sitemap.xml --interval 1800

# Submit to specific API
indexer-cli watch --sitemap https://your-site.com/sitemap.xml --api google

# Run as daemon
indexer-cli watch --sitemap https://your-site.com/sitemap.xml --daemon

# With PID file
indexer-cli watch --sitemap https://your-site.com/sitemap.xml \
  --daemon --pid-file /var/run/indexer-cli.pid
```

### Configuration Management

Manage configuration settings:

```bash
# List all settings
indexer-cli config list

# Get a specific setting
indexer-cli config get google.enabled

# Set a setting
indexer-cli config set google.enabled true

# Set in global configuration
indexer-cli config set google.batch_size 50 --global

# Validate configuration
indexer-cli config validate

# Show configuration file path
indexer-cli config path
```

### Validation

Validate your configuration and setup:

```bash
# Validate everything
indexer-cli validate

# Validate Google configuration only
indexer-cli validate google

# Validate IndexNow configuration only
indexer-cli validate indexnow

# Check IndexNow key file accessibility
indexer-cli validate --check-key-file

# Output as JSON
indexer-cli validate --format json
```

> `--check-key-file` now downloads the configured `indexnow.key_location` from your site and compares its contents with your API key, making it easy to confirm the target domain actually exposes the IndexNow key file.

## Google Setup Guide

indexer-cli authenticates with Google using a **service account JSON key**. If you're migrating from an older version that used `indexer-cli google auth`, you can delete those OAuth tokens—the CLI no longer needs them.

### Prerequisites

1. A Google Cloud Platform (GCP) project
2. The website verified in Google Search Console
3. The service account email added as an **Owner** on that Search Console property

---

## Service Account Authentication (Required)

Service accounts work great for local use, servers, and CI/CD since they rely on a downloaded JSON key. The same account is used for every command (verify, submit, status, etc.).

### Step 1: Create a GCP Project

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select an existing one
3. Note your project ID

### Step 2: Enable the Indexing API

1. Navigate to **APIs & Services** > **Library**
2. Search for "Web Search Indexing API" or "Indexing API"
3. Click **Enable**

### Step 3: Create a Service Account

1. Go to **APIs & Services** > **Credentials**
2. Click **Create Credentials** > **Service Account**
3. Enter a name (e.g., "indexer-cli-service")
4. Click **Create and Continue**
5. Skip the optional steps and click **Done**

### Step 4: Generate Service Account Key

1. In the service accounts list, click on the account you just created
2. Go to the **Keys** tab
3. Click **Add Key** > **Create new key**
4. Select **JSON** format
5. Click **Create** - the key file will be downloaded
6. Save the file securely (e.g., `~/.indexer-cli/service-account.json`)

### Step 5: Grant Search Console Access

1. Go to [Google Search Console](https://search.google.com/search-console/)
2. Select your property
3. Go to **Settings** > **Users and permissions**
4. Click **Add user**
5. Enter the service account email (format: `name@project-id.iam.gserviceaccount.com`)
6. Select **Owner** permission level
7. Click **Add**

### Step 6: Configure indexer-cli

```bash
indexer-cli google setup --service-account ~/.indexer-cli/service-account.json
indexer-cli google verify
```


### Quota Limits

- **Daily publish limit**: 200 URL notifications per day
- **Rate limit**: 380 requests per minute (total)
- **Metadata rate limit**: 180 requests per minute

The tool automatically respects these limits with rate limiting and quota tracking.

### Best Practices

- Only submit URLs you own and have verified in Search Console
- Use the UPDATE action for new or modified pages
- Use the DELETE action for removed pages
- Don't resubmit URLs unnecessarily (use history tracking)
- Monitor your quota usage regularly

## IndexNow Setup Guide

### What is IndexNow?

IndexNow is an open protocol that allows website owners to instantly notify search engines about the latest content changes. Supported search engines include:

- Microsoft Bing
- Yandex
- Seznam.cz
- Naver

### Step 1: Generate an API Key

Generate a new API key (32 characters recommended):

```bash
indexer-cli indexnow generate-key --length 32
```

Example output:
```
Generated IndexNow API key: 3f8a9c2b1d7e6f5a4c3b2a1d7e6f5a4c
```

### Step 2: Create Key File

Create a text file with your API key and host it at the root of your website:

1. Create file: `your-api-key.txt`
2. Content: exactly your API key (no extra spaces or newlines)
3. Upload to: `https://yourdomain.com/your-api-key.txt`

Example:
```bash
echo -n "3f8a9c2b1d7e6f5a4c3b2a1d7e6f5a4c" > 3f8a9c2b1d7e6f5a4c3b2a1d7e6f5a4c.txt
# Upload this file to your web server's document root
```

### Step 3: Configure indexer-cli

```bash
indexer-cli indexnow setup \
  --key 3f8a9c2b1d7e6f5a4c3b2a1d7e6f5a4c \
  --key-location https://yourdomain.com/3f8a9c2b1d7e6f5a4c3b2a1d7e6f5a4c.txt
```

### Step 4: Verify Setup

Verify that the key file is accessible:

```bash
indexer-cli indexnow verify
```

### Key Requirements

- **Length**: 8-128 characters (32 recommended)
- **Characters**: Only alphanumeric (a-z, A-Z, 0-9) and hyphens (-)
- **File location**: Must be accessible via HTTPS at domain root
- **File content**: Must exactly match the API key

### Best Practices

- Keep your API key secret but the key file public
- Use HTTPS for the key file location
- Submit batches of URLs when possible (up to 10,000)
- Include all URLs from the same host in one request
- Don't submit the same URL repeatedly within a short time

### Supported Endpoints

The tool submits to multiple endpoints simultaneously:

- `https://api.indexnow.org/indexnow` - Main endpoint
- `https://www.bing.com/indexnow` - Bing direct
- `https://yandex.com/indexnow` - Yandex direct

You can submit to a specific endpoint using:

```bash
indexer-cli indexnow submit URL --endpoint bing
```

## Advanced Usage

### Batch Processing

Process large numbers of URLs efficiently:

```bash
# From sitemap with custom batch size
indexer-cli submit --sitemap https://your-site.com/sitemap.xml \
  --batch-size 100

# Multiple sitemaps
cat sitemap-urls.txt | while read sitemap; do
  indexer-cli submit --sitemap "$sitemap"
done
```

### URL Filtering

Filter URLs using regex patterns:

```bash
# Submit only blog posts
indexer-cli submit --sitemap https://your-site.com/sitemap.xml \
  --filter "^https://your-site.com/blog/\d{4}/\d{2}/"

# Exclude certain patterns
indexer-cli submit --sitemap https://your-site.com/sitemap.xml \
  --filter "^https://your-site.com/(?!admin|private)"

# Date-based filtering
indexer-cli submit --sitemap https://your-site.com/sitemap.xml \
  --since 2024-01-01
```

### Force Submissions

By default, `indexer-cli` skips URLs that were submitted within the last 24 hours based on the local history database. Use `--force` (or `-F`) when you need to bypass that throttle for an urgent re-submission while still recording the attempt:

```bash
# Force a unified submission
indexer-cli submit --sitemap https://your-site.com/sitemap.xml --force

# Force IndexNow-only submission
indexer-cli indexnow submit https://your-site.com/page1 --force

# Force Google-only submission
indexer-cli google submit https://your-site.com/page1 --force
```

### Custom Retry Strategies

Configure retry behavior in `config.yaml`:

```yaml
retry:
  enabled: true
  max_attempts: 5        # Try up to 5 times
  backoff_factor: 2      # Double delay each time
  max_wait_seconds: 120  # Max 2 minutes between retries
```

### Logging Configuration

Control logging output:

```bash
# Verbose output
indexer-cli --verbose submit https://your-site.com/page1

# Quiet mode (errors only)
indexer-cli --quiet submit https://your-site.com/page1

# Debug logging in config
logging:
  level: debug
  file: ~/.indexer-cli/debug.log
```

### Database Management

The history database is stored at `~/.indexer-cli/history.db` (by default).

#### Backup Database

```bash
cp ~/.indexer-cli/history.db ~/.indexer-cli/history-backup.db
```

#### Export All History

```bash
indexer-cli history export --output all-history.csv
```

#### Clean Old Records

```bash
# Remove records older than 180 days
indexer-cli history clean --older-than 180 --yes
```

### Cron Jobs

Set up automated submissions:

```bash
# Add to crontab (crontab -e)

# Submit sitemap daily at 3 AM
0 3 * * * /usr/local/bin/indexer-cli submit --sitemap https://your-site.com/sitemap.xml

# Clean history monthly
0 0 1 * * /usr/local/bin/indexer-cli history clean --older-than 180 --yes
```

### CI/CD Integration

Use in CI/CD pipelines:

```yaml
# GitHub Actions example
- name: Submit URLs to search engines
  run: |
    indexer-cli submit --file changed-urls.txt --format json
```

### Dry Run Mode

Test without actually submitting:

```bash
# See what would be submitted
indexer-cli submit --sitemap https://your-site.com/sitemap.xml --dry-run
```

## Architecture

### High-Level Overview

```
┌─────────────────┐
│   CLI Layer     │  (args.rs, handler.rs)
│   clap-based    │
└────────┬────────┘
         │
┌────────▼──────────────────────────────────────┐
│           Commands Layer                       │
│  (init, config, google, indexnow, submit...)  │
└────────┬───────────────────────┬───────────────┘
         │                       │
┌────────▼─────────┐    ┌────────▼────────────┐
│  Services Layer  │    │   Database Layer    │
│  - Batch Submit  │    │   - Schema         │
│  - Sitemap Parse │    │   - Queries        │
│  - History Mgmt  │    │   - Models         │
│  - URL Processor │    │                    │
└────────┬─────────┘    └────────────────────┘
         │
┌────────▼─────────────────────┐
│       API Clients            │
│  - Google Indexing (Service Account) │
│  - IndexNow (HTTP)           │
└──────────────────────────────┘
```

### Module Organization

- **cli/**: Command-line interface and argument parsing
- **commands/**: Command implementations
- **api/**: External API client implementations
- **services/**: Business logic and orchestration
- **database/**: SQLite schema, models, and queries
- **config/**: Configuration loading and validation
- **types/**: Shared types and error definitions
- **utils/**: Helper utilities (retry, logging, validation)
- **constants.rs**: Application-wide constants

### Data Flow

1. **User Input** → CLI argument parsing
2. **Configuration Loading** → Merge config file + environment + defaults
3. **API Client Initialization** → Service account or API key authentication
4. **URL Collection** → From args, file, or sitemap
5. **History Check** → Filter out recently submitted URLs
6. **Batch Processing** → Split into batches, concurrent submission
7. **Result Recording** → Save to SQLite database
8. **Output Formatting** → Display results (text, JSON, CSV)

### Error Handling

The project uses a custom error type (`IndexerError`) with variants for:
- Configuration errors
- API errors (Google, IndexNow)
- Database errors
- HTTP errors
- Validation errors

Errors support:
- Retry detection (`.is_retryable()`)
- Detailed context and error chaining
- User-friendly error messages

### Async/Concurrency Model

- Built on **tokio** async runtime
- Concurrent batch processing using **futures** streams
- Rate limiting using token bucket algorithm
- Configurable concurrency levels

### Database Schema

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
    metadata TEXT
);

-- Indexes for efficient queries
CREATE INDEX idx_url ON submission_history(url);
CREATE INDEX idx_api ON submission_history(api);
CREATE INDEX idx_status ON submission_history(status);
CREATE INDEX idx_submitted_at ON submission_history(submitted_at);
```

See [docs/ARCHITECTURE.md](/Users/danielhu/Projects/indexer-cli/docs/ARCHITECTURE.md) for more details.

## Development

### Setup Development Environment

```bash
# Clone repository
git clone https://github.com/your-username/indexer-cli.git
cd indexer-cli

# Install dependencies
cargo build

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run -- submit https://your-site.com
```

### Project Structure

```
indexer-cli/
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # Library exports
│   ├── cli/                 # CLI interface
│   ├── commands/            # Command implementations
│   ├── api/                 # API clients
│   ├── services/            # Business logic
│   ├── database/            # Database layer
│   ├── config/              # Configuration
│   ├── types/               # Type definitions
│   ├── utils/               # Utilities
│   └── constants.rs         # Constants
├── tests/                   # Integration tests
├── examples/                # Usage examples
└── docs/                    # Documentation
```

### Running Tests

```bash
# All tests
cargo test

# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test '*'

# Specific test
cargo test test_sitemap_parser

# With output
cargo test -- --nocapture
```

### Building Documentation

```bash
# Build API documentation
cargo doc --no-deps --open

# Build with all features
cargo doc --all-features --open
```

### Code Style

The project follows standard Rust conventions:

- Use `rustfmt` for formatting: `cargo fmt`
- Use `clippy` for linting: `cargo clippy`
- Run checks before committing: `cargo fmt && cargo clippy && cargo test`

### Contributing

See [CONTRIBUTING.md](/Users/danielhu/Projects/indexer-cli/CONTRIBUTING.md) for detailed contribution guidelines.

## Troubleshooting

### Common Issues

#### "Configuration file not found"

Create a configuration file:
```bash
indexer-cli init
```

#### "Google service account not found"

Set up your Google credentials:
```bash
indexer-cli google setup --service-account /path/to/service-account.json
```

#### "IndexNow key file not accessible"

Verify your key file is publicly accessible:
```bash
curl https://yourdomain.com/your-api-key.txt
```

#### "Rate limit exceeded"

The tool respects API rate limits automatically. If you see this error:
- Wait for the rate limit window to reset
- Reduce batch size in configuration
- Check your quota usage: `indexer-cli google quota`

#### "Permission denied" (Google API)

Ensure your service account has Owner permission in Google Search Console for the domain.

#### "Database locked"

Another instance may be running. Close other instances or wait for operations to complete.

### Debug Mode

Enable verbose logging:

```bash
# Verbose CLI output
indexer-cli --verbose submit URL

# Debug logging
RUST_LOG=debug indexer-cli submit URL

# Trace logging (very detailed)
RUST_LOG=trace indexer-cli submit URL
```

### Log Files

Check log files for detailed information:

```bash
# Default log location
tail -f ~/.indexer-cli/indexer.log

# View recent errors
grep ERROR ~/.indexer-cli/indexer.log
```

### Verify Configuration

```bash
# Check configuration syntax
indexer-cli config validate

# Show current configuration
indexer-cli config list

# Test API connections
indexer-cli validate
```

### Getting Help

If you encounter issues not covered here:

1. Check the [documentation](docs/)
2. Search [existing issues](https://github.com/your-username/indexer-cli/issues)
3. Open a [new issue](https://github.com/your-username/indexer-cli/issues/new) with:
   - Command you ran
   - Error message
   - Configuration (sanitized)
   - Debug log output

## FAQ

### General Questions

**Q: What is the difference between Google Indexing API and IndexNow?**

A: Google Indexing API is specifically for Google Search and requires a Google Cloud service account JSON key. It has strict quota limits (200 URLs/day) but provides detailed status information. IndexNow is an open protocol supported by multiple search engines (Bing, Yandex, Seznam, Naver) with higher limits (10,000 URLs/batch) but requires only an API key.

**Q: Can I use both APIs together?**

A: Yes! The `submit` command submits to all configured APIs simultaneously. You can also use API-specific commands (`google submit`, `indexnow submit`) to target individual services.

**Q: How often should I submit my URLs?**

A: Only submit URLs when they are new or significantly updated. The tool tracks submission history to prevent duplicate submissions. For regular updates, use watch mode or schedule submissions via cron.

**Q: Is there a cost to use these APIs?**

A: Both Google Indexing API and IndexNow are free to use. However, Google has quota limits (200 URLs/day for indexing API).

**Q: What types of URLs can I submit to Google Indexing API?**

A: Google Indexing API is primarily designed for job posting and livestream video content. For general website URLs, use Google Search Console's sitemap submission instead. IndexNow supports all types of content.

### Technical Questions

**Q: Where is my data stored?**

A: All data is stored locally in SQLite databases at `~/.indexer-cli/` (by default). Configuration is stored in `config.yaml`, submission history in `history.db`, and logs in `indexer.log`.

**Q: Can I run this tool on a server?**

A: Yes! The tool is designed for both local and server use. You can run it in watch mode with `--daemon` flag or schedule it via cron jobs. Make sure to set up proper logging and monitoring.

**Q: How do I back up my submission history?**

A: Export your history to CSV or JSON using `indexer-cli history export --output backup.csv`. You can also directly copy the SQLite database at `~/.indexer-cli/history.db`.

**Q: Can I submit multiple domains?**

A: Yes, but each domain requires separate configuration. For Google API, each domain must be verified in Search Console with the service account added as owner. For IndexNow, you need to host the API key file on each domain.

**Q: How does the retry mechanism work?**

A: The tool uses exponential backoff for retries. It will retry failed requests up to 3 times (configurable) with increasing delays (2x, 4x, 8x, etc.). Only transient errors are retried; permanent errors (like authentication failures) are not.

**Q: What happens if I exceed the rate limit?**

A: The tool automatically respects rate limits and will pause submissions when limits are reached. For Google API, it tracks quota usage and prevents exceeding daily limits.

**Q: Can I filter URLs before submission?**

A: Yes! Use regex patterns with `--filter`, date filters with `--since`, or combine both. The sitemap parser also supports filtering by priority and modification date.

**Q: Is the service account key secure?**

A: The service account JSON file contains credentials and should be kept secure. Set proper file permissions (`chmod 600`) and never commit it to version control. Store it in a secure location like `~/.indexer-cli/`.

### Usage Questions

**Q: How do I test without actually submitting?**

A: Use the `--dry-run` flag with any submit command to see what would be submitted without actually making API calls.

**Q: Can I submit URLs from multiple sitemaps at once?**

A: Yes, you can pipe multiple sitemaps through the tool or create a script to iterate through them:
```bash
cat sitemap-list.txt | while read sitemap; do
  indexer-cli submit --sitemap "$sitemap"
done
```

**Q: How do I monitor long-running submissions?**

A: The tool shows progress bars for batch submissions. For watch mode, check the log file at `~/.indexer-cli/indexer.log` or run with `--verbose` flag for detailed output.

**Q: What format should my URL file be in?**

A: One URL per line, plain text format. Empty lines and lines starting with `#` are ignored.

**Q: Can I customize the batch size?**

A: Yes, use `--batch-size N` flag or set it in the configuration file under `google.batch_size` or `indexnow.batch_size`.

## Comparison

### vs. Similar Tools

| Feature | indexer-cli | Google Search Console | Manual Submission | Other CLI Tools |
|---------|-------------|----------------------|-------------------|-----------------|
| **Google Indexing API** | ✅ Full support | ✅ Via UI | ❌ No | ⚠️ Limited |
| **IndexNow Protocol** | ✅ Full support | ❌ No | ✅ Manual | ⚠️ Partial |
| **Batch Submission** | ✅ Unlimited* | ⚠️ Limited | ❌ One by one | ⚠️ Limited |
| **Sitemap Parsing** | ✅ Advanced | ✅ Basic | ❌ No | ✅ Basic |
| **History Tracking** | ✅ SQLite | ❌ No | ❌ No | ❌ No |
| **Retry Logic** | ✅ Exponential backoff | ❌ No | ❌ No | ⚠️ Basic |
| **Rate Limiting** | ✅ Automatic | ✅ Enforced | ❌ No | ⚠️ Basic |
| **Progress Tracking** | ✅ Progress bars | ❌ No | ❌ No | ⚠️ Limited |
| **Watch Mode** | ✅ Continuous | ❌ No | ❌ No | ❌ No |
| **CLI Automation** | ✅ Full | ❌ No | ❌ No | ⚠️ Partial |
| **Multi-Engine** | ✅ Google + Bing + Yandex + More | ❌ Google only | ✅ Multiple | ⚠️ Limited |
| **Open Source** | ✅ MIT | ❌ No | N/A | ⚠️ Varies |
| **Cost** | ✅ Free | ✅ Free | ✅ Free | ⚠️ Varies |

*Subject to API quota limits

### Why Choose indexer-cli?

**For Developers:**
- 🚀 Fast and efficient Rust implementation
- 🔧 Fully customizable via configuration files
- 📦 Single binary with no dependencies
- 🔄 Easy integration into CI/CD pipelines
- 🐳 Docker-friendly (coming soon)

**For SEO Professionals:**
- 📊 Comprehensive submission tracking and reporting
- 🎯 Advanced filtering and targeting capabilities
- ⏰ Automated scheduling with watch mode
- 📈 Batch processing for large sites
- 🔍 Sitemap analysis and validation

**For Site Owners:**
- ✨ Simple setup with interactive wizards
- 📝 Clear documentation and examples
- 🛡️ Built-in safety features (dry-run, rate limiting)
- 💾 Local storage with privacy
- 🆓 Completely free and open source

### Use Cases

1. **New Content Publishing**: Automatically submit new blog posts or products immediately after publication
2. **Site Migrations**: Quickly notify search engines of URL changes during site restructuring
3. **E-commerce**: Submit new product pages as inventory is added
4. **News Sites**: Fast indexing for time-sensitive content
5. **Developer Blogs**: Integrate with static site generators (Hugo, Jekyll, etc.)
6. **SEO Agencies**: Manage multiple client websites efficiently
7. **CI/CD Integration**: Automatic submission as part of deployment pipeline

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

Built with these excellent Rust crates:

- [clap](https://github.com/clap-rs/clap) - Command line argument parsing
- [tokio](https://github.com/tokio-rs/tokio) - Async runtime
- [reqwest](https://github.com/seanmonstar/reqwest) - HTTP client
- [rusqlite](https://github.com/rusqlite/rusqlite) - SQLite bindings
- [yup-oauth2](https://github.com/dermesser/yup-oauth2) - Google service account authentication
- [serde](https://github.com/serde-rs/serde) - Serialization framework
- [roxmltree](https://github.com/RazrFalcon/roxmltree) - XML parsing
- [indicatif](https://github.com/console-rs/indicatif) - Progress bars
- [tracing](https://github.com/tokio-rs/tracing) - Application logging

---

**Made with ❤️ using Rust**

For questions or support, please open an issue on GitHub.
