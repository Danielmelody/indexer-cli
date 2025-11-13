# Contributing to indexer-cli

Thank you for your interest in contributing to indexer-cli! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Environment](#development-environment)
- [How to Contribute](#how-to-contribute)
- [Development Workflow](#development-workflow)
- [Code Style Guidelines](#code-style-guidelines)
- [Testing Requirements](#testing-requirements)
- [Documentation](#documentation)
- [Pull Request Process](#pull-request-process)
- [Issue Guidelines](#issue-guidelines)
- [Community](#community)

## Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive experience for everyone. We expect all contributors to:

- Be respectful and considerate in all interactions
- Welcome newcomers and help them learn
- Accept constructive criticism gracefully
- Focus on what is best for the community
- Show empathy towards other community members

### Unacceptable Behavior

- Harassment, discrimination, or offensive comments
- Personal attacks or trolling
- Publishing others' private information
- Other conduct that would be considered inappropriate in a professional setting

### Enforcement

If you experience or witness unacceptable behavior, please report it by opening an issue or contacting the maintainers directly.

## Getting Started

### Prerequisites

Before contributing, ensure you have:

- **Rust 1.70+** installed via [rustup](https://rustup.rs/)
- **Git** for version control
- A **GitHub account**
- Basic familiarity with Rust and async programming

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork locally:

```bash
git clone https://github.com/YOUR_USERNAME/indexer-cli.git
cd indexer-cli
```

3. Add the upstream repository:

```bash
git remote add upstream https://github.com/original-owner/indexer-cli.git
```

## Development Environment

### Setup

```bash
# Install dependencies and build
cargo build

# Run tests
cargo test

# Run clippy (linter)
cargo clippy

# Format code
cargo fmt
```

### Running Locally

```bash
# Run with cargo
cargo run -- submit https://your-site.com

# With debug logging
RUST_LOG=debug cargo run -- submit https://your-site.com

# Build and run release version
cargo build --release
./target/release/indexer-cli submit https://your-site.com
```

### Project Structure

```
indexer-cli/
├── src/
│   ├── main.rs              # Binary entry point
│   ├── lib.rs               # Library root
│   ├── cli/                 # CLI argument parsing and handling
│   │   ├── args.rs          # Command definitions
│   │   ├── handler.rs       # Command handlers
│   │   └── mod.rs
│   ├── commands/            # Command implementations
│   │   ├── init.rs
│   │   ├── google.rs
│   │   ├── indexnow.rs
│   │   ├── submit.rs
│   │   ├── sitemap.rs
│   │   ├── history.rs
│   │   └── ...
│   ├── api/                 # External API clients
│   │   ├── google_indexing.rs
│   │   ├── indexnow.rs
│   │   └── mod.rs
│   ├── services/            # Business logic
│   │   ├── batch_submitter.rs
│   │   ├── sitemap_parser.rs
│   │   ├── history_manager.rs
│   │   └── ...
│   ├── database/            # Database layer
│   │   ├── schema.rs
│   │   ├── models.rs
│   │   ├── queries.rs
│   │   └── mod.rs
│   ├── config/              # Configuration management
│   ├── types/               # Type definitions and errors
│   ├── utils/               # Utility functions
│   └── constants.rs         # Global constants
├── tests/                   # Integration tests
├── examples/                # Usage examples
├── docs/                    # Documentation
└── benches/                 # Benchmarks (optional)
```

## How to Contribute

### Types of Contributions

We welcome various types of contributions:

- **Bug fixes** - Fix issues in existing code
- **New features** - Add new functionality
- **Documentation** - Improve or add documentation
- **Tests** - Add or improve test coverage
- **Performance** - Optimize existing code
- **Examples** - Add usage examples
- **Bug reports** - Report issues you encounter
- **Feature requests** - Suggest new features

### Finding Work

- Check the [issue tracker](https://github.com/original-owner/indexer-cli/issues)
- Look for issues labeled `good first issue` or `help wanted`
- Ask in discussions if you're unsure where to start

## Development Workflow

### 1. Create a Branch

```bash
# Update your local main branch
git checkout main
git pull upstream main

# Create a feature branch
git checkout -b feature/my-new-feature

# Or for bug fixes
git checkout -b fix/bug-description
```

### Branch Naming Conventions

- `feature/` - New features (e.g., `feature/add-sitemap-compression`)
- `fix/` - Bug fixes (e.g., `fix/handle-empty-sitemap`)
- `docs/` - Documentation changes (e.g., `docs/update-readme`)
- `refactor/` - Code refactoring (e.g., `refactor/simplify-error-handling`)
- `test/` - Test additions/changes (e.g., `test/add-integration-tests`)
- `perf/` - Performance improvements (e.g., `perf/optimize-batch-processing`)

### 2. Make Changes

- Write clear, focused commits
- Follow the code style guidelines
- Add tests for new functionality
- Update documentation as needed

### 3. Test Your Changes

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run integration tests
cargo test --test '*'

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy -- -D warnings
```

### 4. Commit Changes

Write clear commit messages following this format:

```
<type>: <subject>

<body>

<footer>
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Test additions/changes
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `chore`: Maintenance tasks

Example:

```
feat: add support for sitemap compression detection

- Detect gzip compression automatically
- Handle both compressed and uncompressed sitemaps
- Add tests for compression detection

Closes #123
```

### 5. Push and Create PR

```bash
# Push your branch
git push origin feature/my-new-feature

# Create a pull request on GitHub
```

## Code Style Guidelines

### Rust Style

Follow the official [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/):

- Use `rustfmt` for automatic formatting: `cargo fmt`
- Run `clippy` and fix all warnings: `cargo clippy`
- Maximum line length: 100 characters
- Use 4 spaces for indentation (not tabs)

### Code Organization

- **One module per file** - Keep files focused and manageable
- **Public API first** - Put public items at the top of modules
- **Group related items** - Use clear module structure
- **Minimize dependencies** - Only import what you need

### Naming Conventions

- **Types**: `PascalCase` (e.g., `GoogleIndexingClient`)
- **Functions**: `snake_case` (e.g., `submit_urls`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `MAX_BATCH_SIZE`)
- **Modules**: `snake_case` (e.g., `batch_submitter`)

### Comments and Documentation

- **Public items must have doc comments** using `///`
- Include examples in doc comments where helpful
- Explain *why*, not just *what*
- Keep comments up to date with code changes

Example:

```rust
/// Submit multiple URLs to the Google Indexing API in batches.
///
/// This function splits the URLs into batches and submits them concurrently
/// while respecting rate limits and quota restrictions.
///
/// # Arguments
///
/// * `urls` - List of URLs to submit
/// * `notification_type` - Type of notification (UPDATE or DELETE)
///
/// # Returns
///
/// Returns a `BatchSubmissionResult` containing success/failure counts
/// and detailed results for each URL.
///
/// # Errors
///
/// Returns an error if:
/// - Authentication fails
/// - Rate limits are exceeded
/// - Network errors occur
///
/// # Examples
///
/// ```no_run
/// # use indexer_cli::api::google_indexing::*;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = GoogleIndexingClient::new(path)?;
/// let urls = vec!["https://your-site.com/page1".to_string()];
/// let result = client.batch_publish_urls(urls, NotificationType::UrlUpdated).await?;
/// # Ok(())
/// # }
/// ```
pub async fn batch_publish_urls(
    &self,
    urls: Vec<String>,
    notification_type: NotificationType,
) -> Result<BatchSubmissionResult, IndexerError> {
    // Implementation...
}
```

### Error Handling

- Use the `IndexerError` type for all errors
- Provide context with error messages
- Use `?` operator for error propagation
- Handle errors at appropriate levels

```rust
// Good
pub fn parse_url(url: &str) -> Result<ParsedUrl, IndexerError> {
    Url::parse(url).map_err(|e| IndexerError::InvalidUrl {
        url: url.to_string(),
    })
}

// Bad - swallowing errors
pub fn parse_url(url: &str) -> Option<ParsedUrl> {
    Url::parse(url).ok()
}
```

### Async Code

- Use `async`/`await` consistently
- Avoid blocking operations in async contexts
- Use `tokio::spawn` for independent tasks
- Handle cancellation gracefully

## Testing Requirements

### Test Coverage

All new features and bug fixes must include tests:

- **Unit tests** - Test individual functions and modules
- **Integration tests** - Test complete workflows
- **Documentation tests** - Ensure examples in docs work

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // Arrange
        let input = create_test_data();

        // Act
        let result = function_to_test(input);

        // Assert
        assert_eq!(result, expected_value);
    }

    #[tokio::test]
    async fn test_async_function() {
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

### Test Data

- Use fixtures for test data
- Create helper functions for common test setup
- Clean up resources in tests (databases, files)
- Use `tempfile` for temporary files

### Running Tests

```bash
# All tests
cargo test

# Specific module
cargo test api::google_indexing

# With logging output
RUST_LOG=debug cargo test -- --nocapture

# Single-threaded (for database tests)
cargo test -- --test-threads=1
```

## Documentation

### What to Document

- **Public API** - All public functions, types, and modules
- **Examples** - Common use cases and workflows
- **Configuration** - All configuration options
- **Architecture** - System design and decisions
- **Troubleshooting** - Common issues and solutions

### Documentation Style

- Use clear, simple language
- Include code examples
- Explain prerequisites and assumptions
- Provide links to related documentation

### Building Documentation

```bash
# Build API docs
cargo doc --no-deps --open

# Check for broken links
cargo doc --no-deps 2>&1 | grep warning
```

## Pull Request Process

### Before Submitting

Ensure your PR:

1. **Passes all tests**: `cargo test`
2. **Passes clippy**: `cargo clippy -- -D warnings`
3. **Is formatted**: `cargo fmt`
4. **Has documentation** for public APIs
5. **Has tests** for new functionality
6. **Updates CHANGELOG.md** if appropriate

### PR Description

Your PR description should include:

- **What** - What does this PR do?
- **Why** - Why is this change needed?
- **How** - How does it work?
- **Testing** - How was it tested?
- **Related Issues** - Links to related issues

Template:

```markdown
## Description

Brief description of what this PR does.

## Motivation

Why is this change needed? What problem does it solve?

## Changes

- List key changes
- Be specific about what was modified

## Testing

- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manually tested with...

## Checklist

- [ ] Code follows style guidelines
- [ ] Tests pass locally
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] No breaking changes (or clearly documented)

## Related Issues

Closes #123
Related to #456
```

### Review Process

1. **Automated checks** must pass (tests, clippy, formatting)
2. **Maintainer review** - A maintainer will review your code
3. **Address feedback** - Make requested changes
4. **Approval** - Once approved, a maintainer will merge

### After Merge

- Delete your feature branch
- Update your local main branch
- Close related issues if not auto-closed

## Issue Guidelines

### Bug Reports

When reporting bugs, include:

- **Description** - Clear description of the bug
- **Steps to reproduce** - Exact steps to reproduce
- **Expected behavior** - What should happen
- **Actual behavior** - What actually happens
- **Environment** - OS, Rust version, etc.
- **Logs** - Relevant error messages or logs

Template:

```markdown
## Bug Description

Brief description of the bug.

## Steps to Reproduce

1. Run command: `indexer-cli ...`
2. Observe error
3. ...

## Expected Behavior

What should happen.

## Actual Behavior

What actually happens.

## Environment

- OS: Linux/macOS/Windows
- Rust version: 1.75.0
- indexer-cli version: 0.1.0

## Logs

```
Paste relevant logs here
```

## Additional Context

Any other relevant information.
```

### Feature Requests

When requesting features, include:

- **Description** - What feature you'd like
- **Use case** - Why you need it
- **Alternatives** - What you've tried
- **Additional context** - Any other relevant information

## Community

### Getting Help

- **Issues** - For bugs and feature requests
- **Discussions** - For questions and general discussion
- **Pull Requests** - For code contributions

### Communication

- Be respectful and professional
- Assume good intentions
- Ask questions when unclear
- Provide constructive feedback

### Recognition

Contributors are recognized in:

- The CHANGELOG for significant contributions
- The GitHub contributors list
- Release notes for features and fixes

---

## Thank You!

Thank you for contributing to indexer-cli! Your efforts help make this project better for everyone.

If you have questions about contributing, feel free to ask in discussions or open an issue.

Happy coding!
