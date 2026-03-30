/// MUST parse `**MUST BY <duration>**` as a compound keyword with a duration parameter
///
/// Verifies that `**MUST BY 30s**` is recognized as the compound keyword
/// `Keyword::MustBy`, not split into bare `MUST` or an unknown token.
#[test]
fn test_parser__temporal_obligations__must_by_deadline__must_parse_must_by_duration_as_a_compound_keyword_with_a_duration() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::*;

    fn parse(md: &str) -> Spec {
        Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed")
    }

    let md = r#"# Svc

## Deadlines

- **MUST BY 30s** respond to every health-check request
"#;
    let spec = parse(md);
    let clauses = &spec.sections[0].clauses;

    // Exactly one clause — not split
    assert_eq!(clauses.len(), 1, "expected exactly one clause");

    // Keyword must be the compound MustBy variant, not plain Must
    assert_eq!(clauses[0].keyword, Keyword::MustBy);
    assert_ne!(clauses[0].keyword, Keyword::Must);

    // The duration token must not bleed into the clause text body
    assert!(!clauses[0].text.trim_start().to_uppercase().starts_with("BY "));
    assert!(clauses[0].text.contains("respond to every health-check request"));
}