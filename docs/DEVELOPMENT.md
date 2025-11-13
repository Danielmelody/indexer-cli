# Development Guide

Guide for developers contributing to indexer-cli.

## Table of Contents

- [Setup Development Environment](#setup-development-environment)
- [Project Structure](#project-structure)
- [Running Tests](#running-tests)
- [Building Documentation](#building-documentation)
- [Code Style](#code-style)
- [Contributing](#contributing)

## Setup Development Environment

### Prerequisites

- Rust 1.70 or higher
- Git
- SQLite 3 (for testing)

### Clone Repository

```bash
git clone https://github.com/your-username/indexer-cli.git
cd indexer-cli
```

### Build

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Check without building
cargo check
```

### Run Tests

```bash
# All tests
cargo test

# Run with output
cargo test -- --nocapture

# Specific test
cargo test test_sitemap_parser
```

### Run with Debug Logging

```bash
RUST_LOG=debug cargo run -- submit https://example.com
```

## Project Structure

```
indexer-cli/
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # Library exports
│   ├── cli/                 # CLI interface
│   │   ├── args.rs          # Command-line arguments
│   │   ├── handler.rs       # Command routing
│   │   └── ...
│   ├── commands/            # Command implementations
│   │   ├── mod.rs
│   │   ├── google.rs        # Google API commands
│   │   ├── indexnow.rs      # IndexNow commands
│   │   ├── sitemap.rs       # Sitemap operations
│   │   ├── submit.rs        # Unified submission
│   │   ├── init.rs          # Initialization
│   │   ├── config.rs        # Config management
│   │   ├── history.rs       # History management
│   │   ├── validate.rs      # Validation
│   │   └── watch.rs         # Watch mode
│   ├── api/                 # API clients
│   │   ├── mod.rs
│   │   ├── google.rs        # Google API client
│   │   └── indexnow.rs      # IndexNow API client
│   ├── services/            # Business logic
│   │   ├── mod.rs
│   │   ├── batch.rs         # Batch processing
│   │   ├── sitemap.rs       # Sitemap parsing
│   │   ├── url_processor.rs # URL processing
│   │   └── history.rs       # History tracking
│   ├── database/            # Database layer
│   │   ├── mod.rs
│   │   ├── schema.rs        # Database schema
│   │   ├── models.rs        # Data models
│   │   └── queries.rs       # Database queries
│   ├── config/              # Configuration
│   │   ├── mod.rs
│   │   ├── loader.rs        # Config loading
│   │   └── validator.rs     # Config validation
│   ├── types/               # Type definitions
│   │   ├── mod.rs
│   │   ├── errors.rs        # Error types
│   │   └── responses.rs     # Response types
│   ├── utils/               # Utilities
│   │   ├── mod.rs
│   │   ├── retry.rs         # Retry logic
│   │   ├── logging.rs       # Logging setup
│   │   └── validation.rs    # Validation helpers
│   └── constants.rs         # Application constants
├── tests/                   # Integration tests
├── examples/                # Usage examples
├── docs/                    # Documentation
├── Cargo.toml              # Dependencies and metadata
└── README.md               # Project readme
