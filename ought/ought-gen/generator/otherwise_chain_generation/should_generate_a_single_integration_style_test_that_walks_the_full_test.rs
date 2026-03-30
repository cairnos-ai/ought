/// SHOULD generate a single integration-style test that walks the full degradation chain in sequence
#[test]
fn test_generator__otherwise_chain_generation__should_generate_a_single_integration_style_test_that_walks_the_full() {
    use ought_gen::context::GenerationContext;
    use ought_gen::generator::{ClauseGroup, Language};
    use ought_gen::providers::build_batch_prompt;
    use ought_spec::{Clause, ClauseId, Keyword, Severity, SourceLocation};
    use std::path::PathBuf;

    fn mk(id: &str, kw: Keyword, text: &str, otherwise: Vec<Clause>) -> Clause {
        Clause {
            id: ClauseId(id.to_string()),
            keyword: kw,
            severity: Severity::Required,
            text: text.to_string(),
            condition: None,
            otherwise,
            temporal: None,
            hints: vec![],
            source_location: SourceLocation { file: PathBuf::from("spec.ought.md"), line: 1 },
            content_hash: "h".to_string(),
        }
    }

    let ow1 = mk("gen::oc::otherwise_cached", Keyword::Otherwise, "return a cached response", vec![]);
    let ow2 = mk("gen::oc::otherwise_504", Keyword::Otherwise, "return 504 Gateway Timeout", vec![]);
    let parent = mk(
        "gen::oc::must_respond_fast",
        Keyword::Must,
        "respond within 200ms",
        vec![ow1.clone(), ow2.clone()],
    );

    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };

    let group = ClauseGroup {
        section_path: "Generator > OTHERWISE Chain Generation".to_string(),
        clauses: vec![&parent, &ow1, &ow2],
        conditions: vec![],
    };

    let prompt = build_batch_prompt(&group, &context);

    // When a clause has an OTHERWISE chain, the prompt should also ask the LLM
    // to generate an integration-style test that exercises the full degradation path
    // in a single test (primary fails → first fallback fires → first fallback fails → second fires).
    let prompt_lower = prompt.to_lowercase();
    let has_integration_instruction = prompt_lower.contains("integration")
        || prompt_lower.contains("degradation chain")
        || prompt_lower.contains("full chain")
        || prompt_lower.contains("walks the full")
        || prompt_lower.contains("end-to-end");

    assert!(
        has_integration_instruction,
        "when a clause has an OTHERWISE chain, the prompt should instruct the LLM to generate \
         a single integration-style test that walks the full degradation sequence in order. \
         No such instruction found in prompt:\n{}",
        prompt
    );
}