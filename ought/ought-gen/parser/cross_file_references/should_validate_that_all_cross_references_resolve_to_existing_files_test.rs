/// SHOULD validate that all cross-references resolve to existing files and sections
#[test]
fn test_parser__cross_file_references__should_validate_that_all_cross_references_resolve_to_existing_files() {
    use std::time::{SystemTime, UNIX_EPOCH};
    use ought_spec::SpecGraph;

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let tmp = std::env::temp_dir().join(format!("ought_xref_missing_{nanos}"));
    std::fs::create_dir_all(&tmp).unwrap();

    // spec_a references a file that is never written to disk
    std::fs::write(
        tmp.join("spec_a.ought.md"),
        "# SpecA\n\nrequires: [Missing](nonexistent.ought.md)\n\n## Rules\n\n- **MUST** reference real specs\n",
    )
    .unwrap();

    let result = SpecGraph::from_roots(&[tmp.clone()]);

    assert!(
        result.is_err(),
        "a requires: reference to a non-existent file must be reported as a validation error"
    );

    let errors = result.unwrap_err();
    let has_unresolved_error = errors.iter().any(|e| {
        e.message.contains("nonexistent.ought.md")
            || e.message.contains("unresolved")
            || e.message.contains("not found")
    });
    assert!(
        has_unresolved_error,
        "error must identify the unresolved cross-reference; got: {:?}",
        errors.iter().map(|e| &e.message).collect::<Vec<_>>()
    );

    let _ = std::fs::remove_dir_all(&tmp);
}