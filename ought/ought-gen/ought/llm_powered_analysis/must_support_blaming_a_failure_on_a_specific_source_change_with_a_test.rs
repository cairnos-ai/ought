/// MUST support blaming a failure on a specific source change with a causal narrative (`ought blame`)
#[test]
fn test_ought__llm_powered_analysis__must_support_blaming_a_failure_on_a_specific_source_change_with_a() {
    use std::fs;

    struct StubBlameGenerator;
    impl Generator for StubBlameGenerator {
        fn generate(&self, _: &Clause, _: &GenerationContext) -> anyhow::Result<GeneratedTest> {
            Ok(GeneratedTest {
                clause_id: ClauseId("blame::stub".to_string()),
                code: r#"{"narrative":"The payments::checkout::must_reject_expired_cards clause began failing after commit a3f9c12 changed `validate_expiry` to always return true, bypassing the expiry check. Author: Bob Refactorer <bob@example.com>. The diff shows removal of the `if expiry < now` branch in src/payments.rs:47.","suggested_fix":"Restore the expiry check removed in a3f9c12 or add a new validation path."}"#.to_string(),
                language: Language::Rust,
                file_path: PathBuf::from("_blame.json"),
            })
        }
    }

    let clause_id = ClauseId("payments::checkout::must_reject_expired_cards".to_string());
    let base = std::env::temp_dir()
        .join(format!("ought_blame_capability_{}", std::process::id()));
    let spec_dir = base.join("specs");
    fs::create_dir_all(&spec_dir).unwrap();

    fs::write(
        spec_dir.join("payments.ought.md"),
        "# Payments\n\n## Checkout\n\n- **MUST** reject expired cards\n",
    ).unwrap();

    let specs = SpecGraph::from_roots(&[spec_dir.clone()]).expect("spec graph should parse");

    let run_result = RunResult {
        results: vec![TestResult {
            clause_id: clause_id.clone(),
            status: TestStatus::Failed,
            message: Some("expired card was not rejected".to_string()),
            duration: Duration::ZERO,
            details: TestDetails {
                failure_message: Some("expected Err got Ok".to_string()),
                stack_trace: None,
                iterations: None,
                measured_duration: None,
            },
        }],
        total_duration: Duration::ZERO,
    };

    let res = blame(&clause_id, &specs, &run_result, &StubBlameGenerator);
    assert!(
        res.is_ok(),
        "ought blame must be supported and return Ok; err: {:?}",
        res.err()
    );

    let result = res.unwrap();
    // blame must produce a non-empty causal narrative.
    assert!(
        !result.narrative.is_empty(),
        "ought blame must produce a causal narrative explaining what broke and why; got empty string"
    );
    // The narrative must tie the failure to a specific source change.
    assert!(
        result.narrative.contains("commit")
            || result.narrative.contains("change")
            || result.narrative.contains("diff")
            || result.narrative.contains("src/"),
        "blame narrative must reference a specific source change; got: {:?}",
        result.narrative
    );
    // The result must carry back the same clause id it was asked about.
    assert_eq!(
        result.clause_id, clause_id,
        "blame result must carry the clause_id that was passed in"
    );

    let _ = fs::remove_dir_all(&base);
}