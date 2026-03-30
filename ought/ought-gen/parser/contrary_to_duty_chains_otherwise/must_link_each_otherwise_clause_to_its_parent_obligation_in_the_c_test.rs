/// MUST link each OTHERWISE clause to its parent obligation in the clause IR
#[test]
fn test_parser__contrary_to_duty_chains_otherwise__must_link_each_otherwise_clause_to_its_parent_obligation_in_the_c() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::*;

    let md = r#"# Svc

## Api

- **MUST** validate the request payload
  - **OTHERWISE** reject with 400 Bad Request

- **MUST** authenticate the caller
  - **OTHERWISE** reject with 401 Unauthorized
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec.sections[0].clauses;
    assert_eq!(clauses.len(), 2);

    // Each OTHERWISE is reachable only through its parent's .otherwise field
    assert_eq!(clauses[0].keyword, Keyword::Must);
    assert_eq!(clauses[0].otherwise.len(), 1);
    assert!(clauses[0].otherwise[0].text.contains("400"));

    assert_eq!(clauses[1].keyword, Keyword::Must);
    assert_eq!(clauses[1].otherwise.len(), 1);
    assert!(clauses[1].otherwise[0].text.contains("401"));

    // No OTHERWISE appears as a standalone top-level clause
    assert!(
        clauses.iter().all(|c| c.keyword != Keyword::Otherwise),
        "OTHERWISE clauses must not appear as top-level section clauses"
    );
}