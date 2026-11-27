//! Indentation rule - validates consistent indentation

use super::{directives::non_comment_ranges, tokens::scan};
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
    pub(crate) check_multi_line_strings: bool,
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
        let mut flow_lines = HashSet::new();
        let block_width = check_block_indentation(
            context,
            &tokens,
            fixed,
            self.sequences,
            &mut flow_lines,
            &mut structural,
        );
        flow_lines.extend(check_flows(context, &tokens, block_width, &mut structural));
        let (scalar_lines, scalar_problems) =
            check_scalars(context, &tokens, fixed, self.check_multi_line_strings);
        structural.retain(|problem| !scalar_lines.contains(&problem.line));
        structural.extend(scalar_problems);
        flow_lines.extend(scalar_lines);
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
            check_multi_line_strings: false,
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

fn check_scalars(
    context: &LintContext,
    tokens: &[Token],
    fixed: Option<usize>,
    enabled: bool,
) -> (HashSet<usize>, Vec<LintProblem>) {
    use TScalarStyle::*;

    let mut lines = HashSet::new();
    let mut problems = Vec::new();
    let source = &context.content;
    let Some(ranges) = non_comment_ranges(source) else {
        return (lines, problems);
    };
    let offsets: Option<Vec<_>> =
        (!source.is_ascii()).then(|| source.char_indices().map(|(offset, _)| offset).collect());
    let mut width = fixed;
    for (index, Token(mark, token)) in tokens.iter().enumerate() {
        let Scalar(style, value) = token else {
            continue;
        };
        if value.is_empty() {
            continue;
        }
        let start = if let Some(offsets) = &offsets {
            let Some(&start) = offsets.get(mark.index()) else {
                continue;
            };
            start
        } else {
            mark.index()
        };
        let Ok(range) = ranges.binary_search_by_key(&start, |range| range.start) else {
            continue;
        };
        let block = matches!(style, Literal | Folded);
        let expected = if block {
            if enabled {
                let base = scalar_header_indent(context, &tokens[..index], mark.line());
                let step = width.get_or_insert(mark.col().saturating_sub(base));
                base.saturating_add(*step)
            } else {
                mark.col()
            }
        } else {
            mark.col() + usize::from(matches!(style, SingleQuoted | DoubleQuoted))
        };

        for (offset, line) in source[ranges[range].clone()].lines().enumerate() {
            if (!block && offset == 0) || line.trim_matches([' ', '\t']).is_empty() {
                continue;
            }
            let line_number = mark.line() + offset;
            lines.insert(line_number);
            let found = if offset == 0 {
                mark.col()
            } else {
                line.bytes().take_while(|&byte| byte == b' ').count()
            };
            if enabled && found != expected {
                problems.push(LintProblem::new(
                    line_number,
                    found + 1,
                    format!("wrong indentation: expected {expected} but found {found}"),
                    "indentation",
                    LintLevel::Error,
                ));
            }
        }
    }
    (lines, problems)
}

fn scalar_header_indent(context: &LintContext, preceding: &[Token], body_line: usize) -> usize {
    let Some((index, header)) = context.lines
        [..body_line.saturating_sub(1).min(context.lines.len())]
        .iter()
        .enumerate()
        .rev()
        .find(|(_, line)| {
            let content = line.trim_start_matches(' ');
            !content.is_empty() && !content.starts_with('#')
        })
    else {
        return 0;
    };
    let line_indent = header.len() - header.trim_start_matches(' ').len();
    let header_line = index + 1;
    let Some(Token(mark, kind)) = preceding
        .iter()
        .rev()
        .find(|Token(_, kind)| matches!(kind, Value | Key | BlockEntry))
        .filter(|Token(mark, _)| mark.line() == header_line)
    else {
        return line_indent;
    };
    let explicit = header.chars().nth(mark.col()) == Some('?')
        || matches!(kind, Value) && mark.col() == line_indent;
    if explicit || matches!(kind, BlockEntry) {
        if matches!(kind, BlockEntry) {
            return mark.col();
        }
        return header
            .chars()
            .enumerate()
            .skip(mark.col() + 1)
            .find(|(_, ch)| matches!(ch, '|' | '>'))
            .map_or(line_indent, |(column, _)| column);
    }
    preceding
        .iter()
        .rev()
        .take_while(|Token(mark, _)| mark.line() >= header_line)
        .find(|Token(mark, kind)| mark.line() == header_line && matches!(kind, Key))
        .map_or(line_indent, |Token(key, _)| key.col())
}

fn check_block_indentation(
    context: &LintContext,
    tokens: &[Token],
    fixed: Option<usize>,
    mut policy: IndentSequences,
    checked_lines: &mut HashSet<usize>,
    problems: &mut Vec<LintProblem>,
) -> Option<(usize, usize)> {
    use IndentSequences::*;

    let mut collections = Vec::new();
    let mut width = fixed.map(|spaces| (0, spaces));
    let mut root_pending = true;
    let mut explicit_start_line = None;
    for (index, Token(mark, token)) in tokens.iter().enumerate() {
        match token {
            StreamStart(_) | VersionDirective(..) | TagDirective(..) | StreamEnd => continue,
            DocumentStart | DocumentEnd => {
                root_pending = true;
                explicit_start_line = matches!(token, DocumentStart).then_some(mark.line());
                continue;
            }
            _ => {}
        }
        if root_pending {
            let root = if matches!(token, BlockMappingStart) {
                tokens
                    .get(index + 1)
                    .filter(|Token(_, kind)| matches!(kind, Key))
                    .map_or(mark, |Token(key, _)| key)
            } else {
                mark
            };
            root_pending = false;
            if Some(root.line()) != explicit_start_line
                && !matches!(
                    token,
                    Scalar(TScalarStyle::Literal | TScalarStyle::Folded, _)
                )
                && root.col() > 0
                && context.lines.get(root.line() - 1).is_some_and(|line| {
                    line.bytes().take_while(|&byte| byte == b' ').count() == root.col()
                })
            {
                problems.push(indentation_problem(root.line(), root.col(), 0));
            }
        }
        match token {
            BlockMappingStart => {
                let key = tokens
                    .get(index + 1)
                    .filter(|Token(_, kind)| matches!(kind, Key))
                    .map_or(*mark, |Token(key, _)| *key);
                collections.push(Some(key));
            }
            FlowMappingStart | FlowSequenceStart | BlockSequenceStart => collections.push(None),
            BlockEnd | FlowMappingEnd | FlowSequenceEnd => {
                collections.pop();
            }
            Key => {
                if let Some(Some(previous)) = collections.last_mut() {
                    if previous.col() == mark.col() {
                        checked_lines.insert(mark.line());
                    }
                    *previous = *mark;
                    if context
                        .lines
                        .get(mark.line() - 1)
                        .and_then(|line| line.chars().nth(mark.col()))
                        == Some('?')
                        && let Some(Token(child, kind)) = tokens.get(index + 1)
                        && child.line() > mark.line()
                        && child.col() > mark.col()
                        && !matches!(
                            kind,
                            Value
                                | BlockEnd
                                | Key
                                | StreamEnd
                                | Scalar(TScalarStyle::Literal | TScalarStyle::Folded, _)
                        )
                    {
                        let step = width
                            .get_or_insert((child.line(), child.col() - mark.col()))
                            .1;
                        let expected = mark.col().saturating_add(step);
                        checked_lines.insert(child.line());
                        if child.col() != expected {
                            problems.push(indentation_problem(child.line(), child.col(), expected));
                        }
                    }
                }
            }
            Value => {
                let Some(&Some(key)) = collections.last() else {
                    continue;
                };
                let mapping_column = key.col();
                // An explicit key can end with a colon on its own line.
                if mark.line() > key.line()
                    && context
                        .lines
                        .get(key.line() - 1)
                        .and_then(|line| line.chars().nth(mapping_column))
                        == Some('?')
                    && context.lines.get(mark.line() - 1).is_some_and(|line| {
                        line.bytes().take_while(|&byte| byte == b' ').count() == mark.col()
                    })
                {
                    checked_lines.insert(mark.line());
                    if mark.col() != mapping_column {
                        problems.push(LintProblem::new(
                            mark.line(),
                            mark.col() + 1,
                            format!(
                                "wrong indentation: expected {mapping_column} but found {}",
                                mark.col()
                            ),
                            "indentation",
                            LintLevel::Error,
                        ));
                    }
                }
                let mut values = tokens[index + 1..].iter().peekable();
                while values.peek().is_some_and(|Token(property, kind)| {
                    property.line() == mark.line() && matches!(kind, Anchor(_) | Tag(..))
                }) {
                    values.next();
                }
                let Some(Token(location, kind)) = values.next() else {
                    continue;
                };
                let mut value = *location;
                let sequence = matches!(kind, BlockSequenceStart | BlockEntry);
                if matches!(kind, BlockMappingStart)
                    && let Some(Token(key, Key)) = values.peek()
                {
                    value = *key;
                }
                if value.line() <= mark.line()
                    || matches!(
                        kind,
                        BlockEnd
                            | DocumentEnd
                            | DocumentStart
                            | Key
                            | StreamEnd
                            | Scalar(TScalarStyle::Literal | TScalarStyle::Folded, _)
                    )
                {
                    continue;
                }
                // An indentless sequence has no start token. Its entry marker
                // points after the dash and any separating whitespace.
                let column = if matches!(kind, BlockEntry) {
                    let Some(line) = context.lines.get(value.line() - 1) else {
                        continue;
                    };
                    line.bytes().take_while(|&byte| byte == b' ').count()
                } else {
                    value.col()
                };
                if !sequence {
                    if column > mapping_column {
                        let expected = mapping_column.saturating_add(
                            width
                                .get_or_insert((value.line(), column - mapping_column))
                                .1,
                        );
                        checked_lines.insert(value.line());
                        if column != expected {
                            problems.push(indentation_problem(value.line(), column, expected));
                        }
                    }
                    continue;
                }
                let indented = column > mapping_column;
                if policy == Consistent {
                    policy = if indented { Indented } else { Unindented };
                }
                if indented && policy != Unindented {
                    width.get_or_insert((value.line(), column - mapping_column));
                }
                checked_lines.insert(value.line());
                let expected = match (policy, width.map(|(_, spaces)| spaces), indented) {
                    (Unindented, _, true) => Some(mapping_column),
                    (Indented, Some(spaces), _) | (Whatever, Some(spaces), true) => {
                        Some(mapping_column.saturating_add(spaces))
                    }
                    _ => None,
                };
                let message =
                    if let Some(expected) = expected.filter(|&expected| expected != column) {
                        format!("wrong indentation: expected {expected} but found {column}")
                    } else if policy == Indented && !indented && width.is_none() {
                        format!(
                            "wrong indentation: expected at least {}",
                            mapping_column + 1
                        )
                    } else {
                        continue;
                    };
                problems.push(LintProblem::new(
                    value.line(),
                    column + 1,
                    message,
                    "indentation",
                    LintLevel::Error,
                ));
            }
            BlockEntry => {
                let Some(line) = context.lines.get(mark.line() - 1) else {
                    continue;
                };
                let before_entry: String = line.chars().take(mark.col()).collect();
                let before_dash = before_entry
                    .split_once('#')
                    .map_or(before_entry.as_str(), |(source, _)| source)
                    .trim_end_matches([' ', '\t']);
                let Some(prefix) = before_dash.strip_suffix('-') else {
                    continue;
                };
                let column = prefix.chars().count();
                if prefix.bytes().all(|byte| byte == b' ') {
                    checked_lines.insert(mark.line());
                }
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
                        BlockEnd | BlockEntry | DocumentEnd | DocumentStart | StreamEnd
                    )
                {
                    continue;
                }
                let (child_line, child_column) = if matches!(
                    kind,
                    Scalar(TScalarStyle::Literal | TScalarStyle::Folded, _)
                ) {
                    if matches!(line.chars().nth(mark.col()), Some('|' | '>')) {
                        continue;
                    }
                    // yaml-rust2 marks the scalar body rather than the header.
                    let Some((offset, header)) = context.lines[mark.line()..]
                        .iter()
                        .take(child.line() - mark.line())
                        .enumerate()
                        .find(|(_, line)| {
                            let text = line.trim_start();
                            !text.is_empty() && !text.starts_with('#')
                        })
                        .filter(|(_, line)| line.trim_start().starts_with(['|', '>']))
                    else {
                        continue;
                    };
                    (
                        mark.line() + offset + 1,
                        header.len() - header.trim_start().len(),
                    )
                } else {
                    (child.line(), child.col())
                };
                if child_column > column {
                    let expected = column
                        .saturating_add(width.get_or_insert((child_line, child_column - column)).1);
                    if child_column != expected {
                        problems.push(indentation_problem(child_line, child_column, expected));
                    }
                }
            }
            _ => {}
        }
    }
    width
}

fn indentation_problem(line: usize, found: usize, expected: usize) -> LintProblem {
    LintProblem::new(
        line,
        found + 1,
        format!("wrong indentation: expected {expected} but found {found}"),
        "indentation",
        LintLevel::Error,
    )
}

struct FlowIndent {
    opening_line: usize,
    opening_indent: usize,
    content_indent: Option<usize>,
}

fn check_flows(
    context: &LintContext,
    tokens: &[Token],
    block_width: Option<(usize, usize)>,
    problems: &mut Vec<LintProblem>,
) -> HashSet<usize> {
    let mut flows: Vec<Option<FlowIndent>> = Vec::new();
    let mut checked_lines = HashSet::new();
    let mut last_line = 0;
    let mut flow_width = None;
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
                        let width = block_width
                            .filter(|(first, _)| *first <= line)
                            .map(|(_, spaces)| spaces);
                        if let Some(spaces) = flow_width.or(width) {
                            flow.opening_indent.saturating_add(spaces)
                        } else {
                            flow_width = mark
                                .col()
                                .checked_sub(flow.opening_indent)
                                .filter(|spaces| *spaces > 0);
                            mark.col()
                        }
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
    fn checks_block_scalar_header_after_bare_sequence_entry() {
        for indicator in ['|', '>'] {
            let yaml = format!("---\n-\n  -\n     {indicator}\n      multi\n      line\n...\n");
            let rule = IndentationRule::new();
            let problems = rule.check(&LintContext::new(yaml));
            assert_eq!(problems, vec![indentation_problem(4, 5, 4)]);

            let yaml = format!("---\n-\n  -\n    {indicator}\n      multi\n      line\n...\n");
            assert!(rule.check(&LintContext::new(yaml)).is_empty());
        }
    }

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
