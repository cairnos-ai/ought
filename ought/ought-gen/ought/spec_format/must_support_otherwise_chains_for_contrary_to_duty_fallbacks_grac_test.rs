/// MUST support **OTHERWISE** chains for contrary-to-duty fallbacks (graceful
/// degradation when an obligation is violated).
#[test]
fn test_ought__spec_format__must_support_otherwise_chains_for_contrary_to_duty_fallbacks_grac() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::{Keyword, Severity};

    let md = r#"# Spec Format

## Resilience

- **MUST** respond within 200ms
  - **OTHERWISE** return a cached response
  - **OTHERWISE** return 503 Service Unavailable
"#;

    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec.sections[0].clauses;

    // OTHERWISE items are NOT top-level clauses
    assert_eq!(clauses.len(), 1, "OTHERWISE items must not appear as top-level clauses");

    let primary = &clauses[0];
    assert_eq!(primary.keyword, Keyword::Must);
    assert_eq!(primary.otherwise.len(), 2, "Two OTHERWISE fallbacks expected");

    // First fallback
    assert_eq!(primary.otherwise[0].keyword, Keyword::Otherwise);
    assert!(primary.otherwise[0].text.contains("cached response"));
    // OTHERWISE inherits the parent's severity
    assert_eq!(primary.otherwise[0].severity, Severity::Required);

    // Second fallback
    assert_eq!(primary.otherwise[1].keyword, Keyword::Otherwise);
    assert!(primary.otherwise[1].text.contains("503"));
    assert_eq!(primary.otherwise[1].severity, Severity::Required);
}