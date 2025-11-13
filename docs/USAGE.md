# Usage Guide

Complete guide to using indexer-cli commands.

## Table of Contents

- [Initialize Configuration](#initialize-configuration)
- [Google Indexing API](#google-indexing-api)
- [IndexNow API](#indexnow-api)
- [Unified Submit](#unified-submit)
- [Sitemap Operations](#sitemap-operations)
- [Submission History](#submission-history)
- [Watch Mode](#watch-mode)
- [Validation](#validation)
- [Configuration Management](#configuration-management)

## Initialize Configuration

### Interactive Setup

```bash
indexer-cli init
```

Runs interactive wizard to create configuration file.

### Options

```bash
# Create global configuration
indexer-cli init --global

# Overwrite existing configuration
indexer-cli init --force

# Non-interactive mode with defaults
indexer-cli init --non-interactive

# Use custom config path
indexer-cli init --config /path/to/config.yaml
```

### Generated File

Creates `~/.indexer-cli/config.yaml` with:
- Default logging settings
- Disabled APIs (to be configured later)
- History tracking enabled
- Sensible retry and batch settings

## Google Indexing API

### Setup

```bash
# Configure service account
indexer-cli google setup --service-account /path/to/service-account.json

# Save to global configuration
indexer-cli google setup --service-account /path/to/service-account.json --global

# Verify configuration
indexer-cli google verify
```

See [Google Setup Guide](GOOGLE_SETUP.md) for detailed setup instructions.

### Submit URLs

#### Single URL

```bash
indexer-cli google submit https://your-site.com/page1
```

#### Multiple URLs

```bash
indexer-cli google submit https://your-site.com/page1 https://your-site.com/page2
```

#### From File

```bash
# One URL per line
indexer-cli google submit --file urls.txt
```

#### From Sitemap

```bash
indexer-cli google submit --sitemap https://your-site.com/sitemap.xml
```

#### With DELETE Action

```bash
indexer-cli google submit https://your-site.com/old-page --action url-deleted
```

Actions:
- `url-updated`: New or modified page (default)
- `url-deleted`: Removed page

#### With Filters

```bash
indexer-cli google submit --sitemap https://your-site.com/sitemap.xml \
  --filter "^https://your-site.com/blog/" \
  --since 2024-01-01
```

Filter options:
- `--filter`: Regex pattern for URLs
- `--since`: ISO 8601 date for last modification

#### Dry Run

```bash
indexer-cli google submit https://your-site.com/page1 --dry-run
```

Shows what would be submitted without making API calls.

#### Force Submission

```bash
indexer-cli google submit https://your-site.com/page1 --force
```

Bypasses 24-hour deduplication check.

#### Batch Size

```bash
indexer-cli google submit --file urls.txt --batch-size 50
```

Controls concurrent processing batch size.

#### Output Formats

```bash
# JSON output
indexer-cli google submit https://your-site.com/page1 --format json

# CSV output for spreadsheet import
indexer-cli google submit --file urls.txt --format csv
```

### Check Status

#### Single URL

```bash
indexer-cli google status https://your-site.com/page1
```

#### Multiple URLs

```bash
indexer-cli google status --file urls.txt
```

#### Output Options

```bash
# JSON output
indexer-cli google status https://your-site.com/page1 --format json

# Verbose output
indexer-cli google status https://your-site.com/page1 --verbose
```

Status information includes:
- URL notification metadata
- Latest update time
- Type (URL_UPDATED or URL_DELETED)

### Quota Management

```bash
indexer-cli google quota
```

Displays:
- Daily quota usage
- Quota remaining
- Rate limit status

### Advanced Examples

#### Submit with Custom Config

```bash
indexer-cli --config /path/to/config.yaml google submit https://site.com/page
```

#### Submit Multiple Sitemaps

```bash
for sitemap in sitemap1.xml sitemap2.xml sitemap3.xml; do
  indexer-cli google submit --sitemap "https://your-site.com/$sitemap"
done
```

#### Submit Recent Blog Posts Only

```bash
# Get URLs modified in last 7 days
indexer-cli sitemap parse https://your-site.com/sitemap.xml \
  --filter "^https://your-site.com/blog/" \
  --since $(date -d "7 days ago" +%Y-%m-%d) \
  --format csv \
  > recent-blog-posts.csv

# Submit them
indexer-cli google submit --file recent-blog-posts.csv
```

## IndexNow API

### Setup

```bash
# Generate a new key
indexer-cli indexnow generate-key --length 32

# Generate and save to configuration
indexer-cli indexnow generate-key --length 32 --save

# Generate and output key file
indexer-cli indexnow generate-key --length 32 --output /var/www/html/

# Setup with existing key
indexer-cli indexnow setup \
  --key your-api-key-here \
  --key-location https://your-site.com/api-key.txt

# Verify key file accessibility
indexer-cli indexnow verify
```

See [IndexNow Setup Guide](INDEXNOW_SETUP.md) for detailed setup instructions.

### Submit URLs

#### Single URL

```bash
indexer-cli indexnow submit https://your-site.com/page1
```

#### Multiple URLs

```bash
indexer-cli indexnow submit https://your-site.com/page1 https://your-site.com/page2
```

#### From File

```bash
indexer-cli indexnow submit --file urls.txt
```

#### From Sitemap

```bash
indexer-cli indexnow submit --sitemap https://your-site.com/sitemap.xml
```

#### To Specific Endpoint

```bash
indexer-cli indexnow submit https://your-site.com/page1 --endpoint bing
```

Valid endpoints: `indexnow`, `bing`, `yandex`

#### With Filters

```bash
indexer-cli indexnow submit --sitemap https://your-site.com/sitemap.xml \
  --filter "^https://your-site.com/products/" \
  --since 2024-01-01
```

#### Batch Size Control

```bash
indexer-cli indexnow submit --file urls.txt --batch-size 1000
```

IndexNow supports up to 10,000 URLs per batch.

#### Dry Run

```bash
indexer-cli indexnow submit --sitemap https://your-site.com/sitemap.xml --dry-run
```

#### Force Submission

```bash
indexer-cli indexnow submit https://your-site.com/page1 --force
```

#### Output Formats

```bash
indexer-cli indexnow submit https://your-site.com/page1 --format json
indexer-cli indexnow submit --file urls.txt --format csv
```

### Verify

```bash
indexer-cli indexnow verify
```

Checks that:
1. Key file is accessible via HTTPS
2. File content matches configured API key
3. All endpoints are reachable

### Advanced Examples

#### Submit to Multi-Language Site

```bash
# Submit English pages only
indexer-cli indexnow submit --sitemap https://your-site.com/sitemap.xml \
  --filter "^https://your-site.com/en/"

# Submit German pages only
indexer-cli indexnow submit --sitemap https://your-site.com/sitemap.xml \
  --filter "^https://your-site.com/de/"
```

#### Submit Product Pages by Category

```bash
# Submit electronics category only
indexer-cli indexnow submit --sitemap https://your-site.com/sitemap.xml \
  --filter "^https://your-site.com/products/electronics/"
```

## Unified Submit

Submit to all configured APIs at once.

### Basic Usage

```bash
# Submit single URL to all APIs
indexer-cli submit https://your-site.com/page1

# Submit multiple URLs
indexer-cli submit https://your-site.com/page1 https://your-site.com/page2
```

### From File

```bash
indexer-cli submit --file urls.txt
```

### From Sitemap

```bash
indexer-cli submit --sitemap https://your-site.com/sitemap.xml
```

### To Specific API

```bash
# Google only
indexer-cli submit https://your-site.com/page1 --api google

# IndexNow only
indexer-cli submit https://your-site.com/page1 --api indexnow
```

### With Options

```bash
indexer-cli submit --sitemap https://your-site.com/sitemap.xml \
  --filter "^https://your-site.com/" \
  --since 2024-01-01 \
  --batch-size 50 \
  --format json
```

### Force All APIs

```bash
indexer-cli submit --sitemap https://your-site.com/sitemap.xml --force
```

Bypasses deduplication for all configured APIs.

### Dry Run

```bash
indexer-cli submit --sitemap https://your-site.com/sitemap.xml --dry-run
```

Shows what would be submitted to each API.

### Output Formats

```bash
indexer-cli submit --file urls.txt --format json
indexer-cli submit --file urls.txt --format csv
```

### Advanced Examples

#### Submit with Custom Batch Sizes

```bash
# Different batch sizes per API (configured in config.yaml)
cat urls.txt | indexer-cli submit --batch-size 100
```

#### Parallel Sitemap Submission

```bash
# Submit multiple sitemaps in parallel
indexer-cli submit --sitemap https://site.com/sitemap-1.xml &
indexer-cli submit --sitemap https://site.com/sitemap-2.xml &
indexer-cli submit --sitemap https://site.com/sitemap-3.xml &
wait
```

## Sitemap Operations

### Parse Sitemap

```bash
# Parse and display sitemap
indexer-cli sitemap parse https://your-site.com/sitemap.xml
```

Shows sitemap structure and metadata.

#### Follow Sitemap Index

```bash
indexer-cli sitemap parse https://your-site.com/sitemap.xml --follow-index
```

Recursively follows sitemap index files.

#### JSON Output

```bash
indexer-cli sitemap parse https://your-site.com/sitemap.xml --format json
```

Machine-readable output for scripting.

### List URLs

```bash
# List all URLs
indexer-cli sitemap list https://your-site.com/sitemap.xml
```

#### With Filter

```bash
indexer-cli sitemap list https://your-site.com/sitemap.xml \
  --filter "^https://your-site.com/blog/"
```

#### Modified After Date

```bash
indexer-cli sitemap list https://your-site.com/sitemap.xml \
  --since 2024-01-01
```

#### Limit Results

```bash
indexer-cli sitemap list https://your-site.com/sitemap.xml --limit 100
```

### Export URLs

```bash
# Export to text file
indexer-cli sitemap export https://your-site.com/sitemap.xml --output urls.txt
```

#### With Filters

```bash
indexer-cli sitemap export https://your-site.com/sitemap.xml \
  --output urls.txt \
  --filter "^https://your-site.com/products/" \
  --since 2024-01-01
```

### Sitemap Statistics

```bash
indexer-cli sitemap stats https://your-site.com/sitemap.xml
```

Shows:
- Total URLs
- Valid URLs
- Unique domains
- Last modification dates

#### JSON Output

```bash
indexer-cli sitemap stats https://your-site.com/sitemap.xml --format json
```

### Validate Sitemap

```bash
indexer-cli sitemap validate https://your-site.com/sitemap.xml
```

Checks:
- XML format validity
- Required elements
- URL format
- Encoding issues

### Advanced Examples

#### Download and Parse Sitemap

```bash
# Download sitemap
curl -s https://your-site.com/sitemap.xml -o /tmp/sitemap.xml

# Parse local file
indexer-cli sitemap parse /tmp/sitemap.xml
```

#### Extract URLs by Priority

```bash
# Parse sitemap and filter by priority
indexer-cli sitemap parse https://your-site.com/sitemap.xml \
  --format json | jq '.urls[] | select(.priority > 0.8) | .loc'
```

#### Compare Two Sitemaps

```bash
# Get URLs from both sitemaps
indexer-cli sitemap list https://site.com/sitemap-old.xml > old.txt
indexer-cli sitemap list https://site.com/sitemap-new.xml > new.txt

# Find differences
comm -13 <(sort old.txt) <(sort new.txt) > new-urls.txt

# Submit new URLs
indexer-cli submit --file new-urls.txt
```

## Submission History

### List History

```bash
# List last 20 submissions
indexer-cli history list
```

#### Limit Results

```bash
indexer-cli history list --limit 50
```

#### JSON Output

```bash
indexer-cli history list --format json
```

### Search History

```bash
# Search by URL pattern
indexer-cli history search --url "example.com/blog"
```

#### By API

```bash
indexer-cli history search --api google
indexer-cli history search --api indexnow
```

#### By Status

```bash
indexer-cli history search --status success
indexer-cli history search --status failed
```

#### By Date Range

```bash
indexer-cli history search --since 2024-01-01 --until 2024-01-31
```

#### Combined Filters

```bash
indexer-cli history search \
  --url "example.com" \
  --api indexnow \
  --status success \
  --since 2024-01-01 \
  --limit 100
```

### History Statistics

```bash
# Overall stats
indexer-cli history stats
```

Shows:
- Total submissions
- Success rate
- By API breakdown
- Recent activity

#### Date Range Stats

```bash
indexer-cli history stats --since 2024-01-01 --until 2024-01-31
```

#### JSON Output

```bash
indexer-cli history stats --format json
```

### Export History

```bash
# Export as CSV
indexer-cli history export --output history.csv --format csv

# Export as JSON
indexer-cli history export --output history.json --format json
```

#### Date Range Export

```bash
indexer-cli history export --output history.csv \
  --since 2024-01-01 --until 2024-01-31
```

### Clean History

```bash
# Delete records older than 90 days
indexer-cli history clean --older-than 90
```

#### Delete All Records

```bash
indexer-cli history clean --all
```

#### Skip Confirmation

```bash
indexer-cli history clean --older-than 90 --yes
```

## Watch Mode

Continuously monitor a sitemap for changes and auto-submit new URLs.

### Basic Usage

```bash
# Watch sitemap (check every hour)
indexer-cli watch --sitemap https://your-site.com/sitemap.xml
```

### Custom Check Interval

```bash
# Check every 30 minutes (1800 seconds)
indexer-cli watch --sitemap https://your-site.com/sitemap.xml --interval 1800
```

### Submit to Specific API

```bash
indexer-cli watch --sitemap https://your-site.com/sitemap.xml --api google
```

### Run as Daemon

```bash
# Background mode
indexer-cli watch --sitemap https://your-site.com/sitemap.xml --daemon
```

### With PID File

```bash
indexer-cli watch --sitemap https://your-site.com/sitemap.xml \
  --daemon --pid-file /var/run/indexer-cli.pid
```

### Advanced Examples

#### Watch with Filters

```bash
# Only watch blog posts
indexer-cli watch --sitemap https://your-site.com/sitemap.xml \
  --filter "^https://your-site.com/blog/"
```

#### Auto-Restart Script

```bash
#!/bin/bash
# restart-watch.sh

PIDFILE="/var/run/indexer-cli.pid"

# Kill existing process
if [ -f "$PIDFILE" ]; then
  kill $(cat "$PIDFILE") 2>/dev/null
  rm -f "$PIDFILE"
fi

# Start new watcher
indexer-cli watch \
  --sitemap https://your-site.com/sitemap.xml \
  --interval 3600 \
  --daemon \
  --pid-file "$PIDFILE"
```

#### Docker Container

```dockerfile
FROM rust:1.70

RUN cargo install indexer-cli

CMD ["indexer-cli", "watch", \
     "--sitemap", "https://your-site.com/sitemap.xml", \
     "--interval", "3600"]
```

## Validation

### Validate Everything

```bash
indexer-cli validate
```

Checks:
- Configuration syntax
- Google credentials
- IndexNow key file
- API connectivity

### Validate Specific APIs

```bash
# Google only
indexer-cli validate google

# IndexNow only
indexer-cli validate indexnow
```

### Check IndexNow Key File

```bash
indexer-cli validate --check-key-file
```

Downloads and verifies the key file from your site.

### JSON Output

```bash
indexer-cli validate --format json
```

Machine-readable validation results.

See [Troubleshooting Guide](TROUBLESHOOTING.md) for common issues.

## Configuration Management

### List Settings

```bash
indexer-cli config list
```

Shows all current settings with values.

### Get Setting

```bash
indexer-cli config get google.enabled
indexer-cli config get indexnow.batch_size
```

### Set Setting

```bash
# Set locally (project config)
indexer-cli config set google.enabled true

# Set globally
indexer-cli config set google.batch_size 50 --global
```

### Validate Configuration

```bash
indexer-cli config validate
```

### Show Config Path

```bash
indexer-cli config path
```

Shows location of current configuration file.
