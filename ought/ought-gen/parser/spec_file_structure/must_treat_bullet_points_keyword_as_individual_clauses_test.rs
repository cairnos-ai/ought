/// MUST treat bullet points (`- **KEYWORD**`) as individual clauses
#[test]
fn test_parser__spec_file_structure__must_treat_bullet_points_keyword_as_individual_clauses() {
    let md = r#"# Svc

## API

- **MUST** return a response body
- **MUST NOT** leak internal error details
- **SHOULD** include a request-id header
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec.sections[0].clauses;

    assert_eq!(clauses.len(), 3, "each bold-keyword bullet must become exactly one clause");

    assert_eq!(clauses[0].keyword, Keyword::Must);
    assert_eq!(clauses[0].severity, Severity::Required);
    assert!(clauses[0].text.contains("return a response body"));

    assert_eq!(clauses[1].keyword, Keyword::MustNot);
    assert_eq!(clauses[1].severity, Severity::Required);
    assert!(clauses[1].text.contains("leak internal error details"));

    assert_eq!(clauses[2].keyword, Keyword::Should);
    assert_eq!(clauses[2].severity, Severity::Recommended);
    assert!(clauses[2].text.contains("request-id header"));

    // Each clause must have a non-empty, unique stable ID
    assert!(!clauses[0].id.0.is_empty());
    assert_ne!(clauses[0].id.0, clauses[1].id.0);
    assert_ne!(clauses[1].id.0, clauses[2].id.0);
}