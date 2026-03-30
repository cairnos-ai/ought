/// MUST parse `**MUST ALWAYS**` as a single compound keyword
///
/// Verifies that `**MUST ALWAYS**` is recognized as the single compound keyword
/// `Keyword::MustAlways`, not treated as bare `MUST` or split into two tokens.
#[test]
fn test_parser__temporal_obligations__must_always_invariant__must_parse_must_always_as_a_single_compound_keyword() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::*;

    fn parse(md: &str) -> Spec {
        Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed")
    }

    let md = r#"# Svc

## Invariants

- **MUST ALWAYS** keep connection count below the pool maximum
"#;
    let spec = parse(md);
    let clauses = &spec.sections[0].clauses;

    // Exactly one clause — not split into two items
    assert_eq!(clauses.len(), 1);

    // Keyword must be the compound MustAlways variant, not plain Must
    assert_eq!(clauses[0].keyword, Keyword::MustAlways);
    assert_ne!(clauses[0].keyword, Keyword::Must);

    // The word "ALWAYS" must not bleed into the clause text body
    assert!(!clauses[0].text.to_uppercase().starts_with("ALWAYS"));
    assert!(clauses[0].text.contains("keep connection count below the pool maximum"));
}