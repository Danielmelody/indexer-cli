# indexer-cli

[English](README.md) | [简体中文](README_CN.md)

> A production-ready CLI tool for automating website indexing workflows with Google Indexing API and IndexNow

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Build Status](https://img.shields.io/github/actions/workflow/status/your-username/indexer-cli/ci.yml?branch=master)](https://github.com/your-username/indexer-cli/actions)
[![Crates.io](https://img.shields.io/crates/v/indexer-cli.svg)](https://crates.io/crates/indexer-cli)
[![Downloads](https://img.shields.io/crates/d/indexer-cli.svg)](https://crates.io/crates/indexer-cli)

**indexer-cli** is a powerful command-line tool that automates URL submission to search engines. It seamlessly integrates with Google Indexing API and IndexNow protocol to help you get your content indexed faster.

## Features

- **Google Indexing API**: Service account auth, URL submission, status checking, rate limiting
- **IndexNow Protocol**: Multi-engine support (Bing, Yandex, Seznam, Naver), batch submission
- **Sitemap Processing**: Parse XML sitemaps, recursive traversal, URL filtering
- **History Tracking**: SQLite database for submission records and deduplication
- **Advanced**: Concurrent batch processing, retry logic, watch mode, dry-run testing

## Installation

### From Source

```bash
git clone https://github.com/your-username/indexer-cli.git
cd indexer-cli
cargo build --release
```

The binary will be at `target/release/indexer-cli`.

### Global Install

```bash
cargo install --path .
```

## Quick Start

### 1. Initialize Configuration

```bash
indexer-cli init
```

Creates `~/.indexer-cli/config.yaml` with default settings.

### 2. Configure APIs

**Google** (service account JSON key):
```bash
indexer-cli google setup --service-account /path/to/service-account.json
```

**IndexNow** (generate API key):
```bash
indexer-cli indexnow generate-key --length 32 --save
```

See detailed guides: [Google Setup](docs/GOOGLE_SETUP.md) | [IndexNow Setup](docs/INDEXNOW_SETUP.md)

### 3. Submit URL

```bash
# Submit to all configured APIs
indexer-cli submit https://your-site.com/page

# Submit to specific API
indexer-cli google submit https://your-site.com/page
indexer-cli indexnow submit https://your-site.com/page
```

### 4. Submit from Sitemap

```bash
indexer-cli sitemap parse https://your-site.com/sitemap.xml | \
  indexer-cli submit --file -
```

## Documentation

- [Google Setup Guide](docs/GOOGLE_SETUP.md) - Complete Google service account setup
- [IndexNow Setup Guide](docs/INDEXNOW_SETUP.md) - IndexNow API key configuration
- [Configuration](docs/CONFIGURATION.md) - Configuration file format and options
- [Usage Guide](docs/USAGE.md) - Detailed command usage examples
- [Advanced Usage](docs/ADVANCED_USAGE.md) - Batch processing, filtering, automation
- [Architecture](docs/ARCHITECTURE.md) - Technical architecture details
- [Troubleshooting](docs/TROUBLESHOOTING.md) - Common issues and solutions
- [FAQ](docs/FAQ.md) - Frequently asked questions
- [Comparison](docs/COMPARISON.md) - Comparison with other tools
- [Development](docs/DEVELOPMENT.md) - Development setup and contributing

## Configuration

Configuration file location:
- **Global**: `~/.indexer-cli/config.yaml`
- **Project**: `./.indexer-cli/config.yaml`
- **Custom**: Use `--config /path/to/config.yaml`

Basic configuration:
```yaml
google:
  enabled: true
  service_account_file: ~/.indexer-cli/service-account.json

indexnow:
  enabled: true
  api_key: your-api-key-here
  key_location: https://your-site.com/api-key.txt

history:
  enabled: true
  database_path: ~/.indexer-cli/history.db
```

See [Configuration Guide](docs/CONFIGURATION.md) for all options.

## Command Overview

```bash
# Initialize configuration
indexer-cli init

# Google API
indexer-cli google submit <url>
indexer-cli google status <url>
indexer-cli google quota

# IndexNow API
indexer-cli indexnow submit <url>
indexer-cli indexnow verify

# Unified submission
indexer-cli submit <url>
indexer-cli submit --sitemap <sitemap.xml>

# Sitemap operations
indexer-cli sitemap parse <sitemap.xml>
indexer-cli sitemap list <sitemap.xml>

# History management
indexer-cli history list
indexer-cli history stats

# Validation
indexer-cli validate

# Watch mode
indexer-cli watch --sitemap <sitemap.xml>
```

Run `indexer-cli --help` or `indexer-cli <command> --help` for details.

## Examples

### Submit Sitemap with Filtering

```bash
# Submit only blog posts from last 30 days
indexer-cli submit --sitemap https://your-site.com/sitemap.xml \
  --filter "^https://your-site.com/blog/" \
  --since $(date -d "30 days ago" +%Y-%m-%d)
```

### Automation with Cron

```bash
# Submit daily at 3 AM
0 3 * * * /usr/local/bin/indexer-cli submit --sitemap https://your-site.com/sitemap.xml

# Clean old history monthly
0 0 1 * * /usr/local/bin/indexer-cli history clean --older-than 180 --yes
```

### CI/CD Integration

```yaml
# GitHub Actions example
- name: Submit URLs to search engines
  run: |
    indexer-cli submit --file changed-urls.txt --format json
```

See [Usage Guide](docs/USAGE.md) for more examples.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
