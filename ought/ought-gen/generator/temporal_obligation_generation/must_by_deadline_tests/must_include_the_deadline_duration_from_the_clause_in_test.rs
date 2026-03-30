/// MUST include the deadline duration from the clause in the test's timeout/assertion
#[test]
fn test_generator__temporal_obligation_generation__must_by_deadline_tests__must_include_the_deadline_duration_from_the_clause_in() {
    use ought_gen::context::GenerationContext;
    use ought_gen::generator::Language;
    use ought_gen::providers::{build_batch_prompt, build_prompt};
    use ought_gen::generator::ClauseGroup;
    use ought_spec::{Clause, ClauseId, Keyword, Severity, SourceLocation, Temporal};
    use std::path::PathBuf;
    use std::time::Duration;

    fn mk_deadline(id: &str, text: &str, ms: u64) -> Clause {
        Clause {
            id: ClauseId(id.to_string()),
            keyword: Keyword::MustBy,
            severity: Severity::Required,
            text: text.to_string(),
            condition: None,
            otherwise: vec![],
            temporal: Some(Temporal::Deadline(Duration::from_millis(ms))),
            hints: vec![],
            source_location: SourceLocation {
                file: PathBuf::from("spec.ought.md"),
                line: 1,
            },
            content_hash: "x".to_string(),
        }
    }

    // Use a distinctive duration — the numeric value must survive into the prompt
    // so the LLM can embed it in the generated assertion.
    let clause_250 = mk_deadline(
        "search::must_respond_within_250ms",
        "search must respond within 250 milliseconds",
        250,
    );
    let clause_2000 = mk_deadline(
        "export::must_complete_within_2s",
        "export must complete within 2 seconds",
        2000,
    );

    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };

    // Single-clause path
    let prompt_250 = build_prompt(&clause_250, &context);
    assert!(
        prompt_250.contains("250"),
        "must_include_the_deadline_duration_from_the_clause_in: \
         build_prompt for a MUST BY clause with a 250 ms deadline must embed that duration \
         value in the prompt so the LLM can use it in the generated assertion. \
         Prompt:\n{prompt_250}"
    );

    // The two clauses must produce different prompts so the LLM generates distinct assertions.
    let prompt_2000 = build_prompt(&clause_2000, &context);
    assert!(
        prompt_2000.contains("2000") || prompt_2000.contains("2s") || prompt_2000.contains("2 s"),
        "must_include_the_deadline_duration_from_the_clause_in: \
         build_prompt for a MUST BY clause with a 2000 ms deadline must embed that duration \
         value in the prompt. Prompt:\n{prompt_2000}"
    );
    assert_ne!(
        prompt_250, prompt_2000,
        "must_include_the_deadline_duration_from_the_clause_in: \
         prompts for MUST BY clauses with different deadlines must differ so the LLM \
         generates assertions with the correct duration for each clause"
    );

    // Batch path: the duration must also appear when clauses are batched together.
    let group = ClauseGroup {
        section_path: "Generator > Temporal Obligation Generation > MUST BY".to_string(),
        clauses: vec![&clause_250, &clause_2000],
        conditions: vec![],
    };
    let batch = build_batch_prompt(&group, &context);
    assert!(
        batch.contains("250"),
        "must_include_the_deadline_duration_from_the_clause_in: \
         build_batch_prompt must include the 250 ms deadline value so both generated \
         assertions reference their correct durations. Batch prompt:\n{batch}"
    );
}