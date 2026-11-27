use crate::yaml_rust2::parser::{Event, Parser};
use crate::yaml_rust2::scanner::{Marker, Scanner, Token, TokenType};
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
                problems.retain(|problem| (problem.line, problem.column) != position);
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
