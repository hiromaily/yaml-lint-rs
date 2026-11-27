//! Indentation rule - validates consistent indentation

use super::tokens::scan;
use crate::problem::{LintLevel, LintProblem};
use crate::rules::{LintContext, Rule};
use std::collections::HashSet;
use yaml_rust2::scanner::{TScalarStyle, Token, TokenType::*};

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

/// Indentation policy for a block sequence used as a mapping value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndentSequences {
    /// Require an additional indentation level.
    Indented,
    /// Align sequence entries with their mapping key.
    Unindented,
    /// Allow either indentation style.
    Whatever,
    /// Use the same style for every sequence in the stream.
    Consistent,
}

#[derive(Debug)]
pub(crate) struct ConfiguredIndentation {
    pub(crate) rule: IndentationRule,
    pub(crate) sequences: IndentSequences,
}

impl Rule for ConfiguredIndentation {
    fn name(&self) -> &'static str {
        self.rule.name()
    }

    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        let mut problems = self.rule.check_lines(context);
        let fixed = match self.rule.spaces {
            IndentSpaces::Fixed(spaces) => Some(spaces),
            IndentSpaces::Consistent => None,
        };
        let tokens = scan(&context.content);
        let mut structural = Vec::new();
        let mut flow_lines = check_flows(context, &tokens, fixed, &mut structural);
        check_sequences(context, &tokens, fixed, self.sequences, &mut structural);
        flow_lines.extend(structural.iter().map(|problem| problem.line));
        problems.retain(|problem| !flow_lines.contains(&problem.line));
        problems.extend(structural);
        problems.sort();
        problems
    }
}

impl Rule for IndentationRule {
    fn name(&self) -> &'static str {
        "indentation"
    }

    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        ConfiguredIndentation {
            rule: Self {
                spaces: self.spaces,
            },
            sequences: IndentSequences::Indented,
        }
        .check(context)
    }
}

impl IndentationRule {
    fn check_lines(&self, context: &LintContext) -> Vec<LintProblem> {
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
}

fn check_sequences(
    context: &LintContext,
    tokens: &[Token],
    fixed: Option<usize>,
    mut policy: IndentSequences,
    problems: &mut Vec<LintProblem>,
) {
    use IndentSequences::*;

    let mut collections = Vec::new();
    let mut step = fixed;
    for (index, Token(mark, token)) in tokens.iter().enumerate() {
        match token {
            BlockMappingStart => collections.push(Some(mark.col())),
            FlowMappingStart | FlowSequenceStart | BlockSequenceStart => collections.push(None),
            BlockEnd | FlowMappingEnd | FlowSequenceEnd => {
                collections.pop();
            }
            Key => {
                if let Some(Some(column)) = collections.last_mut() {
                    *column = mark.col();
                }
            }
            Value => {
                let Some(&Some(mapping_column)) = collections.last() else {
                    continue;
                };
                let Some(Token(sequence, sequence_token)) = tokens[index + 1..]
                    .iter()
                    .find(|Token(_, token)| !matches!(token, Anchor(_) | Tag(..)))
                else {
                    continue;
                };
                if sequence.line() <= mark.line()
                    || !matches!(sequence_token, BlockSequenceStart | BlockEntry)
                {
                    continue;
                }
                // A start token points at the dash. For an indentless sequence
                // yaml-rust2 emits only a BlockEntry, whose marker follows it.
                let mut column = sequence.col();
                if matches!(sequence_token, BlockEntry) {
                    let Some(line) = context.lines.get(sequence.line() - 1) else {
                        continue;
                    };
                    column = line.bytes().take_while(|&byte| byte == b' ').count();
                }
                let indented = column > mapping_column;
                if policy == Consistent {
                    policy = if indented { Indented } else { Unindented };
                }
                let expected = match (policy, fixed, indented) {
                    (Unindented, _, true) => Some(mapping_column),
                    (Indented, Some(spaces), _) | (Whatever, Some(spaces), true) => {
                        Some(mapping_column.saturating_add(spaces))
                    }
                    _ => None,
                };
                let message =
                    if let Some(expected) = expected.filter(|&expected| expected != column) {
                        format!("wrong indentation: expected {expected} but found {column}")
                    } else if policy == Indented && !indented && fixed.is_none() {
                        format!(
                            "wrong indentation: expected at least {}",
                            mapping_column + 1
                        )
                    } else {
                        continue;
                    };
                problems.push(LintProblem::new(
                    sequence.line(),
                    column + 1,
                    message,
                    "indentation",
                    LintLevel::Error,
                ));
            }
            BlockEntry => {
                let Some(Token(child, kind)) = tokens.get(index + 1) else {
                    continue;
                };
                let (mut child, mut kind) = (*child, kind);
                if matches!(kind, BlockMappingStart)
                    && let Some(Token(key, key_kind @ Key)) = tokens.get(index + 2)
                {
                    child = *key;
                    kind = key_kind;
                }
                if child.line() <= mark.line()
                    || matches!(
                        kind,
                        BlockEnd
                            | BlockEntry
                            | DocumentEnd
                            | DocumentStart
                            | StreamEnd
                            | Scalar(TScalarStyle::Literal | TScalarStyle::Folded, _)
                    )
                {
                    continue;
                }
                let Some(line) = context.lines.get(mark.line() - 1) else {
                    continue;
                };
                let before_entry: String = line.chars().take(mark.col()).collect();
                let before_dash = before_entry.trim_end_matches([' ', '\t']);
                let Some(prefix) = before_dash.strip_suffix('-') else {
                    continue;
                };
                let column = prefix.chars().count();
                if child.col() > column {
                    let expected = column.saturating_add(*step.get_or_insert(child.col() - column));
                    if child.col() != expected {
                        problems.push(LintProblem::new(
                            child.line(),
                            child.col() + 1,
                            format!(
                                "wrong indentation: expected {expected} but found {}",
                                child.col()
                            ),
                            "indentation",
                            LintLevel::Error,
                        ));
                    }
                }
            }
            _ => {}
        }
    }
}

struct FlowIndent {
    opening_line: usize,
    opening_indent: usize,
    content_indent: Option<usize>,
}

fn check_flows(
    context: &LintContext,
    tokens: &[Token],
    fixed: Option<usize>,
    problems: &mut Vec<LintProblem>,
) -> HashSet<usize> {
    let mut flows: Vec<Option<FlowIndent>> = Vec::new();
    let mut checked_lines = HashSet::new();
    let mut last_line = 0;
    for Token(mark, token) in tokens {
        if matches!(token, StreamStart(_) | StreamEnd | BlockEnd) {
            continue;
        }
        let line = mark.line();
        let source = context.lines.get(line - 1);
        if matches!(token, FlowMappingStart)
            && source.and_then(|line| line.chars().nth(mark.col())) != Some('{')
        {
            // The scanner represents an implicit key/value pair in a flow
            // sequence as a mapping with no delimiters in the source.
            flows.push(None);
            continue;
        }
        let closing = matches!(token, FlowSequenceEnd | FlowMappingEnd);
        if matches!(token, FlowMappingEnd) && matches!(flows.last(), Some(None)) {
            flows.pop();
            continue;
        }
        if let Some(flow) = flows.iter_mut().rev().find_map(Option::as_mut) {
            if line > last_line {
                let expected = if closing {
                    flow.opening_indent
                } else {
                    *flow.content_indent.get_or_insert_with(|| {
                        fixed.map_or(mark.col(), |spaces| {
                            flow.opening_indent.saturating_add(spaces)
                        })
                    })
                };
                checked_lines.insert(line);
                if mark.col() != expected {
                    problems.push(LintProblem::new(
                        line,
                        mark.col() + 1,
                        format!(
                            "wrong indentation: expected {expected} but found {}",
                            mark.col()
                        ),
                        "indentation",
                        LintLevel::Error,
                    ));
                }
            } else if line == flow.opening_line && !closing {
                flow.content_indent.get_or_insert(mark.col());
            }
        }
        if closing {
            flows.pop();
        } else if matches!(token, FlowSequenceStart | FlowMappingStart)
            && let Some(source) = source
        {
            flows.push(Some(FlowIndent {
                opening_line: line,
                opening_indent: source.bytes().take_while(|&byte| byte == b' ').count(),
                content_indent: None,
            }));
        }
        last_line = last_line.max(line);
    }
    checked_lines
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
