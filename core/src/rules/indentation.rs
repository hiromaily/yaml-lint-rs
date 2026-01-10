//! Indentation rule - validates consistent indentation

use crate::problem::{LintLevel, LintProblem};
use crate::rules::{LintContext, Rule, RuleLevel};

/// Indentation configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndentSpaces {
    /// Fixed number of spaces
    Fixed(usize),
    /// Consistent indentation (detect from first indented line)
    Consistent,
}

/// Rule that checks indentation consistency
#[derive(Debug)]
pub struct IndentationRule {
    /// Number of spaces for indentation
    pub spaces: IndentSpaces,
}

impl IndentationRule {
    /// Create a new indentation rule with consistent mode
    pub fn new() -> Self {
        Self {
            spaces: IndentSpaces::Consistent,
        }
    }

    /// Create an indentation rule with fixed spaces
    pub fn with_spaces(spaces: usize) -> Self {
        Self {
            spaces: IndentSpaces::Fixed(spaces),
        }
    }

    /// Create an indentation rule with consistent mode
    pub fn consistent() -> Self {
        Self {
            spaces: IndentSpaces::Consistent,
        }
    }
}

impl Default for IndentationRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for IndentationRule {
    fn name(&self) -> &'static str {
        "indentation"
    }

    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        let mut problems = Vec::new();

        // Detect indentation size if in consistent mode
        let indent_size = match self.spaces {
            IndentSpaces::Fixed(n) => n,
            IndentSpaces::Consistent => detect_indent_size(context),
        };

        if indent_size == 0 {
            // Could not detect or no indentation in file
            return problems;
        }

        let mut expected_indent: Option<usize> = None;
        let mut indent_stack: Vec<usize> = vec![0];

        for (line_idx, line) in context.lines.iter().enumerate() {
            // Skip empty lines and comment-only lines
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Check for tabs
            if line.starts_with('\t') || line.contains("\t ") || line.contains(" \t") {
                problems.push(LintProblem::new(
                    line_idx + 1,
                    1,
                    "found tab character in indentation",
                    self.name(),
                    LintLevel::Error,
                ));
                continue;
            }

            // Calculate current indentation
            let current_indent = line.len() - line.trim_start().len();

            // Skip document markers
            if trimmed.starts_with("---") || trimmed.starts_with("...") {
                expected_indent = Some(0);
                indent_stack = vec![0];
                continue;
            }

            // Check if indentation is a multiple of indent_size
            if current_indent % indent_size != 0 {
                problems.push(LintProblem::new(
                    line_idx + 1,
                    1,
                    format!(
                        "wrong indentation: expected multiple of {} but got {}",
                        indent_size, current_indent
                    ),
                    self.name(),
                    LintLevel::Error,
                ));
                continue;
            }

            // Handle list items specially
            if trimmed.starts_with("- ") || trimmed == "-" {
                // List item - adjust expectations
                let list_indent = current_indent;

                // List items should be at a valid indentation level
                #[allow(clippy::collapsible_if)]
                if let Some(parent_indent) = indent_stack.last() {
                    if list_indent <= *parent_indent && list_indent != 0 {
                        // List at same or less indentation than parent
                        // Pop stack until we find the right level
                        while indent_stack.len() > 1 && indent_stack.last().unwrap() >= &list_indent
                        {
                            indent_stack.pop();
                        }
                    }
                }

                // Update expected indent for next line
                expected_indent = Some(list_indent + indent_size);
                indent_stack.push(list_indent);
                continue;
            }

            // Check if indentation matches expectation
            if let Some(expected) = expected_indent {
                if current_indent > expected && (current_indent - expected) % indent_size != 0 {
                    problems.push(LintProblem::new(
                        line_idx + 1,
                        1,
                        format!(
                            "wrong indentation: expected {} but got {}",
                            expected, current_indent
                        ),
                        self.name(),
                        LintLevel::Error,
                    ));
                }

                // Update stack based on current indentation
                if current_indent > expected {
                    // Deeper indentation
                    indent_stack.push(expected);
                    expected_indent = Some(current_indent);
                } else if current_indent < expected {
                    // Less indentation - pop stack
                    while indent_stack.len() > 1 && indent_stack.last().unwrap() >= &current_indent
                    {
                        indent_stack.pop();
                    }
                    expected_indent = indent_stack.last().copied();
                }
            } else {
                // First indented line
                if current_indent > 0 {
                    expected_indent = Some(current_indent);
                    indent_stack.push(0);
                    indent_stack.push(current_indent);
                } else {
                    expected_indent = Some(0);
                }
            }

            // Check for key-value pairs to update expectations
            if line.contains(':') && !trimmed.starts_with('#') {
                // This might be a key with nested values
                let after_colon = line.split(':').nth(1).unwrap_or("").trim();
                if after_colon.is_empty() || after_colon.starts_with('#') {
                    // Key with no value on same line - expect indented content
                    expected_indent = Some(current_indent + indent_size);
                }
            }
        }

        problems
    }

    fn default_level(&self) -> RuleLevel {
        RuleLevel::Error
    }
}

/// Detect the indentation size used in the document
fn detect_indent_size(context: &LintContext) -> usize {
    let mut indents = Vec::new();

    let mut prev_indent = 0;

    for line in &context.lines {
        // Skip empty lines and comments
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Skip document markers
        if trimmed.starts_with("---") || trimmed.starts_with("...") {
            prev_indent = 0;
            continue;
        }

        let current_indent = line.len() - line.trim_start().len();

        if current_indent > prev_indent && prev_indent == 0 {
            // First indentation level
            indents.push(current_indent);
        } else if current_indent > prev_indent {
            // Increased indentation
            let diff = current_indent - prev_indent;
            indents.push(diff);
        }

        if current_indent > 0 {
            prev_indent = current_indent;
        }
    }

    // Find the most common indent size (likely 2 or 4)
    if indents.is_empty() {
        return 2; // Default to 2 if we can't detect
    }

    // Return the smallest non-zero indent (likely the base indent)
    *indents.iter().min().unwrap_or(&2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correct_indentation() {
        let yaml = "key1: value1\nkey2:\n  nested: value\n  nested2: value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = IndentationRule::with_spaces(2);
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_wrong_indentation() {
        let yaml = "key1: value1\nkey2:\n   nested: value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = IndentationRule::with_spaces(2);
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
        assert!(problems[0].message.contains("wrong indentation"));
    }

    #[test]
    fn test_tab_character() {
        let yaml = "key1: value1\n\tnested: value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = IndentationRule::with_spaces(2);
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
        assert!(problems[0].message.contains("tab character"));
    }

    #[test]
    fn test_four_space_indentation() {
        let yaml = "key1: value1\nkey2:\n    nested: value\n    nested2: value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = IndentationRule::with_spaces(4);
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_consistent_mode() {
        let yaml = "key1: value1\nkey2:\n  nested: value\n  nested2: value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = IndentationRule::consistent();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_list_indentation() {
        let yaml = "list:\n  - item1\n  - item2\n";
        let context = LintContext::new(yaml.to_string());
        let rule = IndentationRule::with_spaces(2);
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_nested_lists() {
        let yaml = "list:\n  - item1:\n      nested: value\n  - item2\n";
        let context = LintContext::new(yaml.to_string());
        let rule = IndentationRule::with_spaces(2);
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_detect_indent_size_2() {
        let yaml = "key:\n  nested: value\n  nested2:\n    deep: value\n";
        let context = LintContext::new(yaml.to_string());

        let size = detect_indent_size(&context);
        assert_eq!(size, 2);
    }

    #[test]
    fn test_detect_indent_size_4() {
        let yaml = "key:\n    nested: value\n    nested2:\n        deep: value\n";
        let context = LintContext::new(yaml.to_string());

        let size = detect_indent_size(&context);
        assert_eq!(size, 4);
    }
}
