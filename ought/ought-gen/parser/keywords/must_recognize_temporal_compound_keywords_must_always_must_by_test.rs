/// MUST recognize temporal compound keywords: MUST ALWAYS, MUST BY
#[test]
fn test_parser__keywords__must_recognize_temporal_compound_keywords_must_always_must_by() {
    use std::path::Path;
    use std::time::Duration;
    use ought_spec::parser::Parser;
    use ought_spec::types::*;

    let md = r#"# Svc

## Temporal

- **MUST ALWAYS** keep connection pool below maximum capacity
- **MUST BY 500ms** return a search result
- **MUST BY 10s** complete the checkout flow
- **MUST BY 2m** finish a background import job
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec.sections[0].clauses;
    assert_eq!(clauses.len(), 4);

    assert_eq!(clauses[0].keyword, Keyword::MustAlways);
    assert!(
        matches!(clauses[0].temporal, Some(Temporal::Invariant)),
        "MUST ALWAYS must carry Temporal::Invariant"
    );

    assert_eq!(clauses[1].keyword, Keyword::MustBy);
    assert!(
        matches!(clauses[1].temporal, Some(Temporal::Deadline(d)) if d == Duration::from_millis(500)),
        "MUST BY 500ms must produce a 500ms deadline"
    );

    assert_eq!(clauses[2].keyword, Keyword::MustBy);
    assert!(
        matches!(clauses[2].temporal, Some(Temporal::Deadline(d)) if d == Duration::from_secs(10)),
        "MUST BY 10s must produce a 10s deadline"
    );

    assert_eq!(clauses[3].keyword, Keyword::MustBy);
    assert!(
        matches!(clauses[3].temporal, Some(Temporal::Deadline(d)) if d == Duration::from_secs(120)),
        "MUST BY 2m must produce a 120s deadline"
    );
}