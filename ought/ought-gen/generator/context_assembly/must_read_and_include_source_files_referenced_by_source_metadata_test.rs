/// MUST read and include source files referenced by `source:` metadata
#[test]
fn test_generator__context_assembly__must_read_and_include_source_files_referenced_by_source_metadata() {
    use std::path::PathBuf;
    use std::collections::HashMap;
    use ought_spec::types::{Clause, ClauseId, Keyword, Severity, Spec, Metadata, SourceLocation, Section};
    use ought_spec::config::{Config, ProjectConfig, SpecsConfig, ContextConfig, GeneratorConfig, ToleranceConfig, McpConfig};
    use ought_gen::context::ContextAssembler;

    let tmp = std::env::temp_dir().join("ought_ctxasm_source_meta");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    // Write a fake source file
    let src_content = "fn authenticate(token: &str) -> bool { token == \"secret\" }";
    std::fs::write(tmp.join("auth.rs"), src_content).unwrap();

    let spec = Spec {
        name: "Auth".to_string(),
        metadata: Metadata {
            context: None,
            sources: vec!["auth.rs".to_string()],
            schemas: vec![],
            requires: vec![],
        },
        sections: vec![],
        source_path: tmp.join("auth.ought.md"),
    };

    let clause = Clause {
        id: ClauseId("auth::must_validate_token".to_string()),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: "validate the token before granting access".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("auth.ought.md"), line: 1 },
        content_hash: "def456".to_string(),
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

    assert_eq!(ctx.source_files.len(), 1, "expected one source file from metadata");
    assert!(
        ctx.source_files[0].path.ends_with("auth.rs"),
        "source file path should be auth.rs; got {:?}",
        ctx.source_files[0].path
    );
    assert_eq!(
        ctx.source_files[0].content, src_content,
        "source file content must match what was written"
    );

    let _ = std::fs::remove_dir_all(&tmp);
}