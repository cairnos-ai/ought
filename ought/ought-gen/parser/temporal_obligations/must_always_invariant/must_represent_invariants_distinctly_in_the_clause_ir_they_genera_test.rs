/// MUST represent invariants distinctly in the clause IR (they generate different test patterns)
///
/// Verifies that `**MUST ALWAYS**` clauses are structurally distinct from plain
/// `**MUST**` clauses in the IR: different `keyword`, different `temporal` field,
/// and different stable `id`, ensuring downstream generators can tell them apart.
#[test]
fn test_parser__temporal_obligations__must_always_invariant__must_represent_invariants_distinctly_in_the_clause_ir_they_genera() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::*;

    fn parse(md: &str) -> Spec {
        Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed")
    }

    let md = r#"# Svc

## Rules

- **MUST** validate the request before processing
- **MUST ALWAYS** reject requests that exceed the rate limit
"#;
    let spec = parse(md);
    let clauses = &spec.sections[0].clauses;
    assert_eq!(clauses.len(), 2);

    let plain_must = &clauses[0];
    let invariant = &clauses[1];

    // Keywords are distinct variants
    assert_eq!(plain_must.keyword, Keyword::Must);
    assert_eq!(invariant.keyword, Keyword::MustAlways);
    assert_ne!(plain_must.keyword, invariant.keyword);

    // Temporal field distinguishes them: plain MUST has none, invariant has Some(Invariant)
    assert!(
        plain_must.temporal.is_none(),
        "plain MUST should have no temporal qualifier"
    );
    assert!(
        matches!(invariant.temporal, Some(Temporal::Invariant)),
        "MUST ALWAYS should carry Temporal::Invariant"
    );

    // Stable IDs are distinct (generators key off these to pick test strategy)
    assert_ne!(
        plain_must.id, invariant.id,
        "plain MUST and MUST ALWAYS clauses must have different IDs"
    );

    // Both share Required severity — the distinction is keyword+temporal, not severity
    assert_eq!(plain_must.severity, Severity::Required);
    assert_eq!(invariant.severity, Severity::Required);
}