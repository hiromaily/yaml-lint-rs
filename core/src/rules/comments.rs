//! Comments rule - enforces consistent comment formatting

use crate::problem::{LintLevel, LintProblem};
use crate::rules::{LintContext, Rule, RuleLevel};

/// Rule that enforces consistent comment formatting in YAML files
///
/// This rule checks:
/// - Space after `#` in comments (configurable)
/// - Minimum spacing before inline comments (configurable)
#[derive(Debug)]
pub struct CommentsRule {
    /// Require space after # (default: true)
    require_starting_space: bool,
    /// Ignore shebang lines (default: true)
    ignore_shebangs: bool,
    /// Minimum spaces before inline comment (default: 2)
    min_spaces_from_content: usize,
}

impl CommentsRule {
    /// Create a new rule with default settings
    pub fn new() -> Self {
        Self {
            require_starting_space: true,
            ignore_shebangs: true,
            min_spaces_from_content: 2,
        }
    }

    /// Create a new rule with custom settings
    pub fn with_config(
        require_starting_space: bool,
        ignore_shebangs: bool,
        min_spaces_from_content: usize,
    ) -> Self {
        Self {
            require_starting_space,
            ignore_shebangs,
            min_spaces_from_content,
        }
    }

    /// Check if a line is a shebang
    fn is_shebang(line: &str) -> bool {
        line.starts_with("#!")
    }

    /// Find the position of a comment in a line, if any
    /// Returns None if no comment found or if # is inside a string
    ///
    /// Handles YAML string escaping correctly:
    /// - Single-quoted strings: '' is an escaped single quote
    /// - Double-quoted strings: backslash escapes the next character
    fn find_comment_start(line: &str) -> Option<usize> {
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut chars = line.char_indices().peekable();

        while let Some((idx, ch)) = chars.next() {
            if !in_single_quote && !in_double_quote {
                match ch {
                    '#' => return Some(idx),
                    '\'' => in_single_quote = true,
                    '"' => in_double_quote = true,
                    _ => {}
                }
            } else if in_single_quote {
                if ch == '\'' {
                    // In YAML, '' is an escaped single quote
                    if chars.peek().is_some_and(|&(_, next_ch)| next_ch == '\'') {
                        chars.next(); // Consume the second quote of the pair
                    } else {
                        in_single_quote = false;
                    }
                }
            } else {
                // in_double_quote
                if ch == '\\' {
                    chars.next(); // Consume whatever character is escaped
                } else if ch == '"' {
                    in_double_quote = false;
                }
            }
        }
        None
    }
}

impl Default for CommentsRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for CommentsRule {
    fn name(&self) -> &'static str {
        "comments"
    }

    #[allow(clippy::collapsible_if)] // Nested ifs required for MSRV 1.85 compatibility
    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        let mut problems = Vec::new();

        for (line_idx, line) in context.lines.iter().enumerate() {
            let line_num = line_idx + 1;

            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            // Find comment start position
            let comment_start = match Self::find_comment_start(line) {
                Some(pos) => pos,
                None => continue,
            };

            let after_hash = &line[comment_start..];

            // Check for shebang
            if self.ignore_shebangs && comment_start == 0 && Self::is_shebang(line) {
                continue;
            }

            // Check space after #
            if self.require_starting_space && after_hash.len() > 1 {
                let char_after_hash = after_hash.chars().nth(1);
                // Allow empty comments (just #) and comments starting with space
                // Also allow ## for section headers, #! for shebangs in non-first lines
                if let Some(ch) = char_after_hash {
                    if ch != ' ' && ch != '#' && ch != '!' && ch != '\t' {
                        problems.push(LintProblem::new(
                            line_num,
                            comment_start + 2, // Position after #
                            "missing space after comment marker (#)",
                            self.name(),
                            LintLevel::Error,
                        ));
                    }
                }
            }

            // Check inline comment spacing (when comment is not at start of line)
            if comment_start > 0 {
                let before_comment = &line[..comment_start];
                let trimmed_before = before_comment.trim_end();

                // Count spaces before #
                let spaces_before = before_comment.len() - trimmed_before.len();

                // Only check if there's actual content before the comment
                if !trimmed_before.is_empty() && spaces_before < self.min_spaces_from_content {
                    problems.push(LintProblem::new(
                        line_num,
                        comment_start + 1,
                        format!(
                            "too few spaces before comment ({} < {})",
                            spaces_before, self.min_spaces_from_content
                        ),
                        self.name(),
                        LintLevel::Error,
                    ));
                }
            }
        }

        problems
    }

    fn default_level(&self) -> RuleLevel {
        RuleLevel::Error
    }

    fn is_fixable(&self) -> bool {
        true
    }

    #[allow(clippy::collapsible_if)] // Nested ifs required for MSRV 1.85 compatibility
    fn fix(&self, content: &str, _problem: &LintProblem) -> Option<String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut result_lines: Vec<String> = Vec::new();
        let mut made_changes = false;

        for (line_idx, line) in lines.iter().enumerate() {
            let mut fixed_line = line.to_string();

            // Skip empty lines
            if line.trim().is_empty() {
                result_lines.push(fixed_line);
                continue;
            }

            // Find comment start position
            if let Some(comment_start) = Self::find_comment_start(line) {
                // Skip shebangs on first line
                if self.ignore_shebangs && line_idx == 0 && Self::is_shebang(line) {
                    result_lines.push(fixed_line);
                    continue;
                }

                let after_hash = &line[comment_start..];

                // Fix space after #
                if self.require_starting_space && after_hash.len() > 1 {
                    let char_after_hash = after_hash.chars().nth(1);
                    if let Some(ch) = char_after_hash {
                        if ch != ' ' && ch != '#' && ch != '!' && ch != '\t' {
                            // Insert space after #
                            let before = &line[..comment_start + 1];
                            let after = &line[comment_start + 1..];
                            fixed_line = format!("{} {}", before, after);
                            made_changes = true;
                        }
                    }
                }

                // Fix inline comment spacing
                if comment_start > 0 {
                    let before_comment = &fixed_line[..comment_start];
                    let trimmed_before = before_comment.trim_end();

                    if !trimmed_before.is_empty() {
                        let spaces_before = before_comment.len() - trimmed_before.len();
                        if spaces_before < self.min_spaces_from_content {
                            let comment_part = &fixed_line[comment_start..];
                            let spaces = " ".repeat(self.min_spaces_from_content);
                            fixed_line = format!("{}{}{}", trimmed_before, spaces, comment_part);
                            made_changes = true;
                        }
                    }
                }
            }

            result_lines.push(fixed_line);
        }

        if !made_changes {
            return None;
        }

        let mut result = result_lines.join("\n");
        if content.ends_with('\n') {
            result.push('\n');
        }

        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proper_comment() {
        let yaml = "# This is a proper comment\nkey: value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = CommentsRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_missing_space_after_hash() {
        let yaml = "#This comment has no space\nkey: value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = CommentsRule::new();
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
        assert!(problems[0].message.contains("missing space"));
    }

    #[test]
    fn test_shebang_ignored() {
        let yaml = "#!/bin/bash\n# Normal comment\nkey: value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = CommentsRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_inline_comment_proper_spacing() {
        let yaml = "key: value  # This is an inline comment\n";
        let context = LintContext::new(yaml.to_string());
        let rule = CommentsRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_inline_comment_insufficient_spacing() {
        let yaml = "key: value # Too close\n";
        let context = LintContext::new(yaml.to_string());
        let rule = CommentsRule::new();
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
        assert!(problems[0].message.contains("too few spaces"));
    }

    #[test]
    fn test_hash_in_string_ignored() {
        let yaml = "key: \"value with # hash\"\n";
        let context = LintContext::new(yaml.to_string());
        let rule = CommentsRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_hash_in_single_quote_ignored() {
        let yaml = "key: 'value with # hash'\n";
        let context = LintContext::new(yaml.to_string());
        let rule = CommentsRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_hash_after_escaped_single_quote() {
        // In YAML, '' within single quotes is an escaped single quote
        let yaml = "key: 'Don''t find a # here'\n";
        let context = LintContext::new(yaml.to_string());
        let rule = CommentsRule::new();
        let problems = rule.check(&context);

        assert!(
            problems.is_empty(),
            "Should not find a comment inside a single-quoted string with escaped quotes"
        );
    }

    #[test]
    fn test_hash_after_escaped_double_quote() {
        // In YAML, \" within double quotes is an escaped double quote
        let yaml = "key: \"value with \\\" and # here\"\n";
        let context = LintContext::new(yaml.to_string());
        let rule = CommentsRule::new();
        let problems = rule.check(&context);

        assert!(
            problems.is_empty(),
            "Should not find a comment inside a double-quoted string with escaped quotes"
        );
    }

    #[test]
    fn test_comment_after_single_quoted_string() {
        // Comment should be detected after the closing quote
        let yaml = "key: 'value'  # This is a comment\n";
        let context = LintContext::new(yaml.to_string());
        let rule = CommentsRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_section_header_comment() {
        let yaml = "## Section Header\nkey: value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = CommentsRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_empty_comment() {
        let yaml = "#\nkey: value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = CommentsRule::new();
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_multiple_issues() {
        let yaml = "#Bad comment\nkey: value# Inline too close\n";
        let context = LintContext::new(yaml.to_string());
        let rule = CommentsRule::new();
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 2);
    }

    #[test]
    fn test_custom_config_no_space_required() {
        let yaml = "#No space needed\n";
        let context = LintContext::new(yaml.to_string());
        let rule = CommentsRule::with_config(false, true, 2);
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_custom_min_spaces() {
        let yaml = "key: value   # 3 spaces\n";
        let context = LintContext::new(yaml.to_string());
        let rule = CommentsRule::with_config(true, true, 4);
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
        assert!(problems[0].message.contains("too few spaces"));
    }

    #[test]
    fn test_fix_missing_space() {
        let yaml = "#No space\nkey: value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = CommentsRule::new();
        let problems = rule.check(&context);

        assert!(!problems.is_empty());

        if let Some(fixed) = rule.fix(yaml, &problems[0]) {
            assert!(fixed.starts_with("# "));
        }
    }

    #[test]
    fn test_fix_inline_spacing() {
        let yaml = "key: value# Comment\n";
        let context = LintContext::new(yaml.to_string());
        let rule = CommentsRule::new();
        let problems = rule.check(&context);

        assert!(!problems.is_empty());

        if let Some(fixed) = rule.fix(yaml, &problems[0]) {
            assert!(fixed.contains("value  #"));
        }
    }
}
