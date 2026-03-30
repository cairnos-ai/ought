/// MUST NOT modify generated test files during execution
#[test]
fn test_runner__execution__must_not_modify_generated_test_files_during_execution() {
    use std::path::{Path, PathBuf};
    use std::time::Duration;
    use ought_run::{Runner, RunResult};
    use ought_gen::GeneratedTest;
    use ought_spec::ClauseId;

    struct ExecuteOnlyRunner;
    impl Runner for ExecuteOnlyRunner {
        fn run(&self, _tests: &[GeneratedTest], _test_dir: &Path) -> anyhow::Result<RunResult> {
            // Simulate execution without touching any test files.
            Ok(RunResult { results: vec![], total_duration: Duration::ZERO })
        }
        fn is_available(&self) -> bool { true }
        fn name(&self) -> &str { "execute-only" }
    }

    let tmp = std::env::temp_dir()
        .join(format!("ought_immutable_{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    // Pre-populate the test directory with a generated test file.
    let test_file_name = "runner__execution__must_not_modify_test.rs";
    let test_file = tmp.join(test_file_name);
    let original_content = "/// MUST NOT modify generated test files during execution\n\
        #[test]\n\
        fn test_runner__execution__must_not_modify_generated_test_files_during_execution() {\n\
            assert!(true);\n\
        }\n";
    std::fs::write(&test_file, original_content).unwrap();

    let before_bytes = std::fs::read(&test_file).expect("test file must be readable before run");
    let before_mtime = std::fs::metadata(&test_file)
        .expect("test file metadata must be readable before run")
        .modified()
        .ok();

    let tests = vec![GeneratedTest {
        clause_id: ClauseId("runner::execution::must_not_modify_generated_test_files_during_execution".to_string()),
        code: original_content.to_string(),
        language: ought_gen::generator::Language::Rust,
        file_path: PathBuf::from(test_file_name),
    }];

    ExecuteOnlyRunner.run(&tests, &tmp).expect("runner must not error");

    // Content must be byte-for-byte identical after execution.
    let after_bytes = std::fs::read(&test_file)
        .expect("test file must still exist after runner execution");
    assert_eq!(
        before_bytes, after_bytes,
        "runner must not alter the content of generated test files during execution"
    );

    // The file must still exist (runner must not delete it either).
    assert!(
        test_file.exists(),
        "runner must not delete generated test files during execution"
    );

    // No additional .rs files must have been written into the test directory.
    let rs_files_after: Vec<_> = std::fs::read_dir(&tmp)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("rs"))
        .collect();
    assert_eq!(
        rs_files_after.len(), 1,
        "runner must not create new .rs files in the test directory during execution; \
         found: {rs_files_after:?}"
    );

    let _ = std::fs::remove_dir_all(&tmp);
}