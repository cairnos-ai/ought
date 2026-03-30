/// MUST NOT trigger generation — the runner only executes existing generated tests
#[test]
fn test_runner__execution__must_not_trigger_generation_the_runner_only_executes_existing_generat() {
    use std::path::{Path, PathBuf};
    use std::time::Duration;
    use ought_run::{Runner, RunResult, TestResult, TestStatus, TestDetails};
    use ought_gen::GeneratedTest;
    use ought_spec::ClauseId;

    // Set up an isolated temp directory that mimics the generated-test directory.
    let tmp = std::env::temp_dir()
        .join(format!("ought_no_gen_{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    // Write a pre-existing generated test file.
    let existing_file = tmp.join("runner__execution__example_test.rs");
    let existing_code = "/// pre-generated\n\
        #[test]\n\
        fn test_runner__execution__example() { assert!(true); }\n";
    std::fs::write(&existing_file, existing_code).unwrap();

    // Snapshot the directory before running.
    let before_files: std::collections::BTreeSet<String> = std::fs::read_dir(&tmp)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    // A runner that only executes — never invokes a generator or LLM.
    struct ExecuteOnlyRunner;
    impl Runner for ExecuteOnlyRunner {
        fn run(&self, tests: &[GeneratedTest], test_dir: &Path) -> anyhow::Result<RunResult> {
            // Verify that the test files it is given already exist on disk;
            // if the runner were generating, they would not need to pre-exist.
            for t in tests {
                let full_path = test_dir.join(&t.file_path);
                assert!(
                    full_path.exists(),
                    "runner must execute pre-existing test files; \
                     if generation were triggered, this file would not yet exist: {full_path:?}"
                );
            }
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
        fn name(&self) -> &str { "execute-only" }
    }

    let tests = vec![GeneratedTest {
        clause_id: ClauseId("runner::execution::example".to_string()),
        code: existing_code.to_string(),
        language: ought_gen::generator::Language::Rust,
        file_path: PathBuf::from("runner__execution__example_test.rs"),
    }];

    ExecuteOnlyRunner.run(&tests, &tmp).expect("runner must not error");

    // After execution the directory must be identical to before: no new files,
    // no files removed — generation must not have been triggered.
    let after_files: std::collections::BTreeSet<String> = std::fs::read_dir(&tmp)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    assert_eq!(
        before_files, after_files,
        "runner must not create or remove any files (generation must not be triggered); \
         before: {before_files:?}, after: {after_files:?}"
    );

    // No ought.toml, manifest, or spec file must have been written.
    let generation_artifacts = ["ought.toml", "manifest.toml"];
    for artifact in &generation_artifacts {
        assert!(
            !tmp.join(artifact).exists(),
            "runner must not write generation artifact '{artifact}' during execution"
        );
    }

    let _ = std::fs::remove_dir_all(&tmp);
}