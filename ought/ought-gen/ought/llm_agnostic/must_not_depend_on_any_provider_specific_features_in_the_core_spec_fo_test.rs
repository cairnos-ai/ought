/// MUST NOT depend on any provider-specific features in the core spec format or runner
#[test]
fn test_ought__llm_agnostic__must_not_depend_on_any_provider_specific_features_in_the_core_spec_fo() {
    use std::path::PathBuf;
    use ought_gen::generator::{GeneratedTest, Language};
    use ought_spec::types::{Clause, ClauseId, Keyword, Severity, SourceLocation};
    use ought_run::runner::Runner;
    use ought_run::types::RunResult;

    // The core Clause type must carry no provider-specific fields.
    // Construct one and verify only provider-neutral fields exist.
    let clause = Clause {
        id: ClauseId("ought::llm_agnostic::core_format_clause".to_string()),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: "not depend on provider-specific features".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("spec.ought.md"), line: 1 },
        content_hash: "xyz".to_string(),
    };
    // If this compiles and runs, Clause has no mandatory provider fields.
    assert_eq!(clause.id.0, "ought::llm_agnostic::core_format_clause");

    // The GeneratedTest type must carry no provider-specific fields.
    let test = GeneratedTest {
        clause_id: clause.id.clone(),
        code: "#[test]\nfn test_x() { assert!(true); }".to_string(),
        language: Language::Rust,
        file_path: PathBuf::from("ought/llm_agnostic/core_format_clause_test.rs"),
    };
    assert_eq!(
        test.clause_id, clause.id,
        "GeneratedTest must only record clause_id, code, language, file_path — no provider field"
    );

    // The Runner trait must have no provider-specific methods.
    // Verify via a mock implementation: only run(), is_available(), and name() exist.
    struct NeutralRunner;
    impl Runner for NeutralRunner {
        fn run(&self, _tests: &[GeneratedTest], _test_dir: &std::path::Path) -> anyhow::Result<RunResult> {
            Ok(RunResult { results: vec![], total_duration: std::time::Duration::ZERO })
        }
        fn is_available(&self) -> bool { true }
        fn name(&self) -> &str { "neutral" }
    }

    let runner: Box<dyn Runner> = Box::new(NeutralRunner);
    assert_eq!(runner.name(), "neutral", "Runner::name must work without any provider knowledge");
    assert!(runner.is_available(), "Runner::is_available must work without any provider knowledge");

    // The runner accepts tests regardless of which provider generated them —
    // it only sees GeneratedTest values with language-neutral fields.
    let result = runner.run(&[test], &PathBuf::from("/tmp"))
        .expect("Runner must accept GeneratedTest without caring about the originating provider");
    assert_eq!(result.results.len(), 0, "NeutralRunner returns empty results as expected");
}