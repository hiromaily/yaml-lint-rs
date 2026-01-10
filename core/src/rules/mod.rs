//! Linting rules and rule registry

use crate::problem::LintProblem;
use indexmap::IndexMap;

pub mod colons;
pub mod comments;
pub mod document_start;
pub mod empty_lines;
pub mod hyphens;
pub mod indentation;
pub mod key_duplicates;
pub mod line_length;
pub mod new_line_at_end_of_file;
pub mod trailing_spaces;
pub mod truthy;

/// Configuration for a specific rule
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleLevel {
    /// Rule is disabled
    Disable,
    /// Rule produces warnings
    Warning,
    /// Rule produces errors
    Error,
}

/// Context provided to rules during checking
#[derive(Debug)]
pub struct LintContext {
    /// The full content of the YAML file
    pub content: String,
    /// Lines of the content (for convenience)
    pub lines: Vec<String>,
}

impl LintContext {
    /// Create a new lint context from content
    pub fn new(content: String) -> Self {
        let lines = content.lines().map(|s| s.to_string()).collect();
        Self { content, lines }
    }
}

/// Trait that all linting rules must implement
pub trait Rule: Send + Sync + std::fmt::Debug {
    /// Returns the name of this rule (e.g., "trailing-spaces")
    fn name(&self) -> &'static str;

    /// Check the given context for problems
    fn check(&self, context: &LintContext) -> Vec<LintProblem>;

    /// Returns the default level for this rule
    fn default_level(&self) -> RuleLevel {
        RuleLevel::Error
    }

    /// Returns whether this rule supports auto-fixing
    fn is_fixable(&self) -> bool {
        false
    }

    /// Fix the content for a specific problem
    /// Returns the fixed content if the fix was successful, None otherwise
    fn fix(&self, _content: &str, _problem: &LintProblem) -> Option<String> {
        None
    }
}

/// Registry of all available rules
#[derive(Debug)]
pub struct RuleRegistry {
    rules: IndexMap<String, Box<dyn Rule>>,
    levels: IndexMap<String, RuleLevel>,
}

impl RuleRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            rules: IndexMap::new(),
            levels: IndexMap::new(),
        }
    }

    /// Create a registry with default rules
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register(Box::new(trailing_spaces::TrailingSpacesRule));
        registry.register(Box::new(line_length::LineLengthRule::new()));
        registry.register(Box::new(document_start::DocumentStartRule::new()));
        registry.register(Box::new(colons::ColonsRule::new()));
        registry.register(Box::new(key_duplicates::KeyDuplicatesRule));
        registry.register(Box::new(indentation::IndentationRule::new()));
        registry.register(Box::new(new_line_at_end_of_file::NewLineAtEndOfFileRule));
        registry.register(Box::new(empty_lines::EmptyLinesRule::new()));
        registry.register(Box::new(hyphens::HyphensRule::new()));
        registry.register(Box::new(comments::CommentsRule::new()));
        registry.register(Box::new(truthy::TruthyRule::new()));
        registry
    }

    /// Register a new rule
    pub fn register(&mut self, rule: Box<dyn Rule>) {
        let name = rule.name().to_string();
        let default_level = rule.default_level();
        self.rules.insert(name.clone(), rule);
        self.levels.insert(name, default_level);
    }

    /// Get a rule by name
    pub fn get(&self, name: &str) -> Option<&dyn Rule> {
        self.rules.get(name).map(|b| b.as_ref())
    }

    /// Get the configured level for a rule
    pub fn get_level(&self, name: &str) -> Option<RuleLevel> {
        self.levels.get(name).copied()
    }

    /// Set the level for a rule
    pub fn set_level(&mut self, name: &str, level: RuleLevel) {
        if self.rules.contains_key(name) {
            self.levels.insert(name.to_string(), level);
        }
    }

    /// Get all rule names
    pub fn rule_names(&self) -> Vec<&str> {
        self.rules.keys().map(|s| s.as_str()).collect()
    }

    /// Run all enabled rules on the given context
    pub fn check_all(&self, context: &LintContext) -> Vec<LintProblem> {
        let mut problems = Vec::new();

        for (name, rule) in &self.rules {
            let level = self.levels.get(name).copied().unwrap_or(RuleLevel::Error);

            if level == RuleLevel::Disable {
                continue;
            }

            let rule_problems = rule.check(context);
            problems.extend(rule_problems.into_iter().map(|mut p| {
                // Override problem level based on configuration
                p.level = match level {
                    RuleLevel::Error => crate::problem::LintLevel::Error,
                    RuleLevel::Warning => crate::problem::LintLevel::Warning,
                    RuleLevel::Disable => unreachable!(),
                };
                p
            }));
        }

        // Sort problems by line/column
        problems.sort();
        problems
    }
}

impl Default for RuleRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = RuleRegistry::with_defaults();
        assert!(!registry.rule_names().is_empty());
    }

    #[test]
    fn test_rule_level_override() {
        let mut registry = RuleRegistry::with_defaults();
        let rule_name = registry.rule_names()[0].to_string();

        registry.set_level(&rule_name, RuleLevel::Warning);
        assert_eq!(registry.get_level(&rule_name), Some(RuleLevel::Warning));
    }
}
