//! Shared logic for flow collection rules (braces `{}` and brackets `[]`)

use crate::problem::{LintLevel, LintProblem};
use crate::rules::LintContext;

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

/// Check lines for flow collection spacing issues.
/// `open_char`/`close_char` define the delimiters (e.g. `{`/`}` or `[`/`]`).
/// `collection_name` is used in spacing error messages (e.g. "braces" or "brackets").
/// `forbidden_name` is used in the forbid message (e.g. "flow mapping (braces)").
/// `rule_name` is the rule identifier (e.g. "braces" or "brackets").
pub fn check_flow_collection(
    context: &LintContext,
    config: &FlowCollectionConfig,
    open_char: char,
    close_char: char,
    collection_name: &str,
    forbidden_name: &str,
    rule_name: &'static str,
) -> Vec<LintProblem> {
    let mut problems = Vec::new();

    for (line_idx, line) in context.lines.iter().enumerate() {
        let line_num = line_idx + 1;
        let chars: Vec<char> = line.chars().collect();
        let mut quotes = QuoteState::default();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];
            let prev = if i > 0 { Some(chars[i - 1]) } else { None };

            // Track quote state
            if quotes.process_char(ch, prev) {
                i += 1;
                continue;
            }

            // Skip if inside quotes
            if quotes.in_quotes() {
                i += 1;
                continue;
            }

            // Skip comments
            if ch == '#' {
                break;
            }

            if ch == open_char {
                if config.forbid {
                    problems.push(LintProblem::new(
                        line_num,
                        i + 1,
                        format!("{forbidden_name} is forbidden"),
                        rule_name,
                        LintLevel::Error,
                    ));
                    i += 1;
                    continue;
                }

                // Check if empty
                let after_open = i + 1;
                if let Some(close_pos) =
                    find_closing_char(&chars, after_open, open_char, close_char)
                {
                    let content_between = &chars[after_open..close_pos];
                    let is_empty = content_between.iter().all(|c| *c == ' ');

                    if is_empty {
                        let spaces = close_pos - after_open;
                        let min = config.effective_min_empty();
                        let max = config.effective_max_empty();

                        if spaces < min {
                            problems.push(LintProblem::new(
                                line_num,
                                after_open + 1,
                                format!(
                                    "too few spaces inside empty {collection_name} ({spaces} < {min})"
                                ),
                                rule_name,
                                LintLevel::Error,
                            ));
                        } else if spaces > max {
                            problems.push(LintProblem::new(
                                line_num,
                                after_open + 1,
                                format!(
                                    "too many spaces inside empty {collection_name} ({spaces} > {max})"
                                ),
                                rule_name,
                                LintLevel::Error,
                            ));
                        }
                        i = close_pos + 1;
                        continue;
                    }
                }

                // Non-empty: check spaces after opening
                let spaces_after = chars[after_open..]
                    .iter()
                    .take_while(|c| **c == ' ')
                    .count();

                if spaces_after < config.min_spaces_inside {
                    problems.push(LintProblem::new(
                        line_num,
                        after_open + 1,
                        format!(
                            "too few spaces inside {collection_name} ({} < {})",
                            spaces_after, config.min_spaces_inside
                        ),
                        rule_name,
                        LintLevel::Error,
                    ));
                } else if spaces_after > config.max_spaces_inside {
                    problems.push(LintProblem::new(
                        line_num,
                        after_open + 1,
                        format!(
                            "too many spaces inside {collection_name} ({} > {})",
                            spaces_after, config.max_spaces_inside
                        ),
                        rule_name,
                        LintLevel::Error,
                    ));
                }
            } else if ch == close_char && !config.forbid {
                // Check spaces before closing
                if i > 0 {
                    let spaces_before = chars[..i].iter().rev().take_while(|c| **c == ' ').count();

                    let prev_non_space = chars[..i].iter().rev().find(|c| **c != ' ');

                    if prev_non_space != Some(&open_char) {
                        if spaces_before < config.min_spaces_inside {
                            problems.push(LintProblem::new(
                                line_num,
                                i + 1,
                                format!(
                                    "too few spaces inside {collection_name} ({} < {})",
                                    spaces_before, config.min_spaces_inside
                                ),
                                rule_name,
                                LintLevel::Error,
                            ));
                        } else if spaces_before > config.max_spaces_inside {
                            problems.push(LintProblem::new(
                                line_num,
                                i + 1,
                                format!(
                                    "too many spaces inside {collection_name} ({} > {})",
                                    spaces_before, config.max_spaces_inside
                                ),
                                rule_name,
                                LintLevel::Error,
                            ));
                        }
                    }
                }
            }

            i += 1;
        }
    }

    problems
}
