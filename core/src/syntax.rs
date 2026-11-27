use crate::{LintLevel, LintProblem};
use yaml_rust2::parser::{Event, Parser};

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

#[cfg(test)]
mod tests {
    use crate::{Config, Fixer, LintLevel, Linter};

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
