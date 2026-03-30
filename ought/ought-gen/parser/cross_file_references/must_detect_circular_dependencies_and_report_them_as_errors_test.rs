/// MUST detect circular dependencies and report them as errors
#[test]
fn test_parser__cross_file_references__must_detect_circular_dependencies_and_report_them_as_errors() {
    use std::time::{SystemTime, UNIX_EPOCH};
    use ought_spec::SpecGraph;

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let tmp = std::env::temp_dir().join(format!("ought_xref_cycle_{nanos}"));
    std::fs::create_dir_all(&tmp).unwrap();

    // a.ought.md requires b, and b.ought.md requires a — a direct mutual cycle
    std::fs::write(
        tmp.join("a.ought.md"),
        "# SpecA\n\nrequires: [SpecB](b.ought.md)\n\n## Rules\n\n- **MUST** do something\n",
    )
    .unwrap();
    std::fs::write(
        tmp.join("b.ought.md"),
        "# SpecB\n\nrequires: [SpecA](a.ought.md)\n\n## Rules\n\n- **MUST** do something\n",
    )
    .unwrap();

    let result = SpecGraph::from_roots(&[tmp.clone()]);

    assert!(
        result.is_err(),
        "a circular dependency must be reported as an error rather than silently accepted"
    );

    let errors = result.unwrap_err();
    let has_cycle_error = errors
        .iter()
        .any(|e| e.message.contains("circular dependency"));
    assert!(
        has_cycle_error,
        "error message must identify the circular dependency; got: {:?}",
        errors.iter().map(|e| &e.message).collect::<Vec<_>>()
    );

    let _ = std::fs::remove_dir_all(&tmp);
}