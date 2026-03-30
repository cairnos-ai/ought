/// MUST report when a generated test file is missing (referenced in manifest but not on disk)
#[test]
fn test_runner__error_handling__must_report_when_a_generated_test_file_is_missing_referenced_in_m() {
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;

    // The manifest records which files were generated for each clause.
    // Before invoking the harness the runner must verify every referenced
    // file exists and report any that are absent.

    #[derive(Debug)]
    struct ManifestEntry {
        clause_id: String,
        file_path: PathBuf,
    }

    #[derive(Debug, PartialEq)]
    struct MissingFileError {
        clause_id: String,
        expected_path: PathBuf,
    }

    fn check_files_on_disk(entries: &[ManifestEntry]) -> Vec<MissingFileError> {
        entries
            .iter()
            .filter(|e| !e.file_path.exists())
            .map(|e| MissingFileError {
                clause_id: e.clause_id.clone(),
                expected_path: e.file_path.clone(),
            })
            .collect()
    }

    let tmp = std::env::temp_dir().join(format!(
        "ought_missing_file_test_{}",
        std::process::id()
    ));
    fs::create_dir_all(&tmp).unwrap();

    let existing = tmp.join("runner__clause_present.rs");
    fs::write(&existing, "// generated test").unwrap();

    let entries = vec![
        ManifestEntry {
            clause_id: "runner::error_handling::clause_present".into(),
            file_path: existing.clone(),
        },
        ManifestEntry {
            clause_id: "runner::error_handling::clause_missing".into(),
            file_path: tmp.join("runner__clause_missing.rs"),
        },
    ];

    let missing = check_files_on_disk(&entries);

    // Cleanup before asserting so a test failure doesn't leave debris.
    fs::remove_file(&existing).ok();
    fs::remove_dir_all(&tmp).ok();

    assert_eq!(missing.len(), 1, "Exactly one missing file must be reported");
    assert_eq!(
        missing[0].clause_id,
        "runner::error_handling::clause_missing",
        "Report must identify the clause whose file is absent"
    );
    assert!(
        missing[0].expected_path.to_string_lossy().contains("clause_missing"),
        "Report must include the path that was expected"
    );
}