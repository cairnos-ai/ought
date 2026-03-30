/// MUST support **MUST BY** for deadline obligations (operations that must
/// complete within a time bound).
#[test]
fn test_ought__spec_format__must_support_must_by_for_deadline_obligations_operations_that_mus() {
    use std::path::Path;
    use std::time::Duration;
    use ought_spec::parser::Parser;
    use ought_spec::types::{Keyword, Severity, Temporal};

    let md = r#"# Spec Format

## Performance

- **MUST BY 100ms** return search results
- **MUST BY 5s** complete authentication handshake
- **MUST BY 30m** finish the nightly batch export
"#;

    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec.sections[0].clauses;

    assert_eq!(clauses.len(), 3);

    // All MUST BY clauses are Required
    for clause in clauses {
        assert_eq!(clause.keyword, Keyword::MustBy);
        assert_eq!(clause.severity, Severity::Required);
    }

    // 100ms deadline
    assert!(
        matches!(clauses[0].temporal, Some(Temporal::Deadline(d)) if d == Duration::from_millis(100)),
        "Expected 100ms deadline, got {:?}", clauses[0].temporal
    );
    assert!(clauses[0].text.contains("search results"));

    // 5s deadline
    assert!(
        matches!(clauses[1].temporal, Some(Temporal::Deadline(d)) if d == Duration::from_secs(5)),
        "Expected 5s deadline, got {:?}", clauses[1].temporal
    );
    assert!(clauses[1].text.contains("authentication handshake"));

    // 30m deadline
    assert!(
        matches!(clauses[2].temporal, Some(Temporal::Deadline(d)) if d == Duration::from_secs(30 * 60)),
        "Expected 30m deadline, got {:?}", clauses[2].temporal
    );
    assert!(clauses[2].text.contains("batch export"));
}