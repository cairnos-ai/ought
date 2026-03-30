/// SHOULD support nested GIVEN blocks (conditions that narrow further)
#[test]
fn test_parser__conditional_blocks_given__should_support_nested_given_blocks_conditions_that_narrow_further() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::*;

    fn parse(md: &str) -> Spec {
        Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed")
    }

    let md = r#"# Svc

## Permissions

- **GIVEN** the user is an admin:
  - **GIVEN** the user account is active:
    - **MUST** allow full access
"#;
    let spec = parse(md);
    let clauses = &spec.sections[0].clauses;
    // The innermost MUST should be emitted as a clause
    assert_eq!(clauses.len(), 1);
    assert_eq!(clauses[0].keyword, Keyword::Must);
    assert!(clauses[0].text.contains("allow full access"));
    // The clause carries a condition derived from the inner (narrowing) GIVEN
    assert!(
        clauses[0].condition.is_some(),
        "clause nested inside two GIVENs must carry a condition"
    );
}