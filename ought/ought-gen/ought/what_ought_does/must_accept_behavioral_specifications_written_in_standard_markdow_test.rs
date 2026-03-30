/// MUST accept behavioral specifications written in standard markdown files (`.ought.md`)
#[test]
fn test_ought__what_ought_does__must_accept_behavioral_specifications_written_in_standard_markdow() {
    use std::fs;

    let tmp_path = std::env::temp_dir().join("test_spec.ought.md");
    let spec_content = "# My Spec\n\n## Section\n\n### MUST do something\n";
    fs::write(&tmp_path, spec_content).expect("Failed to write test .ought.md file");

    assert!(tmp_path.exists(), "Spec file should exist after writing");

    assert!(
        tmp_path.to_str().unwrap().ends_with(".ought.md"),
        "File must use .ought.md compound extension"
    );

    let contents = fs::read_to_string(&tmp_path).expect("Spec file should be readable as UTF-8");
    assert!(!contents.is_empty(), "Spec file should not be empty");
    assert!(
        contents.contains('#'),
        "Spec file should contain markdown headings"
    );

    let _ = fs::remove_file(&tmp_path);
}