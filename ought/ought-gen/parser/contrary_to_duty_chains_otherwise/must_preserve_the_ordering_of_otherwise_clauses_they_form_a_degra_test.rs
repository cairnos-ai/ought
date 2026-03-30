/// MUST preserve the ordering of OTHERWISE clauses (they form a degradation chain)
#[test]
fn test_parser__contrary_to_duty_chains_otherwise__must_preserve_the_ordering_of_otherwise_clauses_they_form_a_degra() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::*;

    let md = r#"# Svc

## Resilience

- **MUST** respond with fresh data
  - **OTHERWISE** return stale cache
  - **OTHERWISE** return degraded placeholder
  - **OTHERWISE** return HTTP 503
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let otherwise = &spec.sections[0].clauses[0].otherwise;

    assert_eq!(otherwise.len(), 3);
    // Degradation chain order must match declaration order
    assert!(otherwise[0].text.contains("stale cache"),        "first fallback must be stale cache");
    assert!(otherwise[1].text.contains("degraded placeholder"), "second fallback must be degraded placeholder");
    assert!(otherwise[2].text.contains("503"),                "third fallback must be 503");
}