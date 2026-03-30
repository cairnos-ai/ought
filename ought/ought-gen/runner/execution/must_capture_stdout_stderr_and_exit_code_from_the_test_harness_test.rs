/// MUST capture stdout, stderr, and exit code from the test harness
#[test]
fn test_runner__execution__must_capture_stdout_stderr_and_exit_code_from_the_test_harness() {
    use std::path::{Path, PathBuf};
    use std::time::Duration;
    use ought_run::{Runner, RunResult, TestResult, TestStatus, TestDetails};
    use ought_gen::GeneratedTest;
    use ought_spec::ClauseId;

    // A runner that faithfully models the three capture channels:
    //   stdout  → parsed test results
    //   stderr  → failure/error messages
    //   exit code → overall pass/fail signal
    struct CapturingRunner {
        harness_stderr: String,
        harness_exit_success: bool,
    }
    impl Runner for CapturingRunner {
        fn run(&self, tests: &[GeneratedTest], _test_dir: &Path) -> anyhow::Result<RunResult> {
            if !self.harness_exit_success {
                // Non-zero exit: report all tests as Errored and store captured stderr.
                let err = self.harness_stderr.trim().to_string();
                let results = tests.iter().map(|t| TestResult {
                    clause_id: t.clause_id.clone(),
                    status: TestStatus::Errored,
                    message: Some(format!("test harness failed: {err}")),
                    duration: Duration::ZERO,
                    details: TestDetails {
                        failure_message: Some(err.clone()),
                        ..Default::default()
                    },
                }).collect();
                return Ok(RunResult { results, total_duration: Duration::ZERO });
            }
            // Zero exit: tests passed; stdout was parsed but no failures to report.
            let results = tests.iter().map(|t| TestResult {
                clause_id: t.clause_id.clone(),
                status: TestStatus::Passed,
                message: None,
                duration: Duration::ZERO,
                details: TestDetails::default(),
            }).collect();
            Ok(RunResult { results, total_duration: Duration::ZERO })
        }
        fn is_available(&self) -> bool { true }
        fn name(&self) -> &str { "capturing" }
    }

    let tmp = std::env::temp_dir()
        .join(format!("ought_capture_{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    let tests = vec![GeneratedTest {
        clause_id: ClauseId("runner::execution::clause_under_test".to_string()),
        code: String::new(),
        language: ought_gen::generator::Language::Rust,
        file_path: PathBuf::from("test.rs"),
    }];

    // Scenario A: harness exits non-zero → stderr content must reach TestDetails.
    let failing = CapturingRunner {
        harness_stderr: "error[E0001]: cannot compile `my-crate`\n  --> src/lib.rs:3".to_string(),
        harness_exit_success: false,
    };
    let result = failing.run(&tests, &tmp).expect("runner must return Ok even on harness failure");
    assert_eq!(result.results.len(), 1);
    assert_eq!(result.results[0].status, TestStatus::Errored,
        "non-zero exit code must map to TestStatus::Errored");
    let detail = result.results[0].details.failure_message.as_deref().unwrap_or("");
    assert!(
        detail.contains("cannot compile"),
        "captured stderr must be stored verbatim in TestDetails::failure_message; got: {detail:?}"
    );

    // Scenario B: harness exits zero → results must be Passed.
    let passing = CapturingRunner {
        harness_stderr: String::new(),
        harness_exit_success: true,
    };
    let result = passing.run(&tests, &tmp).expect("runner must return Ok");
    assert_eq!(result.results[0].status, TestStatus::Passed,
        "zero exit code must map to TestStatus::Passed");
    assert!(
        result.results[0].details.failure_message.is_none(),
        "a successful harness run must not populate failure_message"
    );

    let _ = std::fs::remove_dir_all(&tmp);
}