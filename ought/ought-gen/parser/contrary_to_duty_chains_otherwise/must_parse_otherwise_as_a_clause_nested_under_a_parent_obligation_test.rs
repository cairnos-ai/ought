/// MUST parse `**OTHERWISE**` as a clause nested under a parent obligation
#[test]
fn test_parser__contrary_to_duty_chains_otherwise__must_parse_otherwise_as_a_clause_nested_under_a_parent_obligation() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::*;

    let md = r#"# Svc

## Resilience

- **MUST** respond within 200ms
  - **OTHERWISE** return a cached response
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec.sections[0].clauses;

    // The parent MUST is the only top-level clause; OTHERWISE is not promoted to the top level
    assert_eq!(clauses.len(), 1, "only the parent obligation should appear as a top-level clause");
    assert_eq!(clauses[0].keyword, Keyword::Must);

    let otherwise = &clauses[0].otherwise;
    assert_eq!(otherwise.len(), 1);
    assert_eq!(otherwise[0].keyword, Keyword::Otherwise);
    assert!(otherwise[0].text.contains("cached response"));
}