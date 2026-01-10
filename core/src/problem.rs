//! Lint problem types and levels

use std::cmp::Ordering;

/// Severity level of a lint problem
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LintLevel {
    /// Error level problem
    Error,
    /// Warning level problem
    Warning,
}

impl std::fmt::Display for LintLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LintLevel::Error => write!(f, "error"),
            LintLevel::Warning => write!(f, "warning"),
        }
    }
}

/// A lint problem found in a YAML file
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintProblem {
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Description of the problem
    pub message: String,
    /// Name of the rule that detected this problem
    pub rule: String,
    /// Severity level
    pub level: LintLevel,
}

impl LintProblem {
    /// Create a new lint problem
    pub fn new(
        line: usize,
        column: usize,
        message: impl Into<String>,
        rule: impl Into<String>,
        level: LintLevel,
    ) -> Self {
        Self {
            line,
            column,
            message: message.into(),
            rule: rule.into(),
            level,
        }
    }
}

impl PartialOrd for LintProblem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LintProblem {
    fn cmp(&self, other: &Self) -> Ordering {
        // Sort by line, then column, then level (errors before warnings)
        self.line
            .cmp(&other.line)
            .then_with(|| self.column.cmp(&other.column))
            .then_with(|| match (&self.level, &other.level) {
                (LintLevel::Error, LintLevel::Warning) => Ordering::Less,
                (LintLevel::Warning, LintLevel::Error) => Ordering::Greater,
                _ => Ordering::Equal,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_problem_ordering() {
        let p1 = LintProblem::new(1, 1, "test", "rule", LintLevel::Error);
        let p2 = LintProblem::new(1, 2, "test", "rule", LintLevel::Error);
        let p3 = LintProblem::new(2, 1, "test", "rule", LintLevel::Warning);

        assert!(p1 < p2);
        assert!(p2 < p3);
        assert!(p1 < p3);
    }

    #[test]
    fn test_error_before_warning_same_position() {
        let error = LintProblem::new(1, 1, "test", "rule", LintLevel::Error);
        let warning = LintProblem::new(1, 1, "test", "rule", LintLevel::Warning);

        assert!(error < warning);
    }
}
