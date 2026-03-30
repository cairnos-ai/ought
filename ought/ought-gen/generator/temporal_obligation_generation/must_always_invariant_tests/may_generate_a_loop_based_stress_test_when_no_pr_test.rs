/// MAY generate a loop-based stress test when no property testing library is available
#[test]
fn test_generator__temporal_obligation_generation__must_always_invariant_tests__may_generate_a_loop_based_stress_test_when_no_pr() {
    use ought_gen::context::GenerationContext;
    use ought_gen::generator::Language;
    use ought_gen::providers::build_prompt;
    use ought_spec::{Clause, ClauseId, Keyword, Severity, SourceLocation, Temporal};
    use std::path::PathBuf;

    let clause = Clause {
        id: ClauseId("gen::temporal::must_always_deterministic".to_string()),
        keyword: Keyword::MustAlways,
        severity: Severity::Required,
        text: "the hash function must always be deterministic".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: Some(Temporal::Invariant),
        hints: vec![],
        source_location: SourceLocation {
            file: PathBuf::from("spec.ought.md"),
            line: 40,
        },
        content_hash: "jkl012".to_string(),
    };
    // Go has no idiomatic property testing library; a loop-based stress test is the expected fallback.
    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Go,
        verbose: false,
    };

    let prompt = build_prompt(&clause, &context);

    // The prompt should acknowledge loop-based stress testing as a valid fallback
    // for languages/contexts where no property testing library is readily available.
    let mentions_loop_fallback = prompt.contains("loop")
        || prompt.contains("stress test")
        || prompt.contains("stress-test")
        || prompt.contains("for loop")
        || prompt.contains("iterate over");

    assert!(
        mentions_loop_fallback,
        "may_generate_a_loop_based_stress_test_when_no_pr: \
         build_prompt for a MUST ALWAYS clause targeting a language with no standard property \
         testing library (Go) should mention a loop-based stress test as a fallback strategy. \
         Prompt:\n{prompt}"
    );
}