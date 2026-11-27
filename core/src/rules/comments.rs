//! Comments rule - enforces consistent comment formatting

use super::{directives::comments, new_lines::split_ending};
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

    fn missing_space_at(&self, line: &str, hash: usize, line_number: usize) -> Option<usize> {
        let text = line[hash..].trim_start_matches('#');
        let first = text.chars().next()?;
        let shebang = self.ignore_shebangs && line_number == 1 && hash == 0 && first == '!';
        (self.require_starting_space && !shebang && !matches!(first, ' ' | '\r' | '\n' | '\0'))
            .then_some(line.len() - text.len())
    }

    fn inline_content_end(&self, line: &str, hash: usize) -> Option<usize> {
        let end = line[..hash].trim_end_matches([' ', '\t']).len();
        (end > 0 && hash - end < self.min_spaces_from_content).then_some(end)
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

    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        let mut problems = Vec::new();

        for (line_number, line, hash) in comments(&context.content) {
            if self.inline_content_end(line, hash).is_some() {
                problems.push(LintProblem::new(
                    line_number,
                    line[..hash].chars().count() + 1,
                    format!(
                        "too few spaces before comment: expected {}",
                        self.min_spaces_from_content
                    ),
                    self.name(),
                    LintLevel::Error,
                ));
            }
            if let Some(offset) = self.missing_space_at(line, hash, line_number) {
                problems.push(LintProblem::new(
                    line_number,
                    line[..offset].chars().count() + 1,
                    "missing starting space in comment",
                    self.name(),
                    LintLevel::Error,
                ));
            }
        }

        problems
    }

    fn default_level(&self) -> RuleLevel {
        RuleLevel::Warning
    }

    fn is_fixable(&self) -> bool {
        true
    }

    fn fix(&self, content: &str, _problem: &LintProblem) -> Option<String> {
        let suppressed = super::directives::lines_for_rule(content, self.name());
        let mut result = String::with_capacity(content.len());
        let mut made_changes = false;

        let mut comments = comments(content).peekable();
        for (line_index, chunk) in content.split_inclusive('\n').enumerate() {
            let line_number = line_index + 1;
            let Some((_, _, hash)) = comments.next_if(|(number, _, _)| *number == line_number)
            else {
                result.push_str(chunk);
                continue;
            };
            if suppressed.contains(&line_number) {
                result.push_str(chunk);
                continue;
            }

            let (line, ending) = split_ending(chunk);
            if let Some(end) = self.inline_content_end(line, hash) {
                result.push_str(&line[..end]);
                result.push_str(&" ".repeat(self.min_spaces_from_content));
                made_changes = true;
            } else {
                result.push_str(&line[..hash]);
            }
            if let Some(offset) = self.missing_space_at(line, hash, line_number) {
                result.push_str(&line[hash..offset]);
                result.push(' ');
                result.push_str(&line[offset..]);
                made_changes = true;
            } else {
                result.push_str(&line[hash..]);
            }
            result.push_str(ending);
        }

        made_changes.then_some(result)
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
        assert!(problems[0].message.contains("missing starting space"));
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
        let yaml = "#Bad comment\nkey: value # Inline too close\n";
        let context = LintContext::new(yaml.to_string());
        let rule = CommentsRule::new();
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 2);
    }

    #[test]
    fn only_ignores_shebang_on_first_line_and_preserves_scalar_hashes() {
        let source = "#!/bin/sh\nkey: value#fragment #! inline\n##heading\n";
        let rule = CommentsRule::new();
        let problems = rule.check(&LintContext::new(source.into()));
        let positions: Vec<_> = problems.iter().map(|p| (p.line, p.column)).collect();
        assert_eq!(positions, [(2, 21), (2, 22), (3, 3)]);
        assert_eq!(
            rule.fix(source, &problems[0]).as_deref(),
            Some("#!/bin/sh\nkey: value#fragment  # ! inline\n## heading\n")
        );
    }

    #[test]
    fn fix_preserves_non_breaking_space_in_plain_scalar() {
        let source = "key: value\u{00a0} # comment\n";
        let rule = CommentsRule::new();
        let problems = rule.check(&LintContext::new(source.into()));
        assert_eq!(problems.len(), 1);
        assert_eq!(
            rule.fix(source, &problems[0]).as_deref(),
            Some("key: value\u{00a0}  # comment\n")
        );
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
        let yaml = "key: value # Comment\n";
        let context = LintContext::new(yaml.to_string());
        let rule = CommentsRule::new();
        let problems = rule.check(&context);

        assert!(!problems.is_empty());

        if let Some(fixed) = rule.fix(yaml, &problems[0]) {
            assert!(fixed.contains("value  #"));
        }
    }
}
