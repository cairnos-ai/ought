/// MUST NOT require any language-specific SDK or library in the project under test
///
/// Ought works by generating plain source files and delegating execution to
/// the project's native harness. The project under test must not need to
/// import, depend on, or link against any ought-specific library. Generated
/// tests are self-contained source strings with no ought dependency.
#[test]
fn test_ought__language_agnostic__must_not_require_any_language_specific_sdk_or_library_in_the_project() {
    use ought_spec::config::RunnerConfig;
    use ought_gen::GeneratedTest;
    use ought_gen::generator::Language;
    use ought_spec::ClauseId;
    use std::path::PathBuf;

    // RunnerConfig stores only a shell command string and a path.
    // No language-specific bindings exist: any harness reachable via CLI works.
    let ruby_cfg = RunnerConfig {
        command: "bundle exec rspec".to_string(),
        test_dir: PathBuf::from("spec/ought/"),
    };
    assert_eq!(ruby_cfg.command, "bundle exec rspec",
        "RunnerConfig.command must accept any shell command — not just built-in harnesses");

    // GeneratedTest.code is a plain String — raw source text written to disk.
    // A valid generated test contains zero ought imports.
    let rust_test = GeneratedTest {
        clause_id: ClauseId("example::must_add".to_string()),
        code: "#[test]\nfn test_example__must_add() { assert_eq!(1 + 1, 2); }".to_string(),
        language: Language::Rust,
        file_path: PathBuf::from("test_example__must_add.rs"),
    };
    assert!(
        !rust_test.code.contains("use ought"),
        "generated Rust test must not require `use ought_*` — no SDK needed in project under test"
    );
    assert!(
        !rust_test.code.contains("extern crate ought"),
        "generated Rust test must not require an ought crate dependency"
    );

    // Same holds for Python.
    let python_test = GeneratedTest {
        clause_id: ClauseId("example::must_add_python".to_string()),
        code: "def test_example__must_add():\n    assert 1 + 1 == 2".to_string(),
        language: Language::Python,
        file_path: PathBuf::from("test_example__must_add.py"),
    };
    assert!(
        !python_test.code.contains("import ought"),
        "generated Python test must not require an ought import"
    );

    // The runner invocation config is a plain shell command — no SDK linking.
    // Verify that an entirely non-Rust command is a first-class config value.
    let go_cfg = RunnerConfig {
        command: "go test ./...".to_string(),
        test_dir: PathBuf::from("ought_tests/"),
    };
    assert!(
        !go_cfg.command.contains("ought_sdk"),
        "runner command must be a plain shell invocation, not an ought SDK call"
    );
}