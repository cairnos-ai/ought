/// MUST support multiple GIVEN blocks within a section
#[test]
fn test_parser__conditional_blocks_given__must_support_multiple_given_blocks_within_a_section() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::*;

    fn parse(md: &str) -> Spec {
        Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed")
    }

    let md = r#"# Svc

## Auth

- **GIVEN** the user is an admin:
  - **MUST** allow access to the admin panel
  - **MAY** impersonate other users
- **GIVEN** the user is a guest:
  - **MUST NOT** access private resources
  - **SHOULD** be shown a login prompt
"#;
    let spec = parse(md);
    let clauses = &spec.sections[0].clauses;
    assert_eq!(clauses.len(), 4, "two GIVEN blocks with two children each = four clauses");

    let admin_clauses: Vec<_> = clauses.iter()
        .filter(|c| c.condition.as_deref() == Some("the user is an admin:"))
        .collect();
    assert_eq!(admin_clauses.len(), 2);
    assert!(admin_clauses.iter().any(|c| c.keyword == Keyword::Must));
    assert!(admin_clauses.iter().any(|c| c.keyword == Keyword::May));

    let guest_clauses: Vec<_> = clauses.iter()
        .filter(|c| c.condition.as_deref() == Some("the user is a guest:"))
        .collect();
    assert_eq!(guest_clauses.len(), 2);
    assert!(guest_clauses.iter().any(|c| c.keyword == Keyword::MustNot));
    assert!(guest_clauses.iter().any(|c| c.keyword == Keyword::Should));
}