/// MUST NOT treat GIVEN itself as a testable clause — it is a grouping construct with a precondition
#[test]
fn test_parser__conditional_blocks_given__must_not_treat_given_itself_as_a_testable_clause_it_is_a_grouping_con() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::*;

    fn parse(md: &str) -> Spec {
        Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed")
    }

    let md = r#"# Svc

## Access

- **GIVEN** the user is authenticated:
  - **MUST** return their profile data
"#;
    let spec = parse(md);
    let clauses = &spec.sections[0].clauses;

    // GIVEN itself must not appear as a clause
    let given_clauses: Vec<_> = clauses.iter()
        .filter(|c| c.keyword == Keyword::Given)
        .collect();
    assert!(
        given_clauses.is_empty(),
        "GIVEN must not appear as a testable clause in the IR; found {} Given clause(s)",
        given_clauses.len()
    );

    // Only the nested MUST should be present
    assert_eq!(clauses.len(), 1);
    assert_eq!(clauses[0].keyword, Keyword::Must);
}