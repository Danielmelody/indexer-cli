# Configuration Guide

This guide covers all configuration options for indexer-cli.

## Configuration File Location

Configuration is loaded from these locations (in order):

1. **Project-specific**: `./.indexer-cli/config.yaml` (overrides global)
2. **Global**: `~/.indexer-cli/config.yaml`
3. **Custom**: Use `--config /path/to/config.yaml` flag

## Configuration Format

The configuration file uses YAML format:

```yaml
# Google Indexing API Configuration
google:
  enabled: true                      # Enable/disable Google API
  service_account_file: ~/.indexer-cli/service-account.json
  quota:
    daily_limit: 200                 # Daily URL limit
    rate_limit: 380                  # Requests per minute
  batch_size: 100                    # URLs per batch

# IndexNow API Configuration
indexnow:
  enabled: true                      # Enable/disable IndexNow API
  api_key: your-32-character-key
  key_location: https://your-site.com/key.txt
  endpoints:
    - https://api.indexnow.org/indexnow
    - https://www.bing.com/indexnow
    - https://yandex.com/indexnow
  batch_size: 10000                  # URLs per batch (max 10,000)

# Sitemap Configuration
sitemap:
  url: https://your-site.com/sitemap.xml
  follow_index: true                 # Follow sitemap index files
  filters:
    url_pattern: ".*"                # Regex pattern for URLs
    lastmod_after: null              # ISO 8601 date filter
    priority_min: 0.0                # Minimum priority (0.0-1.0)

# History Tracking
history:
  enabled: true                      # Track submission history
  database_path: ~/.indexer-cli/history.db
  retention_days: 365               # Keep history for N days

# Logging Configuration
logging:
  level: info                       # debug, info, warn, error
  file: ~/.indexer-cli/indexer.log
  max_size_mb: 10                   # Rotate at this size
  max_backups: 5                    # Keep N backup files

# Retry Configuration
retry:
  enabled: true                     # Enable retries
  max_attempts: 3                   # Max retry attempts
  backoff_factor: 2                 # Delay multiplier
  max_wait_seconds: 60             # Max delay between retries

# Output Configuration
output:
  format: text                      # text, json, csv
  color: true                       # Enable colored output
  verbose: false                    # Verbose logging
```

## Environment Variables

Configuration can be set via environment variables (overrides config file):

```bash
# Config path
export INDEXER_CONFIG=/path/to/config.yaml

# Google
export INDEXER_GOOGLE_SERVICE_ACCOUNT=/path/to/service-account.json

# IndexNow
export INDEXER_INDEXNOW_API_KEY=your-api-key
export INDEXER_INDEXNOW_KEY_LOCATION=https://your-site.com/key.txt

# Database
export INDEXER_HISTORY_DATABASE=~/.indexer-cli/history.db

# Logging
export INDEXER_LOG_LEVEL=debug
export INDEXER_LOG_FILE=~/.indexer-cli/indexer.log
```

## Google Configuration

### Basic Setup

```yaml
google:
  enabled: true
  service_account_file: ~/.indexer-cli/service-account.json
```

### Quota Management

```yaml
google:
  quota:
    daily_limit: 200      # Google's daily limit
    rate_limit: 380       # Requests per minute limit
```

The tool automatically tracks quota usage. Set these to your actual limits.

### Batch Processing

```yaml
google:
  batch_size: 100        # URLs per batch (1-200 recommended)
```

Google API doesn't support true batching, but the tool processes URLs in concurrent batches for efficiency.

## IndexNow Configuration

### Basic Setup

```yaml
indexnow:
  enabled: true
  api_key: your-api-key-here
  key_location: https://your-site.com/key.txt
```

The `key_location` must be a publicly accessible URL where your API key file is hosted.

### Batch Processing

```yaml
indexnow:
  batch_size: 10000       # Up to 10,000 URLs per batch
```

IndexNow supports true batching - multiple URLs submitted in a single request.

### Custom Endpoints

```yaml
indexnow:
  endpoints:
    - https://api.indexnow.org/indexnow
    - https://www.bing.com/indexnow
    - https://yandex.com/indexnow
```

The tool submits to all configured endpoints by default. Add or remove as needed.

## Sitemap Configuration

### Basic Settings

```yaml
sitemap:
  url: https://your-site.com/sitemap.xml
  follow_index: true
```

`follow_index`: When true, recursively follows sitemap index files.

### URL Filters

```yaml
sitemap:
  filters:
    url_pattern: ".*"                    # Match all URLs
    # url_pattern: "^https://example.com/blog/"  # Only blog posts
    # url_pattern: "^https://example.com/(?!admin)"  # Exclude admin

    lastmod_after: "2024-01-01"         # Modified after date

    priority_min: 0.5                    # Minimum priority
```

Use regex patterns for flexible URL filtering.

## History Configuration

### Database Settings

```yaml
history:
  enabled: true
  database_path: ~/.indexer-cli/history.db
  retention_days: 365
```

`retention_days`: Automatically clean records older than this many days (0 = keep forever).

### Data Stored

The history database tracks:
- URL
- API (google/indexnow)
- Action (url-updated/url-deleted)
- Status (success/failed)
- Response code and message
- Timestamp
- Metadata

## Logging Configuration

### Log Levels

```yaml
logging:
  level: info    # Options: trace, debug, info, warn, error
  file: ~/.indexer-cli/indexer.log
```

Levels:
- **trace**: Very detailed, includes all HTTP requests
- **debug**: Detailed debugging information
- **info**: General information (default)
- **warn**: Warnings only
- **error**: Errors only

### Log Rotation

```yaml
logging:
  max_size_mb: 10      # Rotate at 10 MB
  max_backups: 5       # Keep 5 backups
```

Log files are rotated automatically when they exceed `max_size_mb`.

## Retry Configuration

### Retry Settings

```yaml
retry:
  enabled: true
  max_attempts: 3
  backoff_factor: 2
  max_wait_seconds: 60
```

**backoff_factor**: Delay multiplier for exponential backoff.

Example with factor 2:
1. First retry: 2 seconds
2. Second retry: 4 seconds
3. Third retry: 8 seconds

**max_wait_seconds**: Caps the maximum delay between retries.

### Retryable Errors

The tool retries these error types:
- Network timeouts
- Rate limit errors (429)
- Server errors (5xx)
- Connection errors

It does NOT retry:
- Authentication errors (401, 403)
- Client errors (4xx, except 429)
- Invalid URL errors
- Configuration errors

## Output Configuration

### Output Format

```yaml
output:
  format: text        # Options: text, json, csv
```

**text**: Human-readable output with colors and formatting.
**json**: Machine-readable JSON for scripting.
**csv**: Comma-separated values for spreadsheets.

### Display Options

```yaml
output:
  color: true         # Enable ANSI colors
  verbose: false      # Show verbose output
```

Set `color: false` for non-interactive environments or logs.

## Advanced Configuration Examples

### High-Performance Setup

```yaml
# Optimized for large sites
google:
  batch_size: 200
  quota:
    daily_limit: 200
    rate_limit: 380

indexnow:
  batch_size: 10000
  endpoints:
    - https://api.indexnow.org/indexnow

retry:
  enabled: true
  max_attempts: 5
  backoff_factor: 2
```

### Development/Debug Setup

```yaml
# Verbose logging for debugging
logging:
  level: debug
  file: ~/.indexer-cli/debug.log

output:
  format: json
  verbose: true

retry:
  enabled: true
  max_attempts: 1     # No retries for faster failure
```

### Minimal/Cron Setup

```yaml
# Optimized for automated jobs
logging:
  level: warn         # Only log warnings and errors
  file: /var/log/indexer-cli.log

output:
  format: json
  color: false        # No colors in logs

history:
  enabled: true
  retention_days: 90  # Shorter retention for cron jobs
```

### Multi-Site Configuration

Use different config files for different sites:

**site1.yaml:**
```yaml
google:
  service_account_file: ~/.indexer-cli/site1-service-account.json

indexnow:
  api_key: site1-key
  key_location: https://site1.com/key.txt
```

**site2.yaml:**
```yaml
google:
  service_account_file: ~/.indexer-cli/site2-service-account.json

indexnow:
  api_key: site2-key
  key_location: https://site2.com/key.txt
```

Usage:
```bash
indexer-cli --config site1.yaml submit https://site1.com/page
indexer-cli --config site2.yaml submit https://site2.com/page
```

## Environment-Specific Configurations

### Development

Create `.indexer-cli/config.yaml` in your project:

```yaml
google:
  enabled: true
  service_account_file: ./dev-service-account.json

logging:
  level: debug

output:
  format: text
  color: true
```

### Production

Use global config with stricter settings:

```yaml
logging:
  level: warn
  file: /var/log/indexer-cli/indexer.log
  max_size_mb: 100
  max_backups: 10

output:
  format: json
  color: false
```

## Validation

Validate your configuration:

```bash
# Validate everything
indexer-cli config validate

# Validate specific sections
indexer-cli validate google
indexer-cli validate indexnow

# Show current config
indexer-cli config list

# Show config file path
indexer-cli config path
```

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
indexer-cli config set google.batch_size 50

# Set globally
indexer-cli config set google.batch_size 50 --global
```

Changes are written to the appropriate config file.
