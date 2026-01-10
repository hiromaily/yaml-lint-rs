//! Colored output formatter

use crate::output::OutputFormatter;
use crate::problem::{LintLevel, LintProblem};
use colored::Colorize;

/// Colored output formatter using ANSI colors
pub struct ColoredFormatter;

impl OutputFormatter for ColoredFormatter {
    fn format_problems(&self, problems: &[LintProblem], file_path: &str) -> String {
        if problems.is_empty() {
            return String::new();
        }

        let mut output = String::new();
        output.push_str(&file_path.bold().to_string());
        output.push('\n');

        for problem in problems {
            // Format: "  12:3      error    trailing spaces  (trailing-spaces)"
            let position = format!("{}:{}", problem.line, problem.column);
            output.push_str("  ");
            output.push_str(&position);

            // Pad to align the level column (aim for column 12)
            let padding = if position.len() < 10 {
                10 - position.len()
            } else {
                1
            };
            output.push_str(&" ".repeat(padding));

            // Level with color
            let level_str = match problem.level {
                LintLevel::Error => format!("{:<8}", "error").red().to_string(),
                LintLevel::Warning => format!("{:<8}", "warning").yellow().to_string(),
            };
            output.push_str(&level_str);

            // Message and rule
            output.push_str(&format!(" {}  ", problem.message));
            output.push_str(&format!("({})", problem.rule).dimmed().to_string());
            output.push('\n');
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_single_problem() {
        let formatter = ColoredFormatter;
        let problems = vec![LintProblem::new(
            12,
            3,
            "trailing spaces",
            "trailing-spaces",
            LintLevel::Error,
        )];

        let output = formatter.format_problems(&problems, "test.yaml");
        assert!(output.contains("test.yaml"));
        assert!(output.contains("12:3"));
        assert!(output.contains("trailing spaces"));
        assert!(output.contains("(trailing-spaces)"));
    }

    #[test]
    fn test_format_no_problems() {
        let formatter = ColoredFormatter;
        let output = formatter.format_problems(&[], "test.yaml");
        assert!(output.is_empty());
    }

    #[test]
    fn test_format_warning() {
        let formatter = ColoredFormatter;
        let problems = vec![LintProblem::new(
            5,
            1,
            "line too long",
            "line-length",
            LintLevel::Warning,
        )];

        let output = formatter.format_problems(&problems, "test.yaml");
        assert!(output.contains("5:1"));
        assert!(output.contains("line too long"));
    }
}
