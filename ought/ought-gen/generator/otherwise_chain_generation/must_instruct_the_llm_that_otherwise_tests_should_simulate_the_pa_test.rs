/// MUST instruct the LLM that OTHERWISE tests should simulate the parent obligation's failure
/// condition, then verify the fallback behavior activates
#[test]
fn test_generator__otherwise_chain_generation__must_instruct_the_llm_that_otherwise_tests_should_simulate_the_pa() {
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
    let parent = mk(
        "gen::oc::must_respond_fast",
        Keyword::Must,
        "respond within 200ms",
        vec![ow1.clone()],
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
        clauses: vec![&parent, &ow1],
        conditions: vec![],
    };

    let prompt = build_batch_prompt(&group, &context);

    // The prompt must explicitly tell the LLM how to structure an OTHERWISE test:
    // simulate the parent failing, then assert the fallback activates.
    let prompt_lower = prompt.to_lowercase();
    let instructs_failure_simulation = prompt_lower.contains("simulate")
        || prompt_lower.contains("failure condition")
        || prompt_lower.contains("parent")
            && (prompt_lower.contains("fail") || prompt_lower.contains("trigger"));
    assert!(
        instructs_failure_simulation,
        "the batch prompt must instruct the LLM that an OTHERWISE test should simulate the \
         parent obligation's failure condition and then verify the fallback behavior activates. \
         No such instruction found. Prompt:\n{}",
        prompt
    );
}