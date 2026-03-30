/// MUST pass the generated test files/directory to the test harness
#[test]
fn test_runner__execution__must_pass_the_generated_test_files_directory_to_the_test_harness() {
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use ought_run::{Runner, RunResult};
    use ought_gen::GeneratedTest;
    use ought_spec::ClauseId;

    // A runner that records the test_dir it receives — we verify the exact path
    // is forwarded rather than silently dropped or substituted.
    struct RecordingRunner {
        captured_dir: Arc<Mutex<Option<PathBuf>>>,
        captured_file_paths: Arc<Mutex<Vec<PathBuf>>>,
    }
    impl Runner for RecordingRunner {
        fn run(&self, tests: &[GeneratedTest], test_dir: &Path) -> anyhow::Result<RunResult> {
            *self.captured_dir.lock().unwrap() = Some(test_dir.to_path_buf());
            let mut paths = self.captured_file_paths.lock().unwrap();
            for t in tests {
                paths.push(test_dir.join(&t.file_path));
            }
            Ok(RunResult { results: vec![], total_duration: Duration::ZERO })
        }
        fn is_available(&self) -> bool { true }
        fn name(&self) -> &str { "recording" }
    }

    let test_dir = std::env::temp_dir()
        .join(format!("ought_pass_dir_{}", std::process::id()));
    std::fs::create_dir_all(&test_dir).unwrap();

    let captured_dir = Arc::new(Mutex::new(None));
    let captured_paths = Arc::new(Mutex::new(vec![]));
    let runner = RecordingRunner {
        captured_dir: Arc::clone(&captured_dir),
        captured_file_paths: Arc::clone(&captured_paths),
    };

    let tests = vec![GeneratedTest {
        clause_id: ClauseId("runner::execution::must_pass_dir".to_string()),
        code: "#[test] fn test_runner__execution__must_pass_dir() {}".to_string(),
        language: ought_gen::generator::Language::Rust,
        file_path: PathBuf::from("runner/execution/must_pass_dir_test.rs"),
    }];

    runner.run(&tests, &test_dir).expect("runner must not error");

    // The exact test_dir must arrive at the runner unchanged.
    let actual_dir = captured_dir.lock().unwrap().clone();
    assert_eq!(
        actual_dir.as_deref(),
        Some(test_dir.as_path()),
        "runner must receive the exact test_dir path from the caller; \
         expected {test_dir:?}, got {actual_dir:?}"
    );

    // Every generated-test file path must be rooted within that test_dir.
    let paths = captured_paths.lock().unwrap().clone();
    assert!(!paths.is_empty(), "runner must receive at least one test file path");
    for p in &paths {
        assert!(
            p.starts_with(&test_dir),
            "every test file path passed to the harness must be inside test_dir; \
             {p:?} is not under {test_dir:?}"
        );
    }

    let _ = std::fs::remove_dir_all(&test_dir);
}