/// MUST generate tests that verify the invariant holds across multiple inputs, states, or iterations
#[test]
fn test_generator__temporal_obligation_generation__must_always_invariant_tests__must_generate_tests_that_verify_the_invariant_ho() {
    use ought_gen::context::GenerationContext;
    use ought_gen::generator::{ClauseGroup, Language};
    use ought_gen::providers::build_batch_prompt;
    use ought_spec::{Clause, ClauseId, Keyword, Severity, SourceLocation, Temporal};
    use std::path::PathBuf;

    fn mk_invariant(id: &str, text: &str) -> Clause {
        Clause {
            id: ClauseId(id.to_string()),
            keyword: Keyword::MustAlways,
            severity: Severity::Required,
            text: text.to_string(),
            condition: None,
            otherwise: vec![],
            temporal: Some(Temporal::Invariant),
            hints: vec![],
            source_location: SourceLocation {
                file: PathBuf::from("spec.ought.md"),
                line: 1,
            },
            content_hash: "x".to_string(),
        }
    }

    let clause = mk_invariant(
        "gen::temporal::must_always_idempotent",
        "serialization must always be idempotent across inputs",
    );
    let group = ClauseGroup {
        section_path: "Generator > Temporal Obligation Generation > MUST ALWAYS".to_string(),
        clauses: vec![&clause],
        conditions: vec![],
    };
    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };

    let prompt = build_batch_prompt(&group, &context);

    // "property-based" is accepted as it inherently exercises multiple inputs;
    // the prompt may also explicitly name "multiple inputs", "states", or "iterations".
    let covers_multiple_inputs = prompt.contains("multiple inputs")
        || prompt.contains("multiple states")
        || prompt.contains("iterations")
        || prompt.contains("property-based")
        || prompt.contains("fuzz");

    assert!(
        covers_multiple_inputs,
        "must_generate_tests_that_verify_the_invariant_ho: \
         build_batch_prompt for a MUST ALWAYS clause must instruct the LLM to verify the \
         invariant across multiple inputs, states, or iterations (or mention property-based / \
         fuzz testing, which implies this). Prompt:\n{prompt}"
    );
}