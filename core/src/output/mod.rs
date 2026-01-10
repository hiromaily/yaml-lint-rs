//! Output formatters for lint problems

use crate::problem::LintProblem;

pub mod colored;
pub mod parsable;
pub mod standard;

pub use colored::ColoredFormatter;
pub use parsable::ParsableFormatter;
pub use standard::StandardFormatter;

/// Trait for formatting lint problems for output
pub trait OutputFormatter {
    /// Format a list of problems for a given file
    fn format_problems(&self, problems: &[LintProblem], file_path: &str) -> String;
}

/// Output format types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Standard human-readable format
    Standard,
    /// Colored output (future)
    Colored,
    /// Machine-parsable format (future)
    Parsable,
}

impl OutputFormat {
    /// Get a formatter for this format
    pub fn formatter(&self) -> Box<dyn OutputFormatter> {
        match self {
            OutputFormat::Standard => Box::new(StandardFormatter),
            OutputFormat::Colored => Box::new(ColoredFormatter),
            OutputFormat::Parsable => Box::new(ParsableFormatter),
        }
    }
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "standard" => Ok(OutputFormat::Standard),
            "colored" => Ok(OutputFormat::Colored),
            "parsable" => Ok(OutputFormat::Parsable),
            _ => Err(format!("Unknown output format: {}", s)),
        }
    }
}
