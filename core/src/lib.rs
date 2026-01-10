//! # yaml-lint-core
//!
//! Core YAML linting engine providing the fundamental types and traits
//! for building YAML linters.

pub mod config;
pub mod fixer;
pub mod linter;
pub mod output;
pub mod problem;
pub mod rules;

// Re-export main types for convenience
pub use config::Config;
pub use fixer::{FixResult, Fixer};
pub use linter::Linter;
pub use problem::{LintLevel, LintProblem};
pub use rules::{Rule, RuleRegistry};

/// Result type for lint operations
pub type Result<T> = std::result::Result<T, LintError>;

/// Errors that can occur during linting
#[derive(thiserror::Error, Debug)]
pub enum LintError {
    #[error("Failed to parse YAML: {0}")]
    ParseError(String),

    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    #[error("Unknown rule: {0}")]
    UnknownRule(String),
}
