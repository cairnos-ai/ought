/// MUST read and include schema files referenced by `schema:` metadata
#[test]
fn test_generator__context_assembly__must_read_and_include_schema_files_referenced_by_schema_metadata() {
    use std::path::PathBuf;
    use std::collections::HashMap;
    use ought_spec::types::{Clause, ClauseId, Keyword, Severity, Spec, Metadata, SourceLocation};
    use ought_spec::config::{Config, ProjectConfig, SpecsConfig, ContextConfig, GeneratorConfig, ToleranceConfig, McpConfig};
    use ought_gen::context::ContextAssembler;

    let tmp = std::env::temp_dir().join("ought_ctxasm_schema_meta");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    let schema_content = r#"type Query { user(id: ID!): User } type User { id: ID! name: String! }"#;
    std::fs::write(tmp.join("users.graphql"), schema_content).unwrap();

    let spec = Spec {
        name: "Users".to_string(),
        metadata: Metadata {
            context: None,
            sources: vec![],
            schemas: vec!["users.graphql".to_string()],
            requires: vec![],
        },
        sections: vec![],
        source_path: tmp.join("users.ought.md"),
    };

    let clause = Clause {
        id: ClauseId("users::must_return_user_fields".to_string()),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: "return all required user fields".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("users.ought.md"), line: 1 },
        content_hash: "ghi789".to_string(),
    };

    let config = Config {
        project: ProjectConfig { name: "test".to_string(), version: "0.1.0".to_string() },
        specs: SpecsConfig::default(),
        context: ContextConfig { search_paths: vec![], exclude: vec![], max_files: 50 },
        generator: GeneratorConfig { provider: "claude".to_string(), model: None, tolerance: ToleranceConfig::default() },
        runner: HashMap::new(),
        mcp: McpConfig::default(),
    };

    let assembler = ContextAssembler::new(&config);
    let ctx = assembler.assemble(&clause, &spec).expect("assemble failed");

    assert_eq!(ctx.schema_files.len(), 1, "expected one schema file from metadata");
    assert!(
        ctx.schema_files[0].path.ends_with("users.graphql"),
        "schema file path should be users.graphql; got {:?}",
        ctx.schema_files[0].path
    );
    assert_eq!(
        ctx.schema_files[0].content, schema_content,
        "schema file content must match what was written"
    );
    // Schema files must NOT appear in source_files
    assert!(
        ctx.source_files.is_empty(),
        "schema files must not bleed into source_files"
    );

    let _ = std::fs::remove_dir_all(&tmp);
}