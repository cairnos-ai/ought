/// MUST NOT generate OTHERWISE tests that depend on real infrastructure failures
/// (simulate the failure condition in-process)
#[test]
fn test_generator__otherwise_chain_generation__must_not_generate_otherwise_tests_that_depend_on_real_infrastructure() {
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

    let ow1 = mk(
        "gen::oc::otherwise_cached",
        Keyword::Otherwise,
        "return a cached response",
        vec![],
    );
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

    // The prompt must explicitly instruct the LLM to simulate failure conditions
    // in-process (e.g. via mocks, stubs, or injected errors) rather than relying
    // on real network timeouts, disk failures, or service unavailability.
    let prompt_lower = prompt.to_lowercase();
    let instructs_in_process_simulation = prompt_lower.contains("in-process")
        || prompt_lower.contains("mock")
        || prompt_lower.contains("stub")
        || (prompt_lower.contains("simulate") && prompt_lower.contains("otherwise"))
        || prompt_lower.contains("real infrastructure")
        || prompt_lower.contains("inject");

    assert!(
        instructs_in_process_simulation,
        "the batch prompt must instruct the LLM to simulate OTHERWISE failure conditions \
         in-process (via mocks, stubs, or injected errors) rather than depending on real \
         infrastructure failures such as network timeouts or service outages. \
         No such instruction found in prompt:\n{}",
        prompt
    );
}