//! Implicit null values in collections.

use super::{
    LintContext, Rule,
    tokens::{problem, scan},
};
use crate::problem::LintProblem;
use serde::Deserialize;
use yaml_rust2::scanner::{Token, TokenType::*};

/// Controls which collections permit implicit null values.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct EmptyValuesRule {
    forbid_in_block_mappings: bool,
    forbid_in_flow_mappings: bool,
    forbid_in_block_sequences: bool,
}

impl Default for EmptyValuesRule {
    fn default() -> Self {
        Self {
            forbid_in_block_mappings: true,
            forbid_in_flow_mappings: true,
            forbid_in_block_sequences: true,
        }
    }
}

impl Rule for EmptyValuesRule {
    fn name(&self) -> &'static str {
        "empty-values"
    }

    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        let tokens = scan(&context.content);
        let mut flow_depth: usize = 0;
        let mut problems = Vec::new();
        for pair in tokens.windows(2) {
            let Token(mark, token) = &pair[0];
            let Token(_, next) = &pair[1];
            let message = match token {
                FlowSequenceStart | FlowMappingStart => {
                    flow_depth += 1;
                    None
                }
                FlowSequenceEnd | FlowMappingEnd => {
                    flow_depth = flow_depth.saturating_sub(1);
                    None
                }
                Value
                    if flow_depth > 0
                        && self.forbid_in_flow_mappings
                        && matches!(next, FlowEntry | FlowMappingEnd | FlowSequenceEnd) =>
                {
                    Some("empty value in flow mapping")
                }
                Value
                    if flow_depth == 0
                        && self.forbid_in_block_mappings
                        && matches!(
                            next,
                            Key | BlockEnd | DocumentEnd | DocumentStart | StreamEnd
                        ) =>
                {
                    Some("empty value in block mapping")
                }
                BlockEntry
                    if self.forbid_in_block_sequences
                        && matches!(
                            next,
                            Key | BlockEntry | BlockEnd | DocumentEnd | DocumentStart | StreamEnd
                        ) =>
                {
                    Some("empty value in block sequence")
                }
                _ => None,
            };
            if let Some(message) = message {
                let mut diagnostic = problem(*mark, self.name(), message);
                if matches!(token, Value) {
                    diagnostic.column += 1;
                }
                problems.push(diagnostic);
            }
        }
        problems
    }
}
