/// MUST NOT require users to write any test code by hand
#[test]
fn test_ought__what_ought_does__must_not_require_users_to_write_any_test_code_by_hand() {
    use std::fs;

    // Set up a project directory that contains only a spec file — no hand-written tests.
    let tmp_dir = std::env::temp_dir().join("ought_no_manual_tests");
    let _ = fs::create_dir_all(&tmp_dir);

    let spec_path = tmp_dir.join("feature.ought.md");
    let spec_content = "# Feature\n\n## Section\n\n### MUST work correctly\n";
    fs::write(&spec_path, spec_content).expect("Should be able to write spec file");

    // No hand-written test files should be required to exist.
    let handwritten_test = tmp_dir.join("tests.rs");
    assert!(
        !handwritten_test.exists(),
        "Users must not be required to provide a hand-written tests.rs"
    );

    // The spec itself must be written in human-readable prose, not Rust code.
    let contents = fs::read_to_string(&spec_path).unwrap();
    assert!(
        !contents.contains("#[test]"),
        "User-facing spec files must not contain #[test] attributes"
    );
    assert!(
        !contents.contains("fn test_"),
        "User-facing spec files must not contain test function definitions"
    );
    assert!(
        !contents.contains("assert!("),
        "User-facing spec files must not contain assertion macros"
    );

    let _ = fs::remove_file(&spec_path);
    let _ = fs::remove_dir(&tmp_dir);
}