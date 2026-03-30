/// MUST generate a test for the primary obligation (the parent clause)
#[test]
fn test_generator__otherwise_chain_generation__must_generate_a_test_for_the_primary_obligation_the_parent_clause() {
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
    let parent = mk("gen::oc::must_respond_fast", Keyword::Must, "respond within 200ms", vec![ow1, ow2]);

    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };

    let group = ClauseGroup {
        section_path: "Generator > OTHERWISE Chain Generation".to_string(),
        clauses: vec![&parent],
        conditions: vec![],
    };

    let prompt = build_batch_prompt(&group, &context);

    assert!(
        prompt.contains("respond within 200ms"),
        "primary obligation text must appear in the batch prompt as a testable clause; \
         the parent clause must not be silently dropped when OTHERWISE children are present"
    );
    assert!(
        prompt.contains("gen::oc::must_respond_fast"),
        "primary obligation clause ID must appear in the batch prompt so the LLM can emit the correct marker"
    );
}