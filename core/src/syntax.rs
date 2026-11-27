use crate::yaml_rust2::parser::{Event, Parser};
use crate::yaml_rust2::scanner::{Marker, Scanner, TERMINAL_TAB_ERROR, Token, TokenType};
use crate::{LintLevel, LintProblem};

pub(crate) fn check(content: &str, problems: &mut Vec<LintProblem>) {
    let mut parser = Parser::new_from_str(content);
    loop {
        match parser.next_token() {
            Ok((Event::StreamEnd, _)) => return,
            Ok(_) => {}
            // The parser has consumed the alias and advanced its state. Alias
            // declarations are checked by the configurable anchors rule.
            Err(error) if error.info() == "while parsing node, found unknown anchor" => {}
            Err(error) => {
                let mark = error.marker();
                let position = (mark.line(), mark.col() + 1);
                let message =
                    error
                        .info()
                        .split_once(", ")
                        .map_or(error.info(), |(context, message)| {
                            if context.starts_with("while ") {
                                message
                            } else {
                                error.info()
                            }
                        });
                let message = compatible_message(content, *mark, message);
                problems.retain(|problem| {
                    let same_position = (problem.line, problem.column) == position;
                    // The fallback line check also diagnoses this continuation.
                    let same_continuation = error.at_plain_scalar_continuation()
                        && problem.line == position.0
                        && problem.column == 1
                        && problem.rule == "indentation";
                    let same_terminal_tab = error.info() == TERMINAL_TAB_ERROR
                        && problem.line == position.0
                        && problem.rule == "trailing-spaces";
                    !same_position && !same_continuation && !same_terminal_tab
                });
                problems.push(LintProblem::new(
                    position.0,
                    position.1,
                    format!("syntax error: {message}"),
                    "syntax",
                    LintLevel::Error,
                ));
                problems.sort();
                return;
            }
        }
    }
}

fn compatible_message(content: &str, mark: Marker, message: &str) -> String {
    let expected = match message {
        "mapping values are not allowed in this context" => {
            return "mapping values are not allowed here".into();
        }
        "did not find expected <document start>" => "'<document start>'",
        "did not find expected node content" => "the node content",
        _ => return message.into(),
    };
    let found = Scanner::new(content.chars())
        .take_while(|Token(position, _)| position.index() <= mark.index())
        .filter(|Token(position, _)| position.index() == mark.index())
        .find_map(|Token(_, token)| match token {
            TokenType::Scalar(..) => Some("<scalar>"),
            TokenType::DocumentEnd => Some("<document end>"),
            TokenType::FlowSequenceEnd => Some("]"),
            TokenType::FlowMappingEnd => Some("}"),
            TokenType::StreamEnd => Some("<stream end>"),
            _ => None,
        });
    found.map_or_else(
        || message.into(),
        |found| format!("expected {expected}, but found '{found}'"),
    )
}

#[cfg(test)]
mod tests {
    use crate::{Config, Fixer, LintLevel, Linter};

    #[test]
    fn reports_terminal_tabs_at_token_boundaries_in_lint_and_fix() {
        let config = Config::load_from_str("rules:\n  trailing-spaces: enable\n").unwrap();
        let registry = config.create_registry();
        let fixer = Fixer::new(&registry);
        let linter = Linter::new(config);
        let message = r"found character '\t' that cannot start any token";
        for (source, position) in [
            ("\t\t\t\n", (1, 1)),
            ("---\nsome: text\t\n", (2, 11)),
            ("---\nsome: text \t \r\n", (2, 12)),
            ("---\nsome: 'text'\t", (2, 13)),
        ] {
            assert_syntax(&linter, source, position, message);
            let fix = fixer.dry_run("tabs.yaml", source);
            assert!(!fix.has_fixes());
            assert_eq!(fix.unfixable_problems, linter.lint_string(source).unwrap());
        }

        let source = "one: text  \nsome: text\t\nother: text  \n";
        let fix = fixer.dry_run("tabs.yaml", source);
        assert_eq!(
            fix.fixed_content.as_deref(),
            Some("one: text\nsome: text\t\nother: text\n")
        );
        assert_eq!(
            (
                fix.unfixable_problems[0].line,
                fix.unfixable_problems[0].column
            ),
            (2, 11)
        );
    }

    #[test]
    fn keeps_tabs_in_scalar_content_comments_and_separators() {
        let linter =
            Linter::new(Config::load_from_str("rules: {trailing-spaces: disable}").unwrap());
        for source in [
            "some: 'text\t'\n",
            "some: |\n  text\t\n",
            "# comment\t\n",
            "some: text\t # comment\n",
            "some: \ttext\n",
            "some: [first,\t second]\n",
        ] {
            assert!(linter.lint_string(source).unwrap().is_empty(), "{source:?}");
        }
    }

    #[test]
    fn quoted_continuations_keep_original_indentation_positions() {
        let enabled = Linter::new(
            Config::load_from_str("rules:\n  indentation: {check-multi-line-strings: true}\n")
                .unwrap(),
        );
        let disabled = Linter::new(
            Config::load_from_str("rules:\n  indentation: {check-multi-line-strings: false}\n")
                .unwrap(),
        );
        let double = r#"- long text: "very \"long\"
'string' with

paragraph gap, \n and
spaces."
"#;
        let single = r#"- long text: 'very "long"
''string'' with

paragraph gap, \n and
spaces.'
"#;
        for source in [double, single] {
            let problems = enabled.lint_string(source).unwrap();
            let positions: Vec<_> = problems
                .iter()
                .map(|problem| (problem.line, problem.column, problem.rule.as_str()))
                .collect();
            assert_eq!(
                positions,
                [
                    (2, 1, "indentation"),
                    (4, 1, "indentation"),
                    (5, 1, "indentation")
                ]
            );
            assert!(disabled.lint_string(source).unwrap().is_empty());
            let unterminated = &source[..source.len() - 2];
            assert_syntax(
                &disabled,
                unterminated,
                (1, 14),
                "found unexpected end of stream",
            );
        }
    }

    #[test]
    fn malformed_block_collections_report_the_boundary_token() {
        let config =
            Config::load_from_str("rules:\n  indentation: {indent-sequences: whatever}\n").unwrap();
        let linter = Linter::new(config);
        let cases = [
            (
                "---\nlist:\n    - 1\n    - 2\n    - 3\n- a\n- b\n- c\n",
                (6, 1),
                "expected <block end>, but found '-'",
                "---\nlist:\n    - 1\n    - 2\n    - 3\n    - a\n",
            ),
            (
                "---\n- o:\n k1: v1\n...\n",
                (3, 2),
                "expected <block end>, but found '<block mapping start>'",
                "---\n- o:\n  k1: v1\n...\n",
            ),
            (
                "---\na:\n  b:\n    c:\n   d:\n...\n",
                (5, 4),
                "expected <block end>, but found '<block mapping start>'",
                "---\na:\n  b:\n    c:\n  d:\n...\n",
            ),
            (
                "---\na:\n  b:\n    c:\n d:\n...\n",
                (5, 2),
                "expected <block end>, but found '<block mapping start>'",
                "---\na:\n  b:\n    c:\nd:\n...\n",
            ),
        ];
        for (source, position, message, neighbor) in cases {
            assert_syntax(&linter, source, position, message);
            assert!(
                linter.lint_string(neighbor).unwrap().is_empty(),
                "{neighbor:?}"
            );
        }
        for (step, name) in [(2, "Unix"), (4, "Linux")] {
            let key = " ".repeat(step);
            let dash = " ".repeat(step * 2);
            let prefix = format!("---\nobject:\n{key}k3:\n{dash}- name: {name}\n");
            let child = format!("{dash}  ");
            let over = format!("{prefix}{child} date: 1969\n...\n");
            assert_syntax(
                &linter,
                &over,
                (5, child.len() + 6),
                "mapping values are not allowed here",
            );
            let under = format!("{prefix}{dash} date: 1969\n...\n");
            assert_syntax(
                &linter,
                &under,
                (5, child.len()),
                "expected <block end>, but found '<block mapping start>'",
            );
            let neighbor = format!("{prefix}{child}date: 1969\n...\n");
            assert!(
                linter.lint_string(&neighbor).unwrap().is_empty(),
                "{neighbor:?}"
            );
        }
    }

    fn assert_syntax(linter: &Linter, source: &str, position: (usize, usize), message: &str) {
        let problems = linter.lint_string(source).unwrap();
        assert_eq!(problems.len(), 1, "{source:?}: {problems:?}");
        let problem = &problems[0];
        assert_eq!((problem.line, problem.column), position, "{source:?}");
        assert_eq!(problem.rule, "syntax");
        assert_eq!(problem.level, LintLevel::Error);
        assert_eq!(problem.message, format!("syntax error: {message}"));
    }

    #[test]
    fn requires_an_explicit_start_after_an_explicit_document_end() {
        let config =
            Config::load_from_str("rules:\n  document-start: {present: true, level: warning}\n")
                .unwrap();
        let registry = config.create_registry();
        let linter = Linter::new(config);
        let source = "---\nfirst: document\n...\nsecond: document\n---\nthird: document\n";
        let problems = linter.lint_string(source).unwrap();
        assert_eq!(problems.len(), 1);
        let problem = &problems[0];
        assert_eq!(
            (problem.line, problem.column, problem.rule.as_str()),
            (4, 1, "syntax")
        );
        assert_eq!(
            problem.message,
            "syntax error: expected '<document start>', but found '<block mapping start>'"
        );
        assert_eq!(
            Fixer::new(&registry)
                .dry_run("documents.yaml", source)
                .unfixable_problems,
            problems
        );

        let source = "first: document\n...\n---\nsecond: document\n";
        let problems = linter.lint_string(source).unwrap();
        assert_eq!(problems.len(), 1);
        assert_eq!(
            (
                problems[0].line,
                problems[0].column,
                problems[0].rule.as_str(),
                problems[0].level
            ),
            (1, 1, "document-start", LintLevel::Warning)
        );
    }

    #[test]
    fn recognizes_alias_keys_immediately_followed_by_a_colon() {
        let config =
            Config::load_from_str("rules:\n  anchors: enable\n  colons: enable\n").unwrap();
        let linter = Linter::new(config);
        let source = "---\n- anchor: &a key\n- *a: 42\n- {*a: 42}\n- *a : 42\n- {*a : 42}\n- *a  : 42\n- {*a  : 42}\n";
        let problems = linter.lint_string(source).unwrap();
        let positions: Vec<_> = problems
            .iter()
            .map(|problem| (problem.line, problem.column, problem.rule.as_str()))
            .collect();
        assert_eq!(positions, [(7, 6, "colons"), (8, 7, "colons")]);

        let source = "- &a:b value\n- *a:b\n- *missing: 42\n";
        let problems = linter.lint_string(source).unwrap();
        assert_eq!(problems.len(), 1);
        assert_eq!(
            (
                problems[0].line,
                problems[0].column,
                problems[0].rule.as_str()
            ),
            (3, 3, "anchors")
        );
    }

    #[test]
    fn outdented_flow_closers_reach_the_lint_and_fix_rules() {
        let config = Config::load_from_str("rules:\n  indentation: enable\n").unwrap();
        let registry = config.create_registry();
        let linter = Linter::new(config);
        for source in ["---\ndict: {\n}\n", "---\narray: [\n]\n"] {
            assert!(linter.lint_string(source).unwrap().is_empty());
            assert!(
                Fixer::new(&registry)
                    .dry_run("flow.yaml", source)
                    .unfixable_problems
                    .is_empty()
            );
        }
        let source = "---\ntop:\n  rules: [\n    1, 2,\n]\n  rulez: [\n    1, 2,\n    ]\n";
        let problems = linter.lint_string(source).unwrap();
        let positions: Vec<_> = problems
            .iter()
            .map(|problem| (problem.line, problem.column, problem.rule.as_str()))
            .collect();
        assert_eq!(positions, [(5, 1, "indentation"), (8, 5, "indentation")]);
        assert_eq!(
            Fixer::new(&registry)
                .dry_run("flow.yaml", source)
                .unfixable_problems,
            problems
        );
    }

    #[test]
    fn invalid_flow_closures_still_report_syntax() {
        let linter = Linter::new(Config::load_from_str("rules: {}").unwrap());
        for source in ["key: [\n}\n", "key: [\n", "key: value\n]\n"] {
            assert!(
                linter
                    .lint_string(source)
                    .unwrap()
                    .iter()
                    .any(|problem| problem.rule == "syntax")
            );
        }
    }

    #[test]
    fn reports_syntax_after_an_unknown_alias_without_suppressing_other_rules() {
        let config = Config::load_from_str(
            "rules:\n  anchors: {forbid-undeclared-aliases: false}\n  trailing-spaces: enable\n",
        )
        .unwrap();
        let source =
            "alias: *unknown  \nbroken: value: other # yamllint disable-line rule:syntax\n";
        let registry = config.create_registry();
        let problems = Linter::new(config).lint_string(source).unwrap();
        assert_eq!(problems.len(), 2);
        assert_eq!(problems[0].rule, "trailing-spaces");
        let syntax = &problems[1];
        assert_eq!((syntax.line, syntax.column), (2, 14));
        assert_eq!(syntax.rule, "syntax");
        assert_eq!(syntax.level, LintLevel::Error);

        let fix = Fixer::new(&registry).dry_run("broken.yaml", source);
        assert!(fix.has_fixes());
        assert_eq!(fix.unfixable_problems, std::slice::from_ref(syntax));
    }
}
