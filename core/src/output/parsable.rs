//! Parsable output formatter for editor integration

use crate::output::OutputFormatter;
use crate::problem::LintProblem;

/// Parsable output formatter (file:line:col: [level] message)
/// Format suitable for editors and CI tools
pub struct ParsableFormatter;

impl OutputFormatter for ParsableFormatter {
    fn format_problems(&self, problems: &[LintProblem], file_path: &str) -> String {
        let mut output = String::new();

        for problem in problems {
            // Format: "file.yaml:12:3: [error] trailing spaces (trailing-spaces)"
            output.push_str(&format!(
                "{}:{}:{}: [{}] {} ({})\n",
                file_path,
                problem.line,
                problem.column,
                problem.level,
                problem.message,
                problem.rule
            ));
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
        let formatter = ParsableFormatter;
        let problems = vec![LintProblem::new(
            12,
            3,
            "trailing spaces",
            "trailing-spaces",
            LintLevel::Error,
        )];

        let output = formatter.format_problems(&problems, "test.yaml");
        assert_eq!(
            output,
            "test.yaml:12:3: [error] trailing spaces (trailing-spaces)\n"
        );
    }

    #[test]
    fn test_format_no_problems() {
        let formatter = ParsableFormatter;
        let output = formatter.format_problems(&[], "test.yaml");
        assert!(output.is_empty());
    }

    #[test]
    fn test_format_multiple_problems() {
        let formatter = ParsableFormatter;
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
        assert!(output.contains("test.yaml:1:10: [error]"));
        assert!(output.contains("test.yaml:5:1: [warning]"));
    }
}
