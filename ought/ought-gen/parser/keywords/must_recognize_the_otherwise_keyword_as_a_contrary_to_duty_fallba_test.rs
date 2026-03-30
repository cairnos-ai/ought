/// MUST recognize the OTHERWISE keyword as a contrary-to-duty fallback (from deontic logic)
#[test]
fn test_parser__keywords__must_recognize_the_otherwise_keyword_as_a_contrary_to_duty_fallba() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::*;

    let md = r#"# Svc

## Resilience

- **MUST** respond within 100ms
  - **OTHERWISE** return a stale cached response
  - **OTHERWISE** return HTTP 503 with Retry-After header
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec.sections[0].clauses;
    assert_eq!(clauses.len(), 1);
    assert_eq!(clauses[0].keyword, Keyword::Must);
    let otherwise = &clauses[0].otherwise;
    assert_eq!(otherwise.len(), 2);
    assert_eq!(otherwise[0].keyword, Keyword::Otherwise);
    assert!(otherwise[0].text.contains("stale cached"));
    assert_eq!(otherwise[1].keyword, Keyword::Otherwise);
    assert!(otherwise[1].text.contains("503"));
}