/// MUST generate tests that assert the operation completes within the specified duration
#[test]
fn test_generator__temporal_obligation_generation__must_by_deadline_tests__must_generate_tests_that_assert_the_operation_complet() {
    use ought_gen::context::GenerationContext;
    use ought_gen::generator::Language;
    use ought_gen::providers::build_prompt;
    use ought_spec::{Clause, ClauseId, Keyword, Severity, SourceLocation, Temporal};
    use std::path::PathBuf;
    use std::time::Duration;

    let clause = Clause {
        id: ClauseId("api::handler::must_respond_within_deadline".to_string()),
        keyword: Keyword::MustBy,
        severity: Severity::Required,
        text: "the request handler must respond within the deadline".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: Some(Temporal::Deadline(Duration::from_millis(500))),
        hints: vec![],
        source_location: SourceLocation {
            file: PathBuf::from("spec.ought.md"),
            line: 10,
        },
        content_hash: "x".to_string(),
    };
    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };

    let prompt = build_prompt(&clause, &context);

    let instructs_deadline_assertion = prompt.contains("completes within")
        || prompt.contains("within this duration")
        || prompt.contains("within the deadline")
        || prompt.contains("deadline");

    assert!(
        instructs_deadline_assertion,
        "must_generate_tests_that_assert_the_operation_complet: \
         build_prompt for a MUST BY (Deadline) clause must instruct the LLM to generate \
         a test asserting the operation completes within the specified duration, \
         but no such instruction was found. Prompt:\n{prompt}"
    );
}