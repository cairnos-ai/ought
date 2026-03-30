/// MAY support glob patterns in `source:` and `schema:` paths
#[test]
fn test_parser__context_metadata__may_support_glob_patterns_in_source_and_schema_paths() {
    let md = r#"# MySpec

source: src/**/*.rs, tests/**/*.rs
schema: migrations/*.sql, config/*.json

## Rules

- **MUST** do something
"#;
    // Glob patterns must be accepted without error and stored as-is (not expanded)
    let spec = parse(md);
    assert!(
        spec.metadata.sources.iter().any(|s| s == "src/**/*.rs"),
        "recursive glob in source not preserved"
    );
    assert!(
        spec.metadata.sources.iter().any(|s| s == "tests/**/*.rs"),
        "recursive glob in tests source not preserved"
    );
    assert!(
        spec.metadata.schemas.iter().any(|s| s == "migrations/*.sql"),
        "wildcard glob in schema not preserved"
    );
    assert!(
        spec.metadata.schemas.iter().any(|s| s == "config/*.json"),
        "wildcard glob in config schema not preserved"
    );
}
```