/// MUST support **MUST ALWAYS** for invariants (properties that must hold
/// across all states and inputs).
#[test]
fn test_ought__spec_format__must_support_must_always_for_invariants_properties_that_must_hold() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::{Keyword, Severity, Temporal};

    let md = r#"# Spec Format

## Invariants

- **MUST ALWAYS** keep database connections below pool maximum
- **MUST ALWAYS** produce a valid UTF-8 response body
"#;

    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec.sections[0].clauses;

    assert_eq!(clauses.len(), 2);

    for clause in clauses {
        assert_eq!(clause.keyword, Keyword::MustAlways);
        assert_eq!(clause.severity, Severity::Required);
        assert!(
            matches!(clause.temporal, Some(Temporal::Invariant)),
            "MUST ALWAYS must set temporal to Invariant, got {:?}", clause.temporal
        );
    }

    assert!(clauses[0].text.contains("pool maximum"));
    assert!(clauses[1].text.contains("UTF-8"));
}