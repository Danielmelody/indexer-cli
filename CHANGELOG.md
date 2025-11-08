# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Future enhancements will be listed here

## [0.1.0] - 2024-01-15

### Added

#### Core Features
- Complete Google Indexing API v3 integration with OAuth2 service account authentication
- Full IndexNow API protocol support for multiple search engines (Bing, Yandex, Seznam, Naver)
- XML sitemap parser with support for sitemap indexes and gzip compression
- SQLite-based submission history tracking with persistent storage
- Comprehensive CLI interface built with clap

#### Google Indexing API
- URL submission with UPDATE and DELETE notification types
- URL metadata retrieval and status checking
- Intelligent rate limiting (380 requests/minute)
- Daily quota management (200 publish requests/day)
- Automatic OAuth2 token refresh
- Exponential backoff retry logic for failed requests
- Batch processing with configurable batch size (default: 100 URLs)

#### IndexNow API
- Single and batch URL submission (up to 10,000 URLs per request)
- Multi-endpoint submission (api.indexnow.org, bing.com, yandex.com)
- API key generation with customizable length (8-128 characters)
- Key file verification and validation
- Concurrent submission to all configured endpoints
- Automatic retry on transient failures

#### Sitemap Features
- Parse regular XML sitemaps and sitemap indexes
- Recursive sitemap index traversal with depth limiting
- Automatic gzip decompression support
- URL deduplication
- Advanced filtering by:
  - URL pattern (regex)
  - Last modification date
  - Priority threshold
- Size validation (50MB limit, 50,000 URLs per sitemap)
- Export URLs to text files

#### Submission History
- SQLite database with optimized schema and indexes
- Prevent duplicate submissions within configurable time windows
- Query history with filters:
  - URL pattern
  - API type (Google, IndexNow)
  - Status (success, failed)
  - Date range
- Statistics and reporting:
  - Total submissions by API
  - Success/failure rates
  - Daily submission counts
- Export history to CSV or JSON formats
- Clean old records with configurable retention period

#### Configuration Management
- YAML-based configuration files
- Support for global (`~/.indexer-cli/config.yaml`) and project-local (`./.indexer-cli/config.yaml`) configs
- Environment variable overrides
- Interactive initialization wizard
- Configuration validation
- Get/set individual configuration values via CLI

#### CLI Commands
- `init` - Initialize configuration with interactive wizard
- `config` - Manage configuration settings (list, get, set, validate, path)
- `google` - Google Indexing API operations
  - `setup` - Configure service account
  - `submit` - Submit URLs
  - `status` - Check URL status
  - `quota` - View quota usage
  - `verify` - Verify configuration
- `indexnow` - IndexNow API operations
  - `setup` - Configure API key
  - `generate-key` - Generate new API key
  - `submit` - Submit URLs
  - `verify` - Verify key file
- `submit` - Unified submission to all configured APIs
- `sitemap` - Sitemap operations
  - `parse` - Parse and display sitemap
  - `list` - List URLs
  - `export` - Export URLs to file
  - `stats` - Show statistics
  - `validate` - Validate sitemap format
- `history` - Submission history management
  - `list` - List recent submissions
  - `search` - Search with filters
  - `stats` - View statistics
  - `export` - Export to CSV/JSON
  - `clean` - Remove old records
- `watch` - Monitor sitemap for changes and auto-submit
- `validate` - Validate configuration and setup

#### Advanced Features
- Concurrent batch processing with configurable concurrency
- Progress bars with ETA for batch operations
- Multiple output formats (text, JSON, CSV)
- Colored terminal output with `--no-color` option
- Verbose and quiet modes for logging control
- Dry-run mode for testing without submission
- Skip history checks for forced resubmission
- Custom batch sizes for all operations
- URL validation and filtering with regex
- Date-based filtering for sitemap URLs
- Comprehensive error handling with retry logic
- Request timeout configuration
- Database connection pooling
- Log file rotation with size limits

#### Developer Experience
- Comprehensive error types with context
- Detailed tracing and logging with multiple levels
- Well-documented public API
- Extensive unit and integration tests
- Example code for common use cases
- Modular architecture for easy extension

### Documentation
- Comprehensive README with usage examples
- Detailed Google Indexing API setup guide
- Detailed IndexNow API setup guide
- Troubleshooting section
- Architecture documentation (planned)
- API reference documentation (planned)
- Contributing guidelines (planned)

### Dependencies
- clap 4.5 - CLI framework
- tokio 1.47 - Async runtime
- reqwest 0.12 - HTTP client with rustls
- rusqlite 0.37 - SQLite database
- yup-oauth2 12.1 - OAuth2 authentication
- google-indexing3 6.0 - Google API client
- serde 1.0 - Serialization
- serde_yaml 0.9 - YAML support
- chrono 0.4 - Date/time handling
- tracing 0.1 - Logging framework
- indicatif 0.18 - Progress bars
- roxmltree 0.20 - XML parsing
- url 2.5 - URL parsing and validation
- regex 1.12 - Regular expressions
- anyhow 1.0 - Error handling
- thiserror 2.0 - Error derivation

### Security
- OAuth2 service account credentials stored securely
- No hardcoded credentials or API keys
- HTTPS-only for API communications
- Rustls for TLS instead of OpenSSL
- Input validation for all user-provided data
- SQL injection protection via parameterized queries

### Performance
- Concurrent request processing with tokio
- Connection pooling for HTTP requests
- SQLite WAL mode for better concurrency
- Indexed database queries
- Streaming sitemap processing
- Efficient XML parsing with roxmltree
- Minimal allocations in hot paths

### Platform Support
- Linux (tested)
- macOS (tested)
- Windows (should work, not extensively tested)

## [0.1.0-beta.1] - 2024-01-01

### Added
- Initial beta release
- Core functionality for Google Indexing API
- Basic IndexNow support
- Sitemap parsing
- SQLite history tracking

### Known Issues
- Watch mode daemon functionality not fully tested
- Windows compatibility needs more testing
- Some edge cases in sitemap parsing may not be handled

---

## Version History

- **0.1.0** (2024-01-15) - Initial production release
- **0.1.0-beta.1** (2024-01-01) - Initial beta release

## Upgrade Guide

### Upgrading to 0.1.0 from beta

No breaking changes. Configuration and database formats are compatible.

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for details on how to contribute to this project.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
