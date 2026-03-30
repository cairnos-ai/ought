/// MUST parse `schema:` as a list of file paths (schemas, configs, migrations)
#[test]
fn test_parser__context_metadata__must_parse_schema_as_a_list_of_file_paths_schemas_configs_migrati(
) {
    let md = r#"# MySpec

schema: schema/auth.graphql, config/settings.json, migrations/001_init.sql

## Rules

- **MUST** do something
"#;
    let spec = parse(md);
    // GraphQL schemas, JSON configs, and SQL migrations must all be accepted as schema paths
    assert_eq!(spec.metadata.schemas.len(), 3);
    assert!(
        spec.metadata.schemas.iter().any(|s| s == "schema/auth.graphql"),
        "graphql schema not found"
    );
    assert!(
        spec.metadata.schemas.iter().any(|s| s == "config/settings.json"),
        "json config not found"
    );
    assert!(
        spec.metadata.schemas.iter().any(|s| s == "migrations/001_init.sql"),
        "sql migration not found"
    );
}