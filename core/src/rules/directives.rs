//! Line-level suppressions embedded in YAML comments.

use crate::problem::LintProblem;
use std::collections::HashSet;
use yaml_rust2::scanner::{Scanner, Token};

pub(super) fn suppress(content: &str, problems: &mut Vec<LintProblem>) {
    if problems.is_empty() || !content.contains("disable-line") {
        return;
    }
    let candidates: HashSet<_> = problems.iter().map(|p| (p.line, p.rule.as_str())).collect();
    let suppressed = collect(content, |line, rule| candidates.contains(&(line, rule)));
    problems.retain(|problem| !suppressed.contains(&(problem.line, problem.rule.as_str())));
}

pub(super) fn lines_for_rule(content: &str, rule: &str) -> HashSet<usize> {
    collect(content, |_, candidate| candidate == rule)
        .into_iter()
        .map(|(line, _)| line)
        .collect()
}

fn collect(content: &str, applies: impl Fn(usize, &str) -> bool) -> HashSet<(usize, &str)> {
    if !content.contains("disable-line") {
        return HashSet::new();
    }

    let mut offset = 0;
    let mut original_tokens = None;
    let mut suppressed = HashSet::new();
    for (index, line) in content.split_inclusive('\n').enumerate() {
        for (hash, _) in line.match_indices('#') {
            let before = &line[..hash];
            let after = &line[hash + 1..];
            let command = after.trim_start_matches([' ', '\t']);
            let mut words = command.split_ascii_whitespace();
            if words.next() != Some("yamllint") || words.next() != Some("disable-line") {
                continue;
            }
            let Some(rule) = words.next().and_then(|word| word.strip_prefix("rule:")) else {
                continue;
            };
            if rule.is_empty() || words.next().is_some() {
                continue;
            }

            let target = index + if before.trim().is_empty() { 2 } else { 1 };
            if !applies(target, rule) {
                continue;
            }

            let original = original_tokens.get_or_insert_with(|| {
                let mut scanner = Scanner::new(content.chars());
                let tokens: Vec<_> = scanner.by_ref().collect();
                scanner.get_error().is_none().then_some(tokens)
            });
            let Some(original) = original else {
                return suppressed;
            };

            // The scanner discards comments. Editing the command changes a scalar
            // token, while editing the hash changes the stream only if it opens
            // the comment rather than appearing inside an earlier comment.
            let command_start = offset + hash + 1 + after.len() - command.len();
            if same_tokens(content, command_start, original)
                && !same_tokens(content, offset + hash, original)
            {
                suppressed.insert((target, rule));
            }
        }
        offset += line.len();
    }

    suppressed
}

fn same_tokens(content: &str, offset: usize, original: &[Token]) -> bool {
    let changed = content[..offset]
        .chars()
        .chain(std::iter::once('x'))
        .chain(content[offset + 1..].chars());
    let mut scanner = Scanner::new(changed);
    original.iter().all(|token| {
        scanner
            .next_token()
            .is_ok_and(|next| next.as_ref() == Some(token))
    }) && matches!(scanner.next_token(), Ok(None))
}
