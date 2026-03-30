/// MUST include the free-text `context:` block
#[test]
fn test_generator__context_assembly__must_include_the_free_text_context_block() {
    use std::path::PathBuf;
    use std::collections::HashMap;
    use ought_spec::types::{Clause, ClauseId, Keyword, Severity, Spec, Metadata, SourceLocation};
    use ought_spec::config::{Config, ProjectConfig, SpecsConfig, ContextConfig, GeneratorConfig, ToleranceConfig, McpConfig};
    use ought_gen::context::{ContextAssembler, GenerationContext};
    use ought_gen::generator::Language;
    use ought_gen::providers::build_prompt;

    let tmp = std::env::temp_dir().join("ought_ctxasm_context_block");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    let free_text = "This service handles payment processing for subscriptions.";

    let spec = Spec {
        name: "Payments".to_string(),
        metadata: Metadata {
            context: Some(free_text.to_string()),
            sources: vec![],
            schemas: vec![],
            requires: vec![],
        },
        sections: vec![],
        source_path: tmp.join("payments.ought.md"),
    };

    let clause = Clause {
        id: ClauseId("payments::must_charge_correct_amount".to_string()),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: "charge the correct subscription amount".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("payments.ought.md"), line: 1 },
        content_hash: "jkl000".to_string(),
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

    // assemble must propagate the free-text context to spec_context
    assert_eq!(
        ctx.spec_context.as_deref(),
        Some(free_text),
        "spec_context must equal the context: block from spec metadata"
    );

    // build_prompt must embed the context block in the outgoing LLM prompt
    let prompt = build_prompt(&clause, &ctx);
    assert!(
        prompt.contains(free_text),
        "prompt must include the free-text context block; got:\n{prompt}"
    );
    assert!(
        prompt.contains("## Context"),
        "prompt must have a Context section header; got:\n{prompt}"
    );

    let _ = std::fs::remove_dir_all(&tmp);
}