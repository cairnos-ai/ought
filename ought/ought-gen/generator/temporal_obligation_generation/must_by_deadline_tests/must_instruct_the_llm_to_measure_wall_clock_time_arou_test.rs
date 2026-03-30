/// MUST instruct the LLM to measure wall-clock time around the operation under test
#[test]
fn test_generator__temporal_obligation_generation__must_by_deadline_tests__must_instruct_the_llm_to_measure_wall_clock_time_arou() {
    use ought_gen::context::GenerationContext;
    use ought_gen::generator::Language;
    use ought_gen::providers::build_prompt;
    use ought_spec::{Clause, ClauseId, Keyword, Severity, SourceLocation, Temporal};
    use std::path::PathBuf;
    use std::time::Duration;

    let clause = Clause {
        id: ClauseId("processor::must_finish_within_1s".to_string()),
        keyword: Keyword::MustBy,
        severity: Severity::Required,
        text: "the processor must finish within 1 second".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: Some(Temporal::Deadline(Duration::from_secs(1))),
        hints: vec![],
        source_location: SourceLocation {
            file: PathBuf::from("spec.ought.md"),
            line: 20,
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

    // The prompt must explicitly tell the LLM to capture time before and after the call
    // so the generated test measures real elapsed (wall-clock) duration, not CPU time.
    let instructs_wall_clock = prompt.contains("wall-clock")
        || prompt.contains("wall clock")
        || prompt.contains("Instant")
        || prompt.contains("elapsed")
        || prompt.contains("measure");

    assert!(
        instructs_wall_clock,
        "must_instruct_the_llm_to_measure_wall_clock_time_arou: \
         build_prompt for a MUST BY clause must tell the LLM to measure wall-clock time \
         (e.g. Instant::now() / .elapsed()) around the operation under test, \
         but no such instruction was found. Prompt:\n{prompt}"
    );
}