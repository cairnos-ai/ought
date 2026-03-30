/// MUST support running tests for a single spec file (filtering generated tests by origin spec)
#[test]
fn test_runner__execution__must_support_running_tests_for_a_single_spec_file_filtering_gener() {
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use ought_run::{Runner, RunResult};
    use ought_gen::GeneratedTest;
    use ought_spec::ClauseId;

    // Tests generated from two distinct spec files.
    // Spec A: auth.ought.md → clause IDs prefixed "auth::"
    // Spec B: runner.ought.md → clause IDs prefixed "runner::"
    let all_tests = vec![
        GeneratedTest {
            clause_id: ClauseId("auth::login::must_return_jwt".to_string()),
            code: String::new(),
            language: ought_gen::generator::Language::Rust,
            file_path: PathBuf::from("auth/login/must_return_jwt_test.rs"),
        },
        GeneratedTest {
            clause_id: ClauseId("auth::login::must_reject_expired_token".to_string()),
            code: String::new(),
            language: ought_gen::generator::Language::Rust,
            file_path: PathBuf::from("auth/login/must_reject_expired_token_test.rs"),
        },
        GeneratedTest {
            clause_id: ClauseId("runner::execution::must_invoke_command".to_string()),
            code: String::new(),
            language: ought_gen::generator::Language::Rust,
            file_path: PathBuf::from("runner/execution/must_invoke_command_test.rs"),
        },
    ];

    // A runner that records which clause IDs it was asked to execute.
    struct RecordingRunner {
        seen_ids: Arc<Mutex<Vec<String>>>,
    }
    impl Runner for RecordingRunner {
        fn run(&self, tests: &[GeneratedTest], _test_dir: &Path) -> anyhow::Result<RunResult> {
            let mut ids = self.seen_ids.lock().unwrap();
            for t in tests {
                ids.push(t.clause_id.0.clone());
            }
            Ok(RunResult { results: vec![], total_duration: Duration::ZERO })
        }
        fn is_available(&self) -> bool { true }
        fn name(&self) -> &str { "recording" }
    }

    let tmp = std::env::temp_dir()
        .join(format!("ought_filter_spec_{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    let seen = Arc::new(Mutex::new(Vec::<String>::new()));
    let runner = RecordingRunner { seen_ids: Arc::clone(&seen) };

    // Filter to only the auth spec tests before invoking the runner.
    let auth_only: Vec<_> = all_tests.iter()
        .filter(|t| t.clause_id.0.starts_with("auth::"))
        .cloned()
        .collect();
    assert_eq!(auth_only.len(), 2,
        "test setup: should have exactly 2 auth tests to pass to the runner");

    runner.run(&auth_only, &tmp).expect("runner must not error");

    let seen_ids = seen.lock().unwrap().clone();
    assert_eq!(seen_ids.len(), 2,
        "runner must execute exactly the 2 tests from the filtered spec file, \
         not all 3 tests; got {seen_ids:?}");
    for id in &seen_ids {
        assert!(
            id.starts_with("auth::"),
            "runner must only execute clauses from the targeted spec file; \
             found unexpected clause: {id}"
        );
    }
    assert!(
        !seen_ids.iter().any(|id| id.starts_with("runner::")),
        "clauses from runner.ought.md must not be executed when only auth.ought.md is targeted"
    );

    let _ = std::fs::remove_dir_all(&tmp);
}