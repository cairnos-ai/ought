/// SHOULD generate tests that run the operation multiple times and assert the p99 latency is within the deadline
#[test]
fn test_generator__temporal_obligation_generation__must_by_deadline_tests__should_generate_tests_that_run_the_operation_multiple() {
    use ought_gen::context::GenerationContext;
    use ought_gen::generator::{ClauseGroup, Language};
    use ought_gen::providers::build_batch_prompt;
    use ought_spec::{Clause, ClauseId, Keyword, Severity, SourceLocation, Temporal};
    use std::path::PathBuf;
    use std::time::Duration;

    let clause = Clause {
        id: ClauseId("index::search::must_respond_within_100ms".to_string()),
        keyword: Keyword::MustBy,
        severity: Severity::Required,
        text: "search must respond within 100 milliseconds".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: Some(Temporal::Deadline(Duration::from_millis(100))),
        hints: vec![],
        source_location: SourceLocation {
            file: PathBuf::from("spec.ought.md"),
            line: 30,
        },
        content_hash: "x".to_string(),
    };

    let group = ClauseGroup {
        section_path: "Generator > Temporal Obligation Generation > MUST BY".to_string(),
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

    // A single-sample timing test is fragile — the prompt should guide the LLM to run
    // the operation multiple times and assert a high-percentile (p99) latency, which is
    // far more meaningful for catching flakes than a single measurement.
    let mentions_repeated_runs = prompt.contains("p99")
        || prompt.contains("percentile")
        || prompt.contains("multiple times")
        || prompt.contains("repeated")
        || prompt.contains("iterations");

    assert!(
        mentions_repeated_runs,
        "should_generate_tests_that_run_the_operation_multiple: \
         build_batch_prompt for a MUST BY clause should instruct the LLM to run the \
         operation multiple times and assert the p99 latency is within the deadline, \
         but the prompt contained no such instruction. Prompt:\n{prompt}"
    );
}