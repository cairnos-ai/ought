/// MUST attach the GIVEN condition text to all clauses nested within it
#[test]
fn test_parser__conditional_blocks_given__must_attach_the_given_condition_text_to_all_clauses_nested_within() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::*;

    fn parse(md: &str) -> Spec {
        Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed")
    }

    let md = r#"# Svc

## Access

- **GIVEN** the request carries a valid token:
  - **MUST** allow the request through
  - **MUST NOT** log the token value
  - **SHOULD** refresh the token if near expiry
"#;
    let spec = parse(md);
    let clauses = &spec.sections[0].clauses;
    assert_eq!(clauses.len(), 3, "all three nested clauses should be emitted");
    for clause in clauses {
        assert_eq!(
            clause.condition.as_deref(),
            Some("the request carries a valid token:"),
            "every nested clause must carry the GIVEN condition text; clause '{}' did not",
            clause.text
        );
    }
}