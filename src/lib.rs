// Library module for indexer-cli

// Core modules
pub mod database;
pub mod types;

// Example module for testing
pub mod example {
    pub fn hello() -> &'static str {
        "Hello from indexer-cli"
    }
}
