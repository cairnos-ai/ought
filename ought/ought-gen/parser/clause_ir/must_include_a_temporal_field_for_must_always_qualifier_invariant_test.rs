/// MUST include a `temporal` field for MUST ALWAYS (qualifier: invariant) and
/// MUST BY (qualifier: deadline, duration: value+unit)
#[test]
fn test_parser__clause_ir__must_include_a_temporal_field_for_must_always_qualifier_invariant() {
    use std::path::Path;
    use std::time::Duration;
    use ought_spec::parser::Parser;
    use ought_spec::types::*;

    // Plain MUST has no temporal field
    let md = "# Svc\n\n## Rules\n\n- **MUST** validate input\n";
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    assert!(spec.sections[0].clauses[0].temporal.is_none());

    // MUST ALWAYS → Temporal::Invariant
    let md_always =
        "# Svc\n\n## Invariants\n\n- **MUST ALWAYS** keep connections below pool maximum\n";
    let spec_always =
        Parser::parse_string(md_always, Path::new("test.ought.md")).expect("parse failed");
    let clause_always = &spec_always.sections[0].clauses[0];
    assert_eq!(clause_always.keyword, Keyword::MustAlways);
    assert_eq!(clause_always.severity, Severity::Required);
    assert!(
        matches!(clause_always.temporal, Some(Temporal::Invariant)),
        "MUST ALWAYS should produce Temporal::Invariant"
    );

    // MUST BY <N>ms → Temporal::Deadline(Duration::from_millis(N))
    let md_ms = "# Svc\n\n## Perf\n\n- **MUST BY 200ms** return a response\n";
    let spec_ms = Parser::parse_string(md_ms, Path::new("test.ought.md")).expect("parse failed");
    let clause_ms = &spec_ms.sections[0].clauses[0];
    assert_eq!(clause_ms.keyword, Keyword::MustBy);
    assert_eq!(clause_ms.severity, Severity::Required);
    assert!(
        matches!(clause_ms.temporal, Some(Temporal::Deadline(d)) if d == Duration::from_millis(200)),
        "MUST BY 200ms should produce Deadline(200ms)"
    );

    // MUST BY <N>s → Temporal::Deadline(Duration::from_secs(N))
    let md_s = "# Svc\n\n## Perf\n\n- **MUST BY 5s** complete handshake\n";
    let spec_s = Parser::parse_string(md_s, Path::new("test.ought.md")).expect("parse failed");
    assert!(
        matches!(spec_s.sections[0].clauses[0].temporal, Some(Temporal::Deadline(d)) if d == Duration::from_secs(5)),
        "MUST BY 5s should produce Deadline(5s)"
    );

    // MUST BY <N>m → Temporal::Deadline(Duration::from_secs(N * 60))
    let md_m = "# Svc\n\n## Perf\n\n- **MUST BY 30m** finish batch job\n";
    let spec_m = Parser::parse_string(md_m, Path::new("test.ought.md")).expect("parse failed");
    assert!(
        matches!(spec_m.sections[0].clauses[0].temporal, Some(Temporal::Deadline(d)) if d == Duration::from_secs(30 * 60)),
        "MUST BY 30m should produce Deadline(30min)"
    );
}