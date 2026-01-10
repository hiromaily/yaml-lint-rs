//! Standard output formatter

use crate::output::OutputFormatter;
use crate::problem::LintProblem;

/// Standard human-readable output formatter
pub struct StandardFormatter;

impl OutputFormatter for StandardFormatter {
    fn format_problems(&self, problems: &[LintProblem], file_path: &str) -> String {
        if problems.is_empty() {
            return String::new();
        }

        let mut output = String::new();
        output.push_str(file_path);
        output.push('\n');

        for problem in problems {
            // Format: "  12:3      error    trailing spaces  (trailing-spaces)"
            output.push_str(&format!("  {}:{}", problem.line, problem.column,));

            // Pad to align the level column (aim for column 12)
            let pos_str = format!("{}:{}", problem.line, problem.column);
            let padding = if pos_str.len() < 10 {
                10 - pos_str.len()
            } else {
                1
            };
            output.push_str(&" ".repeat(padding));

            // Level
            output.push_str(&format!("{:<8}", problem.level.to_string()));

            // Message and rule
            output.push_str(&format!(" {}  ({})", problem.message, problem.rule));
            output.push('\n');
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::problem::LintLevel;

    #[test]
    fn test_format_single_problem() {
        let formatter = StandardFormatter;
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
        assert!(output.contains("error"));
        assert!(output.contains("trailing spaces"));
        assert!(output.contains("(trailing-spaces)"));
    }

    #[test]
    fn test_format_no_problems() {
        let formatter = StandardFormatter;
        let output = formatter.format_problems(&[], "test.yaml");
        assert!(output.is_empty());
    }

    #[test]
    fn test_format_multiple_problems() {
        let formatter = StandardFormatter;
        let problems = vec![
            LintProblem::new(
                1,
                10,
                "trailing spaces",
                "trailing-spaces",
                LintLevel::Error,
            ),
            LintProblem::new(5, 1, "line too long", "line-length", LintLevel::Warning),
        ];

        let output = formatter.format_problems(&problems, "test.yaml");
        assert!(output.contains("1:10"));
        assert!(output.contains("5:1"));
    }
}
