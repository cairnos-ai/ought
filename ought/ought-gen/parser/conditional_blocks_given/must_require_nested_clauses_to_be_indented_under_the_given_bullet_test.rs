/// MUST require nested clauses to be indented under the GIVEN bullet (standard markdown nesting)
#[test]
fn test_parser__conditional_blocks_given__must_require_nested_clauses_to_be_indented_under_the_given_bullet() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::*;

    fn parse(md: &str) -> Spec {
        Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed")
    }

    // The MUST is at the same indentation level as the GIVEN — not nested under it.
    // It should be treated as a top-level clause with no condition.
    let md = r#"# Svc

## Rules

- **GIVEN** user is admin:
- **MUST** do something important
"#;
    let spec = parse(md);
    let clauses = &spec.sections[0].clauses;
    // The MUST is a sibling of GIVEN, not a child — it gets no condition
    let must_clause = clauses.iter().find(|c| c.keyword == Keyword::Must)
        .expect("expected a MUST clause");
    assert!(
        must_clause.condition.is_none(),
        "un-indented MUST after GIVEN must not inherit the GIVEN condition"
    );
}