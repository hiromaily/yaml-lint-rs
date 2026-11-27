//! Auto-fix functionality for lint problems

use crate::problem::LintProblem;
use crate::rules::{LintContext, RuleRegistry};
use std::collections::HashMap;

/// Result of a fix operation for a single file
#[derive(Debug, Clone)]
pub struct FixResult {
    /// Original file path
    pub path: String,
    /// Number of problems fixed
    pub fixes_applied: usize,
    /// Breakdown of fixes by rule
    pub fixes_by_rule: HashMap<String, usize>,
    /// Problems that could not be fixed
    pub unfixable_problems: Vec<LintProblem>,
    /// The fixed content (if any fixes were applied)
    pub fixed_content: Option<String>,
}

impl FixResult {
    /// Create a new FixResult with no fixes
    pub fn new(path: String) -> Self {
        Self {
            path,
            fixes_applied: 0,
            fixes_by_rule: HashMap::new(),
            unfixable_problems: Vec::new(),
            fixed_content: None,
        }
    }

    /// Check if any fixes were applied
    pub fn has_fixes(&self) -> bool {
        self.fixes_applied > 0
    }

    /// Check if there are unfixable problems
    pub fn has_unfixable(&self) -> bool {
        !self.unfixable_problems.is_empty()
    }
}

/// Fixer that can automatically fix lint problems
#[derive(Debug)]
pub struct Fixer<'a> {
    registry: &'a RuleRegistry,
}

impl<'a> Fixer<'a> {
    /// Create a new Fixer with the given rule registry
    pub fn new(registry: &'a RuleRegistry) -> Self {
        Self { registry }
    }

    /// Fix all fixable problems in the given content
    /// Returns the fix result including the fixed content
    pub fn fix(&self, path: &str, content: &str) -> FixResult {
        self.fix_file(std::path::Path::new(path), content)
    }

    /// Fix YAML using its file path for rule-level ignore patterns.
    pub fn fix_file(&self, path: &std::path::Path, content: &str) -> FixResult {
        let mut result = FixResult::new(path.display().to_string());
        let mut current_content = content.to_string();

        // Iteratively fix problems until no more fixes can be applied
        // This handles cases where fixing one problem might reveal or affect others
        loop {
            let context = LintContext::new(current_content.clone());
            let mut problems = self.registry.check_for_path(&context, Some(path));

            // Try to fix problems in order (sorted by line number)
            // This applies fixes top-to-bottom and avoids HashMap overhead
            let mut made_progress = false;
            for problem in &problems {
                if let Some(rule) = self.registry.get(&problem.rule)
                    && rule.is_fixable()
                    && let Some(fixed) = rule.fix(&current_content, problem)
                {
                    current_content = fixed;
                    result.fixes_applied += 1;
                    *result
                        .fixes_by_rule
                        .entry(problem.rule.clone())
                        .or_insert(0) += 1;
                    made_progress = true;
                    break; // Re-check all problems after each fix
                }
            }
            if !made_progress {
                // Some rules cannot safely fix every diagnostic in context.
                crate::syntax::check(&current_content, &mut problems);
                result.unfixable_problems = problems;
                break;
            }
        }

        if result.fixes_applied > 0 {
            result.fixed_content = Some(current_content);
        }

        result
    }

    /// Check what fixes would be applied without actually applying them (dry-run)
    pub fn dry_run(&self, path: &str, content: &str) -> FixResult {
        // For dry-run, we actually apply fixes to a copy to see what would change
        self.fix(path, content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_trailing_spaces() {
        let registry = RuleRegistry::with_defaults();
        let fixer = Fixer::new(&registry);

        let content = "key: value   \nkey2: value2\n";
        let result = fixer.fix("test.yaml", content);

        assert!(result.has_fixes());
        assert_eq!(result.fixes_applied, 1);
        assert!(result.fixes_by_rule.contains_key("trailing-spaces"));
        assert_eq!(
            result.fixed_content,
            Some("key: value\nkey2: value2\n".to_string())
        );
    }

    #[test]
    fn test_fix_newline_at_end() {
        let registry = RuleRegistry::with_defaults();
        let fixer = Fixer::new(&registry);

        let content = "key: value";
        let result = fixer.fix("test.yaml", content);

        assert!(result.has_fixes());
        assert!(result.fixes_by_rule.contains_key("new-line-at-end-of-file"));
        assert_eq!(result.fixed_content, Some("key: value\n".to_string()));
    }

    #[test]
    fn test_no_fixes_needed() {
        let registry = RuleRegistry::with_defaults();
        let fixer = Fixer::new(&registry);

        let content = "key: value\n";
        let result = fixer.fix("test.yaml", content);

        assert!(!result.has_fixes());
        assert_eq!(result.fixes_applied, 0);
        assert!(result.fixed_content.is_none());
    }

    #[test]
    fn test_multiple_fixes() {
        let registry = RuleRegistry::with_defaults();
        let fixer = Fixer::new(&registry);

        let content = "key: value   \nkey2: value2  ";
        let result = fixer.fix("test.yaml", content);

        assert!(result.has_fixes());
        // Should fix trailing spaces (2) and add newline (1)
        assert!(result.fixes_applied >= 2);
    }

    #[test]
    fn test_unfixable_problems() {
        let registry = RuleRegistry::with_defaults();
        let fixer = Fixer::new(&registry);

        // Duplicate keys can't be auto-fixed
        let content = "name: value1\nname: value2\n";
        let result = fixer.fix("test.yaml", content);

        assert!(result.has_unfixable());
        assert!(
            result
                .unfixable_problems
                .iter()
                .any(|p| p.rule == "key-duplicates")
        );
    }
}

#[cfg(test)]
mod directive_tests {
    use super::*;

    #[test]
    fn fix_preserves_suppressed_trailing_spaces() {
        let registry = RuleRegistry::with_defaults();
        let yaml = "# yamllint disable-line rule:trailing-spaces\na: value   \nb: value   \n";
        let result = Fixer::new(&registry).fix("test.yaml", yaml);
        assert_eq!(
            result.fixed_content.as_deref(),
            Some("# yamllint disable-line rule:trailing-spaces\na: value   \nb: value\n")
        );
    }

    #[test]
    fn fix_preserves_suppressed_comment_spacing() {
        let registry = RuleRegistry::with_defaults();
        let yaml = "a: value # yamllint disable-line rule:comments\nb: value # comment\n";
        let result = Fixer::new(&registry).fix("test.yaml", yaml);
        assert_eq!(
            result.fixed_content.as_deref(),
            Some("a: value # yamllint disable-line rule:comments\nb: value  # comment\n")
        );
    }

    #[test]
    fn fix_preserves_a_standalone_directives_blank_target() {
        use crate::rules::empty_lines::EmptyLinesRule;

        let mut registry = RuleRegistry::with_defaults();
        registry.set_level("document-start", crate::rules::RuleLevel::Disable);
        registry.register(Box::new(EmptyLinesRule::with_config(0, 0, 0)));
        let yaml = "a: value\n# yamllint disable-line rule:trailing-spaces\n\n\nb: value   \n";
        let result = Fixer::new(&registry).fix("test.yaml", yaml);

        assert_eq!(
            result.fixed_content.as_deref(),
            Some("a: value\n# yamllint disable-line rule:trailing-spaces\n\nb: value\n")
        );
        assert_eq!(result.unfixable_problems.len(), 1);
        assert_eq!(result.unfixable_problems[0].rule, "empty-lines");
        assert_eq!(result.unfixable_problems[0].line, 3);
    }

    #[test]
    fn fix_preserves_a_suppressed_blank_line_run() {
        let registry = RuleRegistry::with_defaults();
        let yaml = "# yamllint disable-line rule:empty-lines\n\n\n\na: value\n\n";
        let result = Fixer::new(&registry).fix("test.yaml", yaml);
        assert_eq!(
            result.fixed_content.as_deref(),
            Some("# yamllint disable-line rule:empty-lines\n\n\n\na: value\n")
        );
    }
}
