/// MUST parse duration suffixes: `ms` (milliseconds), `s` (seconds), `m` (minutes)
///
/// Verifies that all three recognised unit suffixes — `ms`, `s`, and `m` — are
/// accepted and round-trip correctly through the IR as their respective `DurationUnit`
/// variants.
#[test]
fn test_parser__temporal_obligations__must_by_deadline__must_parse_duration_suffixes_ms_milliseconds_s_seconds_m_minutes() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::*;

    fn parse(md: &str) -> Spec {
        Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed")
    }

    // --- milliseconds ---
    let md_ms = r#"# Svc

## Deadlines

- **MUST BY 200ms** return a cached response
"#;
    let spec_ms = parse(md_ms);
    let clause_ms = &spec_ms.sections[0].clauses[0];
    assert_eq!(clause_ms.keyword, Keyword::MustBy);
    let temporal_ms = clause_ms.temporal.as_ref().expect("temporal must be Some for MUST BY");
    match temporal_ms {
        Temporal::Deadline(d) => {
            assert_eq!(d.value, 200, "expected value 200 for ms clause");
            assert_eq!(d.unit, DurationUnit::Milliseconds);
        }
        other => panic!("expected Temporal::Deadline, got {:?}", other),
    }

    // --- seconds ---
    let md_s = r#"# Svc

## Deadlines

- **MUST BY 5s** complete the database write
"#;
    let spec_s = parse(md_s);
    let clause_s = &spec_s.sections[0].clauses[0];
    assert_eq!(clause_s.keyword, Keyword::MustBy);
    let temporal_s = clause_s.temporal.as_ref().expect("temporal must be Some for MUST BY");
    match temporal_s {
        Temporal::Deadline(d) => {
            assert_eq!(d.value, 5, "expected value 5 for s clause");
            assert_eq!(d.unit, DurationUnit::Seconds);
        }
        other => panic!("expected Temporal::Deadline, got {:?}", other),
    }

    // --- minutes ---
    let md_m = r#"# Svc

## Deadlines

- **MUST BY 10m** finish the batch export job
"#;
    let spec_m = parse(md_m);
    let clause_m = &spec_m.sections[0].clauses[0];
    assert_eq!(clause_m.keyword, Keyword::MustBy);
    let temporal_m = clause_m.temporal.as_ref().expect("temporal must be Some for MUST BY");
    match temporal_m {
        Temporal::Deadline(d) => {
            assert_eq!(d.value, 10, "expected value 10 for m clause");
            assert_eq!(d.unit, DurationUnit::Minutes);
        }
        other => panic!("expected Temporal::Deadline, got {:?}", other),
    }
}