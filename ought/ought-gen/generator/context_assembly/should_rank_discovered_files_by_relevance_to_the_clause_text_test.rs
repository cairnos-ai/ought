/// SHOULD rank discovered files by relevance to the clause text
#[test]
fn test_generator__context_assembly__should_rank_discovered_files_by_relevance_to_the_clause_text() {
    use std::path::PathBuf;
    use std::collections::HashMap;
    use ought_spec::types::{Clause, ClauseId, Keyword, Severity, SourceLocation};
    use ought_spec::config::{Config, ProjectConfig, SpecsConfig, ContextConfig, GeneratorConfig, ToleranceConfig, McpConfig};
    use ought_gen::context::ContextAssembler;

    let tmp = std::env::temp_dir().join("ought_ctxasm_ranking");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    // "high_match.rs" mentions all three long keywords from the clause text
    std::fs::write(
        tmp.join("high_match.rs"),
        "fn process_payment(invoice: &Invoice) -> Receipt { \
         // validate invoice, charge payment, generate receipt \
         todo!() }",
    )
    .unwrap();

    // "low_match.rs" mentions only one keyword
    std::fs::write(
        tmp.join("low_match.rs"),
        "fn noop() { /* invoice placeholder */ }",
    )
    .unwrap();

    // "no_match.rs" mentions none of the keywords
    std::fs::write(
        tmp.join("no_match.rs"),
        "fn unrelated() -> bool { true }",
    )
    .unwrap();

    let clause = Clause {
        id: ClauseId("billing::must_process_payment".to_string()),
        keyword: Keyword::Must,
        severity: Severity::Required,
        // All three long words appear in high_match.rs: "invoice", "charge", "receipt"
        text: "process payment for the invoice and generate receipt".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("billing.ought.md"), line: 1 },
        content_hash: "rank444".to_string(),
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
    let discovered = assembler.discover_sources(&clause).expect("discover_sources failed");

    // Files with zero score are excluded; remaining should be ranked best-first
    assert!(
        !discovered.is_empty(),
        "at least one file should match the clause keywords"
    );

    // high_match.rs must come before low_match.rs
    let high_pos = discovered.iter().position(|f| f.path.ends_with("high_match.rs"));
    let low_pos = discovered.iter().position(|f| f.path.ends_with("low_match.rs"));

    assert!(
        high_pos.is_some(),
        "high_match.rs should be discovered (it mentions multiple clause keywords)"
    );
    assert!(
        low_pos.is_some(),
        "low_match.rs should be discovered (it mentions at least one clause keyword)"
    );
    assert!(
        high_pos.unwrap() < low_pos.unwrap(),
        "high_match.rs (more keyword matches) must rank before low_match.rs (fewer matches)"
    );

    // no_match.rs must not appear — zero score means excluded
    assert!(
        !discovered.iter().any(|f| f.path.ends_with("no_match.rs")),
        "files with no keyword matches must not appear in discovered sources"
    );

    let _ = std::fs::remove_dir_all(&tmp);
}