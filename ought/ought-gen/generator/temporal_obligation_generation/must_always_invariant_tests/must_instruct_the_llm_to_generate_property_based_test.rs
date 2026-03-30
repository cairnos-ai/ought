/// MUST instruct the LLM to generate property-based or fuzz-style tests for MUST ALWAYS clauses
#[test]
fn test_generator__temporal_obligation_generation__must_always_invariant_tests__must_instruct_the_llm_to_generate_property_based() {
    use ought_gen::context::GenerationContext;
    use ought_gen::generator::Language;
    use ought_gen::providers::build_prompt;
    use ought_spec::{Clause, ClauseId, Keyword, Severity, SourceLocation, Temporal};
    use std::path::PathBuf;

    let clause = Clause {
        id: ClauseId(
            "gen::temporal::must_always_output_valid_utf8".to_string(),
        ),
        keyword: Keyword::MustAlways,
        severity: Severity::Required,
        text: "the output must always be valid UTF-8".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: Some(Temporal::Invariant),
        hints: vec![],
        source_location: SourceLocation {
            file: PathBuf::from("spec.ought.md"),
            line: 10,
        },
        content_hash: "abc123".to_string(),
    };
    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };

    let prompt = build_prompt(&clause, &context);

    assert!(
        prompt.contains("property-based") || prompt.contains("fuzz-style") || prompt.contains("fuzz"),
        "must_instruct_the_llm_to_generate_property_based: \
         build_prompt for a MUST ALWAYS (Invariant) clause must tell the LLM to generate \
         property-based or fuzz-style tests, but the prompt contained neither. Prompt:\n{prompt}"
    );
}