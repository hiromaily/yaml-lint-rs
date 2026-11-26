//! Line-level suppressions embedded in YAML comments.

use crate::problem::LintProblem;
use std::{collections::HashSet, ops::Range};
use yaml_rust2::scanner::{Scanner, TScalarStyle, Token, TokenType};

#[derive(Clone, Copy)]
pub(super) struct Directive<'a> {
    pub(super) target: usize,
    pub(super) rule: &'a str,
    pub(super) standalone: bool,
}

pub(super) fn suppress(content: &str, problems: &mut Vec<LintProblem>) {
    if problems.is_empty() {
        return;
    }
    let suppressed: HashSet<_> = collect(content)
        .map(|directive| (directive.target, directive.rule))
        .collect();
    problems.retain(|problem| !suppressed.contains(&(problem.line, problem.rule.as_str())));
}

pub(super) fn lines_for_rule(content: &str, rule: &str) -> HashSet<usize> {
    collect(content)
        .filter(|directive| directive.rule == rule)
        .map(|directive| directive.target)
        .collect()
}

pub(super) fn collect(content: &str) -> impl Iterator<Item = Directive<'_>> {
    let mut excluded = content
        .contains("disable-line")
        .then(|| non_comment_ranges(content))
        .flatten()
        .map(|ranges| ranges.into_iter().peekable());
    let mut offset = 0;
    let lines = if excluded.is_some() { content } else { "" };
    lines
        .split_inclusive('\n')
        .enumerate()
        .filter_map(move |(index, line)| {
            let start = offset;
            offset += line.len();
            let excluded = excluded.as_mut()?;
            let (hash, _) = line.match_indices('#').find(|&(hash, _)| {
                let position = start + hash;
                while excluded.peek().is_some_and(|range| range.end <= position) {
                    excluded.next();
                }
                !excluded
                    .peek()
                    .is_some_and(|range| range.contains(&position))
            })?;
            let mut words = line[hash + 1..].split_ascii_whitespace();
            if words.next()? != "yamllint" || words.next()? != "disable-line" {
                return None;
            }
            let rule = words.next()?.strip_prefix("rule:")?;
            if rule.is_empty() || words.next().is_some() {
                return None;
            }
            let standalone = line[..hash].trim().is_empty();
            Some(Directive {
                target: index + if standalone { 2 } else { 1 },
                rule,
                standalone,
            })
        })
}

/// Map tokens that can contain a hash to byte ranges. The scanner discards
/// comments, but its cursor after fetching each token gives us the token's
/// upper bound, even when emitting a token requires looking ahead.
fn non_comment_ranges(content: &str) -> Option<Vec<Range<usize>>> {
    let offsets: Option<Vec<_>> = (!content.is_ascii()).then(|| {
        content
            .char_indices()
            .map(|(offset, _)| offset)
            .chain([content.len()])
            .collect()
    });
    let char_count = offsets.as_ref().map_or(content.len(), |map| map.len() - 1);
    let byte_offset = |index| offsets.as_ref().map_or(index, |map| map[index]);
    let mut scanner = Scanner::new(content.chars());
    let mut ends = Vec::new();
    scanner.fetch_next_token().ok()?; // Stream start does not consume input.
    let mut cursor = 0;
    while cursor < char_count {
        scanner.fetch_next_token().ok()?;
        let end = scanner.mark().index();
        if end == cursor {
            return None;
        }
        ends.push(end);
        cursor = end;
    }
    // Fetch the stream-end token if the last fetch stopped at the end of a
    // scalar. Reaching end of input can also reveal a missing simple key.
    scanner.fetch_next_token().ok()?;

    let mut ends = ends.into_iter().peekable();
    let mut ranges = Vec::new();
    while let Some(Token(mark, token)) = scanner.next_token().ok()? {
        match &token {
            // An empty block scalar may be marked at its header, where hashes
            // are comments. Its content can be skipped altogether.
            TokenType::Scalar(TScalarStyle::Literal | TScalarStyle::Folded, value)
                if value.trim().is_empty() =>
            {
                continue;
            }
            TokenType::Scalar(..)
            | TokenType::Tag(..)
            | TokenType::Anchor(_)
            | TokenType::Alias(_)
            | TokenType::TagDirective(..) => {}
            _ => continue,
        }
        let start = mark.index();
        while ends.peek().is_some_and(|&end| end <= start) {
            ends.next();
        }
        let Some(&end) = ends.peek() else {
            break;
        };
        let start = byte_offset(start);
        let end = byte_offset(end);
        let source = &content[start..end];
        let range = match token {
            TokenType::Scalar(TScalarStyle::SingleQuoted | TScalarStyle::DoubleQuoted, _) => {
                start..start + quoted_len(source)
            }
            // Nonempty block scalar tokens are marked at their first
            // content character, after any header comment.
            TokenType::Scalar(TScalarStyle::Literal | TScalarStyle::Folded, _) => start..end,
            TokenType::Scalar(TScalarStyle::Plain, _) => start..start + plain_len(source),
            TokenType::Tag(..) | TokenType::Anchor(_) | TokenType::Alias(_) => start..end,
            TokenType::TagDirective(..)
                if source.split_ascii_whitespace().next() == Some("%TAG") =>
            {
                // A tag prefix can contain a hash.
                let prefix = source.split_ascii_whitespace().nth(2)?;
                let prefix_end = prefix.as_ptr() as usize - source.as_ptr() as usize + prefix.len();
                start..start + prefix_end
            }
            TokenType::TagDirective(..) => start..end,
            _ => continue,
        };
        ranges.push(range);
    }
    Some(ranges)
}

fn quoted_len(source: &str) -> usize {
    let mut bytes = source.bytes().enumerate();
    let Some((_, quote)) = bytes.next() else {
        return 0;
    };
    let mut bytes = bytes.peekable();
    while let Some((index, byte)) = bytes.next() {
        if quote == b'"' && byte == b'\\'
            || quote == b'\''
                && byte == quote
                && bytes.peek().is_some_and(|&(_, next)| next == quote)
        {
            bytes.next();
        } else if byte == quote {
            return index + 1;
        }
    }
    source.len()
}

fn plain_len(source: &str) -> usize {
    // During a multiline plain scalar, the scanner can consume a comment
    // after a tab when the line's indentation falls short of the scalar.
    if let Some(line_break) = source.rfind(['\r', '\n']) {
        let tail = &source[line_break + 1..];
        let text = tail.trim_start_matches([' ', '\t']);
        let indent = &tail[..tail.len() - text.len()];
        if text.starts_with('#') && indent.contains('\t') {
            return line_break + 1;
        }
    }
    source.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distinguishes_comment_hashes_from_token_content() {
        let yaml = concat!(
            "%TAG !e! tag:example.com,2026:prefix#suffix # yamllint disable-line rule:demo\r\n",
            "---\r\n",
            "quoted: '日本語 # it''s quoted' # yamllint disable-line rule:demo\r\n",
            "double: \"escaped \\\" # still quoted\" # yamllint disable-line rule:demo\r\n",
            "tagged: !e!prefix#suffix value # yamllint disable-line rule:demo\r\n",
            "block: >2- # yamllint disable-line rule:demo\r\n",
            "  | text\r\n",
            "  # yamllint disable-line rule:demo\r\n",
            "flow: [value, # yamllint disable-line rule:demo\r\n",
            "  another]\r\n",
            "empty: |- # yamllint disable-line rule:demo",
        );
        assert_eq!(
            lines_for_rule(yaml, "demo"),
            HashSet::from([1, 3, 4, 5, 6, 9, 11])
        );
    }

    #[test]
    fn scanner_errors_leave_directives_inactive() {
        let yaml = "ok: value # yamllint disable-line rule:demo\nbad: 'unterminated\n";
        assert!(lines_for_rule(yaml, "demo").is_empty());
    }
}
