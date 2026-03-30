/// SHOULD generate tests that exercise boundary conditions and edge cases for the invariant
#[test]
fn test_generator__temporal_obligation_generation__must_always_invariant_tests__should_generate_tests_that_exercise_boundary_con() {
    use ought_gen::context::GenerationContext;
    use ought_gen::generator::Language;
    use ought_gen::providers::build_prompt;
    use ought_spec::{Clause, ClauseId, Keyword, Severity, SourceLocation, Temporal};
    use std::path::PathBuf;

    let clause = Clause {
        id: ClauseId("gen::temporal::must_always_non_negative".to_string()),
        keyword: Keyword::MustAlways,
        severity: Severity::Required,
        text: "the result count must always be non-negative".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: Some(Temporal::Invariant),
        hints: vec![],
        source_location: SourceLocation {
            file: PathBuf::from("spec.ought.md"),
            line: 22,
        },
        content_hash: "def456".to_string(),
    };
    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };

    let prompt = build_prompt(&clause, &context);

    let mentions_boundaries = prompt.contains("boundary")
        || prompt.contains("edge case")
        || prompt.contains("corner case")
        || prompt.contains("edge-case");

    assert!(
        mentions_boundaries,
        "should_generate_tests_that_exercise_boundary_con: \
         build_prompt for a MUST ALWAYS (Invariant) clause should mention boundary conditions \
         and edge cases so the LLM exercises them. Prompt:\n{prompt}"
    );
}