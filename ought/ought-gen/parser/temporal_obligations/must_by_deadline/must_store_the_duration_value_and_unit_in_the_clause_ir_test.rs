/// MUST store the duration value and unit in the clause IR
///
/// Verifies that the numeric value and the unit of a `**MUST BY**` clause are
/// faithfully preserved in `clause.temporal` as `Temporal::Deadline` with the
/// correct `value` and `unit` fields, and that the clause is otherwise well-formed.
#[test]
fn test_parser__temporal_obligations__must_by_deadline__must_store_the_duration_value_and_unit_in_the_clause_ir() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::*;

    fn parse(md: &str) -> Spec {
        Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed")
    }

    let md = r#"# Svc

## SLAs

- **MUST BY 250ms** acknowledge every incoming message
"#;
    let spec = parse(md);
    let clause = &spec.sections[0].clauses[0];

    // Keyword is the compound MustBy variant
    assert_eq!(clause.keyword, Keyword::MustBy);

    // Temporal must be present and carry a Deadline
    let temporal = clause.temporal.as_ref().expect("temporal should be Some for MUST BY clause");
    match temporal {
        Temporal::Deadline(d) => {
            assert_eq!(d.value, 250, "deadline value should be 250");
            assert_eq!(d.unit, DurationUnit::Milliseconds, "deadline unit should be Milliseconds");
        }
        other => panic!("expected Temporal::Deadline, got {:?}", other),
    }

    // Must NOT be an Invariant
    assert!(
        !matches!(clause.temporal, Some(Temporal::Invariant)),
        "MUST BY must not produce an Invariant temporal qualifier"
    );

    // Severity is Required (same as plain MUST / MUST ALWAYS)
    assert_eq!(clause.severity, Severity::Required);

    // Clause text body contains the obligation, not the duration token
    assert!(clause.text.contains("acknowledge every incoming message"));
    assert!(!clause.text.contains("250ms"));
}