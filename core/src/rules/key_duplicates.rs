//! Detection of duplicate scalar keys in YAML mappings.

use super::{
    LintContext, Rule,
    tokens::{problem, scan},
};
use crate::problem::LintProblem;
use serde::Deserialize;
use std::collections::HashSet;
use yaml_rust2::scanner::{TScalarStyle, Token, TokenType::*};

/// Rule that detects duplicate keys in mappings.
#[derive(Debug)]
pub struct KeyDuplicatesRule;

impl Rule for KeyDuplicatesRule {
    fn name(&self) -> &'static str {
        "key-duplicates"
    }

    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        KeyDuplicatesOptions::default().check(context)
    }
}

/// Configured duplicate key rule.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct KeyDuplicatesOptions {
    forbid_duplicated_merge_keys: bool,
}

impl Rule for KeyDuplicatesOptions {
    fn name(&self) -> &'static str {
        KeyDuplicatesRule.name()
    }

    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        let tokens = scan(&context.content);
        let mut collections = Vec::new();
        let mut problems = Vec::new();

        for (index, Token(key_mark, token)) in tokens.iter().enumerate() {
            match token {
                BlockMappingStart | FlowMappingStart => collections.push(Some(HashSet::new())),
                BlockSequenceStart | FlowSequenceStart => collections.push(None),
                BlockEnd | FlowMappingEnd | FlowSequenceEnd => {
                    collections.pop();
                }
                Key => {
                    let Some(Some(keys)) = collections.last_mut() else {
                        continue;
                    };
                    let Some(Token(mark, Scalar(style, key))) = tokens.get(index + 1) else {
                        continue;
                    };
                    if (key != "<<" || self.forbid_duplicated_merge_keys)
                        && !keys.insert(key.as_str())
                    {
                        let mut diagnostic = problem(
                            *mark,
                            self.name(),
                            format!("duplication of key \"{key}\" in mapping"),
                        );
                        if matches!(style, TScalarStyle::Literal | TScalarStyle::Folded) {
                            // The scanner points to the scalar body. Locate its
                            // header after the explicit key marker.
                            for (index, line) in context
                                .lines
                                .iter()
                                .enumerate()
                                .take(mark.line())
                                .skip(key_mark.line() - 1)
                            {
                                let start = if index + 1 == key_mark.line() {
                                    key_mark.col() + 1
                                } else {
                                    0
                                };
                                if let Some((column, ch)) = line
                                    .chars()
                                    .enumerate()
                                    .skip(start)
                                    .find(|(_, ch)| !matches!(ch, ' ' | '\t'))
                                    && matches!(ch, '|' | '>')
                                {
                                    diagnostic.line = index + 1;
                                    diagnostic.column = column + 1;
                                    break;
                                }
                            }
                        }
                        problems.push(diagnostic);
                    }
                }
                _ => {}
            }
        }

        problems
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_duplicates() {
        let yaml = "key1: value1\nkey2: value2\nkey3: value3\n";
        let context = LintContext::new(yaml.to_string());
        let rule = KeyDuplicatesRule;
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_simple_duplicate() {
        let yaml = "key: value1\nkey: value2\n";
        let context = LintContext::new(yaml.to_string());
        let rule = KeyDuplicatesRule;
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].line, 2);
        assert_eq!(problems[0].message, "duplication of key \"key\" in mapping");
    }

    #[test]
    fn test_duplicate_with_quotes() {
        let yaml = "\"key\": value1\n\"key\": value2\n";
        let context = LintContext::new(yaml.to_string());
        let rule = KeyDuplicatesRule;
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
    }

    #[test]
    fn points_to_the_header_of_a_duplicate_block_scalar_key() {
        let source = "duplicate key: 1\n? >-\n    duplicate\n    key\n: 2\n";
        let problems = KeyDuplicatesRule.check(&LintContext::new(source.into()));
        assert_eq!(problems.len(), 1);
        assert_eq!((problems[0].line, problems[0].column), (2, 3));
    }

    #[test]
    fn test_nested_no_duplicate() {
        let yaml = "parent:\n  key: value1\nanother:\n  key: value2\n";
        let context = LintContext::new(yaml.to_string());
        let rule = KeyDuplicatesRule;
        let problems = rule.check(&context);

        // Same key name in different scopes is OK
        assert!(problems.is_empty());
    }

    #[test]
    fn test_nested_duplicate() {
        let yaml = "parent:\n  key: value1\n  key: value2\n";
        let context = LintContext::new(yaml.to_string());
        let rule = KeyDuplicatesRule;
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].line, 3);
    }

    #[test]
    fn test_multiple_duplicates() {
        let yaml = "key1: value1\nkey1: value2\nkey2: value3\nkey2: value4\n";
        let context = LintContext::new(yaml.to_string());
        let rule = KeyDuplicatesRule;
        let problems = rule.check(&context);

        assert_eq!(problems.len(), 2);
    }

    #[test]
    fn test_comment_ignored() {
        let yaml = "key: value1\n# key: comment\nkey2: value2\n";
        let context = LintContext::new(yaml.to_string());
        let rule = KeyDuplicatesRule;
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_colon_in_string_ignored() {
        let yaml = "key: \"value:with:colon\"\nkey2: value\n";
        let context = LintContext::new(yaml.to_string());
        let rule = KeyDuplicatesRule;
        let problems = rule.check(&context);

        assert!(problems.is_empty());
    }

    #[test]
    fn test_list_items_different_scope() {
        let yaml = "list:\n  - key: value1\n  - key: value2\n";
        let context = LintContext::new(yaml.to_string());
        let rule = KeyDuplicatesRule;
        let problems = rule.check(&context);

        // Same key in different list items is OK
        assert!(problems.is_empty());
    }
}
