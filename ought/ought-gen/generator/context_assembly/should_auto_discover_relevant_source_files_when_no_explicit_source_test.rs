/// SHOULD auto-discover relevant source files when no explicit `source:` is provided
#[test]
fn test_generator__context_assembly__should_auto_discover_relevant_source_files_when_no_explicit_source() {
    use std::path::PathBuf;
    use std::collections::HashMap;
    use ought_spec::types::{Clause, ClauseId, Keyword, Severity, Spec, Metadata, SourceLocation};
    use ought_spec::config::{Config, ProjectConfig, SpecsConfig, ContextConfig, GeneratorConfig, ToleranceConfig, McpConfig};
    use ought_gen::context::ContextAssembler;

    let tmp = std::env::temp_dir().join("ought_ctxasm_autodiscover");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    // Write a file that mentions words from the clause text
    std::fs::write(
        tmp.join("session.rs"),
        "fn refresh_session(token: &str) -> Option<Session> { /* refresh the session token */ None }",
    )
    .unwrap();

    // Spec has NO explicit sources
    let spec = Spec {
        name: "Session".to_string(),
        metadata: Metadata {
            context: None,
            sources: vec![],   // empty — triggers auto-discovery
            schemas: vec![],
            requires: vec![],
        },
        sections: vec![],
        source_path: tmp.join("session.ought.md"),
    };

    let clause = Clause {
        id: ClauseId("session::must_refresh_token".to_string()),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: "refresh the session token before expiry".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("session.ought.md"), line: 1 },
        content_hash: "auto333".to_string(),
    };

    let config = Config {
        project: ProjectConfig { name: "test".to_string(), version: "0.1.0".to_string() },
        specs: SpecsConfig::default(),
        context: ContextConfig {
            search_paths: vec![tmp.clone()],
            exclude: vec![],
            max_files: 50,
        },
        generator: GeneratorConfig { provider: "claude".to_string(), model: None, tolerance: ToleranceConfig::default() },
        runner: HashMap::new(),
        mcp: McpConfig::default(),
    };

    let assembler = ContextAssembler::new(&config);
    let ctx = assembler.assemble(&clause, &spec).expect("assemble failed");

    assert!(
        !ctx.source_files.is_empty(),
        "when no explicit source: is given, assembler should auto-discover relevant files"
    );
    assert!(
        ctx.source_files.iter().any(|f| f.path.ends_with("session.rs")),
        "auto-discovered files should include session.rs which mentions clause keywords"
    );

    let _ = std::fs::remove_dir_all(&tmp);
}