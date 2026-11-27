//! Spacing for flow mappings and sequences.

use super::{LintContext, tokens};
use crate::LintProblem;
use yaml_rust2::scanner::{Scanner, Token, TokenType};

/// Whether flow collections are allowed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FlowPolicy {
    /// Allow all flow collections.
    #[default]
    Allow,
    /// Forbid all flow collections.
    Forbid,
    /// Allow only empty flow collections.
    EmptyOnly,
}

#[derive(Debug, Default)]
pub(super) struct FlowSpacing {
    pub policy: FlowPolicy,
    pub min_spaces_inside: usize,
    pub max_spaces_inside: usize,
    pub min_spaces_inside_empty: Option<usize>,
    pub max_spaces_inside_empty: Option<usize>,
}

impl FlowSpacing {
    pub(super) fn check(
        &self,
        context: &LintContext,
        sequence: bool,
        collection_name: &str,
        forbidden_message: &str,
        rule: &'static str,
    ) -> Vec<LintProblem> {
        use TokenType::*;
        let (open, close, delimiter) = if sequence {
            (FlowSequenceStart, FlowSequenceEnd, '[')
        } else {
            (FlowMappingStart, FlowMappingEnd, '{')
        };
        if !context.content.contains(delimiter) {
            return Vec::new();
        }

        let mut scanner = Scanner::new(context.content.chars()).peekable();
        let mut previous = None;
        let mut problems = Vec::new();
        while let Some(Token(mark, token)) = scanner.next() {
            let mut min = self.min_spaces_inside;
            let mut max = self.max_spaces_inside;
            let mut empty = false;
            let mut boundary = None;
            if token == open {
                empty = scanner.peek().is_some_and(|Token(_, next)| *next == close);
                if self.policy == FlowPolicy::Forbid
                    || (self.policy == FlowPolicy::EmptyOnly && !empty)
                {
                    problems.push(LintProblem::new(
                        mark.line(),
                        mark.col() + 2,
                        forbidden_message,
                        rule,
                        crate::LintLevel::Error,
                    ));
                } else {
                    if empty {
                        min = self.min_spaces_inside_empty.unwrap_or(min);
                        max = self.max_spaces_inside_empty.unwrap_or(max);
                    }
                    if let Some(Token(next, _)) = scanner.peek()
                        && next.line() == mark.line()
                        && let Some(spaces) = next.col().checked_sub(mark.col() + 1)
                    {
                        boundary = Some((next.col(), spaces));
                    }
                }
            } else if token == close
                && previous.as_ref().is_some_and(|token| *token != open)
                && let Some(line) = context.lines.get(mark.line() - 1)
            {
                let spaces = tokens::spaces_before(line, mark.col());
                if spaces < mark.col() {
                    boundary = Some((mark.col(), spaces));
                }
            }

            if let Some((column, spaces)) = boundary {
                let violation = if spaces > max {
                    Some((column, "too many"))
                } else if spaces < min {
                    Some((column + 1, "too few"))
                } else {
                    None
                };
                if let Some((column, violation)) = violation {
                    problems.push(LintProblem::new(
                        mark.line(),
                        column.max(1),
                        format!(
                            "{violation} spaces inside {}{collection_name}",
                            if empty { "empty " } else { "" }
                        ),
                        rule,
                        crate::LintLevel::Error,
                    ));
                }
            }
            previous = Some(token);
        }
        problems
    }
}

/// Configuration for a flow collection spacing rule
#[derive(Debug)]
pub struct FlowCollectionConfig {
    /// Forbid this flow collection type entirely
    pub forbid: bool,
    /// Minimum spaces inside the collection delimiters
    pub min_spaces_inside: usize,
    /// Maximum spaces inside the collection delimiters
    pub max_spaces_inside: usize,
    /// Minimum spaces inside empty collections (None means use min_spaces_inside)
    pub min_spaces_inside_empty: Option<usize>,
    /// Maximum spaces inside empty collections (None means use max_spaces_inside)
    pub max_spaces_inside_empty: Option<usize>,
}

impl FlowCollectionConfig {
    /// Get effective min spaces for empty collections
    pub fn effective_min_empty(&self) -> usize {
        self.min_spaces_inside_empty
            .unwrap_or(self.min_spaces_inside)
    }

    /// Get effective max spaces for empty collections
    pub fn effective_max_empty(&self) -> usize {
        self.max_spaces_inside_empty
            .unwrap_or(self.max_spaces_inside)
    }
}

/// Tracks quote state while scanning YAML content
#[derive(Debug, Default)]
pub struct QuoteState {
    pub in_single_quote: bool,
    pub in_double_quote: bool,
}

impl QuoteState {
    /// Process a character and update quote state.
    /// Returns `true` if the character was a quote toggle (caller should skip further processing).
    pub fn process_char(&mut self, ch: char, prev_char: Option<char>) -> bool {
        if ch == '\'' && !self.in_double_quote {
            self.in_single_quote = !self.in_single_quote;
            return true;
        }
        if ch == '"' && !self.in_single_quote {
            if prev_char == Some('\\') {
                return true;
            }
            self.in_double_quote = !self.in_double_quote;
            return true;
        }
        false
    }

    /// Returns true if currently inside any quote
    pub fn in_quotes(&self) -> bool {
        self.in_single_quote || self.in_double_quote
    }
}

/// Find the matching closing delimiter on the same line, respecting quotes and nesting.
/// Returns the position of the closing char if found and content between is only spaces.
pub fn find_closing_char(
    chars: &[char],
    start: usize,
    open_char: char,
    close_char: char,
) -> Option<usize> {
    let mut j = start;
    let mut quotes = QuoteState::default();
    let mut depth = 0;

    while j < chars.len() {
        let c = chars[j];
        let prev = if j > 0 { Some(chars[j - 1]) } else { None };

        if quotes.process_char(c, prev) {
            j += 1;
            continue;
        }

        if !quotes.in_quotes() {
            if c == open_char {
                depth += 1;
            } else if c == close_char {
                if depth == 0 {
                    return Some(j);
                }
                depth -= 1;
            } else if c != ' ' {
                return None;
            }
        }
        j += 1;
    }
    None
}

/// Check flow collection delimiters for spacing problems.
pub fn check_flow_collection(
    context: &LintContext,
    config: &FlowCollectionConfig,
    open_char: char,
    close_char: char,
    collection_name: &str,
    forbidden_name: &str,
    rule_name: &'static str,
) -> Vec<LintProblem> {
    let sequence = match (open_char, close_char) {
        ('[', ']') => true,
        ('{', '}') => false,
        _ => return Vec::new(),
    };
    FlowSpacing {
        policy: if config.forbid {
            FlowPolicy::Forbid
        } else {
            FlowPolicy::Allow
        },
        min_spaces_inside: config.min_spaces_inside,
        max_spaces_inside: config.max_spaces_inside,
        min_spaces_inside_empty: config.min_spaces_inside_empty,
        max_spaces_inside_empty: config.max_spaces_inside_empty,
    }
    .check(
        context,
        sequence,
        collection_name,
        &format!("{forbidden_name} is forbidden"),
        rule_name,
    )
}
