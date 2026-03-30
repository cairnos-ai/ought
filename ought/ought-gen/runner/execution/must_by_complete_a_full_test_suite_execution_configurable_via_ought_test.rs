/// MUST BY 300s complete a full test suite execution (configurable via `ought.toml`)
#[test]
fn test_runner__execution__must_by_complete_a_full_test_suite_execution_configurable_via_ought() {
    use std::path::{Path, PathBuf};
    use std::time::{Duration, Instant};
    use ought_run::{Runner, RunResult, TestResult, TestStatus, TestDetails};
    use ought_gen::GeneratedTest;
    use ought_spec::ClauseId;

    // A realistic runner that returns promptly — simulates a well-behaved harness.
    struct TimedRunner {
        simulated_duration: Duration,
    }
    impl Runner for TimedRunner {
        fn run(&self, tests: &[GeneratedTest], _test_dir: &Path) -> anyhow::Result<RunResult> {
            let results = tests.iter().map(|t| TestResult {
                clause_id: t.clause_id.clone(),
                status: TestStatus::Passed,
                message: None,
                duration: self.simulated_duration,
                details: TestDetails { measured_duration: Some(self.simulated_duration), ..Default::default() },
            }).collect();
            Ok(RunResult {
                results,
                total_duration: self.simulated_duration,
            })
        }
        fn is_available(&self) -> bool { true }
        fn name(&self) -> &str { "timed" }
    }

    let deadline = Duration::from_secs(300); // MUST BY deadline from the spec
    let tmp = std::env::temp_dir()
        .join(format!("ought_mustby_{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    let tests: Vec<GeneratedTest> = (0..5).map(|i| GeneratedTest {
        clause_id: ClauseId(format!("runner::execution::clause_{i}")),
        code: String::new(),
        language: ought_gen::generator::Language::Rust,
        file_path: PathBuf::from(format!("clause_{i}_test.rs")),
    }).collect();

    let wall_start = Instant::now();
    let result = TimedRunner { simulated_duration: Duration::from_millis(1) }
        .run(&tests, &tmp)
        .expect("runner must succeed");
    let wall_elapsed = wall_start.elapsed();

    // The suite must complete within the 300s MUST BY deadline.
    assert!(
        wall_elapsed < deadline,
        "full test suite execution must complete within 300s; elapsed: {wall_elapsed:?}"
    );

    // RunResult must carry a total_duration so the caller can enforce the deadline.
    assert!(
        result.total_duration <= deadline,
        "RunResult::total_duration must be within the 300s MUST BY deadline; got: {:?}",
        result.total_duration
    );

    // Per-result measured_duration must be populated for MUST BY clauses.
    for r in &result.results {
        assert!(
            r.details.measured_duration.is_some(),
            "each MUST BY clause result must carry TestDetails::measured_duration; \
             missing for clause {:?}", r.clause_id
        );
        let measured = r.details.measured_duration.unwrap();
        assert!(
            measured <= deadline,
            "per-clause measured_duration must be within the 300s deadline; \
             clause {:?} reported {measured:?}", r.clause_id
        );
    }

    let _ = std::fs::remove_dir_all(&tmp);
}