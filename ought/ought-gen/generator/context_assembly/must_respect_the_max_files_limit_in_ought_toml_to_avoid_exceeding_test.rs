/// MUST respect the `max_files` limit in `ought.toml` to avoid exceeding LLM context
#[test]
fn test_generator__context_assembly__must_respect_the_max_files_limit_in_ought_toml_to_avoid_exceeding() {
    use std::path::PathBuf;
    use std::collections::HashMap;
    use ought_spec::types::{Clause, ClauseId, Keyword, Severity, SourceLocation};
    use ought_spec::config::{Config, ProjectConfig, SpecsConfig, ContextConfig, GeneratorConfig, ToleranceConfig, McpConfig};
    use ought_gen::context::ContextAssembler;

    let tmp = std::env::temp_dir().join("ought_ctxasm_max_files");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    // Write 10 files all containing the keyword "authenticate" so every file scores > 0
    for i in 0..10 {
        let content = format!("fn authenticate_{i}() {{ /* authenticate user #{i} */ }}");
        std::fs::write(tmp.join(format!("module_{i}.rs")), content).unwrap();
    }

    let max_files: usize = 3;
    let config = Config {
        project: ProjectConfig { name: "test".to_string(), version: "0.1.0".to_string() },
        specs: SpecsConfig::default(),
        context: ContextConfig {
            search_paths: vec![tmp.clone()],
            exclude: vec![],
            max_files,
        },
        generator: GeneratorConfig { provider: "claude".to_string(), model: None, tolerance: ToleranceConfig::default() },
        runner: HashMap::new(),
        mcp: McpConfig::default(),
    };

    let clause = Clause {
        id: ClauseId("auth::must_authenticate".to_string()),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: "authenticate the user before granting access".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("auth.ought.md"), line: 1 },
        content_hash: "maxf222".to_string(),
    };

    let assembler = ContextAssembler::new(&config);
    let discovered = assembler.discover_sources(&clause).expect("discover_sources failed");

    assert!(
        discovered.len() <= max_files,
        "discover_sources must not return more than max_files ({max_files}) files; got {}",
        discovered.len()
    );

    let _ = std::fs::remove_dir_all(&tmp);
}