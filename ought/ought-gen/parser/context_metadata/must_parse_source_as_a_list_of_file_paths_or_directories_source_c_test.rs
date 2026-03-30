/// MUST parse `source:` as a list of file paths or directories (source code hints for the LLM)
#[test]
fn test_parser__context_metadata__must_parse_source_as_a_list_of_file_paths_or_directories_source_c(
) {
    let md = r#"# MySpec

source: src/handlers/, src/models/user.rs

## Rules

- **MUST** do something
"#;
    let spec = parse(md);
    // Directories (trailing slash) and file paths (with extension) must both be accepted
    assert_eq!(spec.metadata.sources.len(), 2);
    assert!(
        spec.metadata.sources.iter().any(|s| s == "src/handlers/"),
        "directory path not found in sources"
    );
    assert!(
        spec.metadata.sources.iter().any(|s| s == "src/models/user.rs"),
        "file path not found in sources"
    );
}