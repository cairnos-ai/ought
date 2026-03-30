/// MUST assign the `invariant` temporal qualifier to the clause
///
/// Verifies that the `temporal` field of a `**MUST ALWAYS**` clause is
/// `Some(Temporal::Invariant)` and not `None` or a `Deadline` variant.
#[test]
fn test_parser__temporal_obligations__must_always_invariant__must_assign_the_invariant_temporal_qualifier_to_the_clause() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::*;

    fn parse(md: &str) -> Spec {
        Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed")
    }

    let md = r#"# Svc

## Invariants

- **MUST ALWAYS** hold the invariant that no account balance drops below zero
"#;
    let spec = parse(md);
    let clause = &spec.sections[0].clauses[0];

    // Temporal must be present and be the Invariant variant
    assert!(clause.temporal.is_some(), "temporal should be Some, not None");
    assert!(
        matches!(clause.temporal, Some(Temporal::Invariant)),
        "temporal should be Invariant, got {:?}",
        clause.temporal
    );

    // Must NOT be a deadline — Invariant and Deadline are mutually exclusive
    assert!(
        !matches!(clause.temporal, Some(Temporal::Deadline(_))),
        "MUST ALWAYS must not produce a Deadline temporal qualifier"
    );
}