/// MUST support multiple values per metadata key (one per line or comma-separated)
#[test]
fn test_parser__context_metadata__must_support_multiple_values_per_metadata_key_one_per_line_or_com(
) {
    // Comma-separated: all values on a single line
    let md_comma = r#"# MySpec

source: src/a/, src/b/, src/c/

## Rules

- **MUST** do something
"#;
    let spec = parse(md_comma);
    assert_eq!(
        spec.metadata.sources.len(),
        3,
        "comma-separated: expected 3 sources"
    );
    assert!(spec.metadata.sources.iter().any(|s| s == "src/a/"));
    assert!(spec.metadata.sources.iter().any(|s| s == "src/b/"));
    assert!(spec.metadata.sources.iter().any(|s| s == "src/c/"));

    // One per line: the same key repeated on adjacent lines (soft-break within same paragraph)
    let md_lines = r#"# MySpec

source: src/a/
source: src/b/
source: src/c/

## Rules

- **MUST** do something
"#;
    let spec2 = parse(md_lines);
    assert_eq!(
        spec2.metadata.sources.len(),
        3,
        "one-per-line: expected 3 sources"
    );
    assert!(spec2.metadata.sources.iter().any(|s| s == "src/a/"));
    assert!(spec2.metadata.sources.iter().any(|s| s == "src/b/"));
    assert!(spec2.metadata.sources.iter().any(|s| s == "src/c/"));
}