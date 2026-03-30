/// MUST recognize files with the `.ought.md` extension
#[test]
fn test_parser__spec_file_structure__must_recognize_files_with_the_ought_md_extension() {
    use std::fs;

    let content = "# Ext Test\n\n## Section\n\n- **MUST** work\n";
    let path = std::env::temp_dir().join("ought_recognize_ext_test.ought.md");
    fs::write(&path, content).expect("failed to write temp .ought.md file");

    let result = Parser::parse_file(&path);
    fs::remove_file(&path).ok();

    assert!(
        result.is_ok(),
        "Parser must recognize and parse .ought.md files: {:?}",
        result.err()
    );
    let spec = result.unwrap();
    assert_eq!(spec.name, "Ext Test");
    assert_eq!(spec.sections[0].clauses.len(), 1);
    // source_path must reflect the .ought.md filename
    assert!(
        spec.source_path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.ends_with(".ought.md"))
            .unwrap_or(false),
        "source_path must preserve the .ought.md extension"
    );
}