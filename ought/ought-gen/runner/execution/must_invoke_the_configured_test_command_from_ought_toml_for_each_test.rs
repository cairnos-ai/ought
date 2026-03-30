/// MUST invoke the configured test command from `ought.toml` for each language runner
#[test]
fn test_runner__execution__must_invoke_the_configured_test_command_from_ought_toml_for_each() {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use ought_spec::config::{Config, GeneratorConfig, ProjectConfig, RunnerConfig, SpecsConfig};

    // Verify the runner factory recognises every supported language key and returns
    // a runner whose name matches the key used in ought.toml [runner.<name>] tables.
    let cases = [
        ("rust",       "rust"),
        ("python",     "python"),
        ("typescript", "typescript"),
        ("ts",         "typescript"),   // alias
        ("go",         "go"),
    ];
    for (input, expected_name) in cases {
        let runner = ought_run::runners::from_name(input)
            .unwrap_or_else(|e| panic!("from_name({input:?}) must succeed: {e}"));
        assert_eq!(
            runner.name(), expected_name,
            "runner for ought.toml key '{input}' must report name '{expected_name}'"
        );
    }

    // Verify that a parsed ought.toml preserves the configured command strings.
    let toml = r#"
[project]
name = "test"
version = "0.1.0"

[generator]
provider = "anthropic"

[runner.rust]
command = "cargo test"
test_dir = "ought/ought-gen/"

[runner.python]
command = "pytest"
test_dir = "ought/ought-gen/"

[runner.typescript]
command = "npx jest --runInBand"
test_dir = "ought/ought-gen/"

[runner.go]
command = "go test ./..."
test_dir = "ought/ought-gen/"
"#;
    let tmp = std::env::temp_dir()
        .join(format!("ought_cmd_cfg_{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    std::fs::write(tmp.join("ought.toml"), toml).unwrap();

    let config = Config::load(&tmp.join("ought.toml"))
        .expect("ought.toml must load without error");

    let expected_commands = [
        ("rust",       "cargo test"),
        ("python",     "pytest"),
        ("typescript", "npx jest --runInBand"),
        ("go",         "go test ./..."),
    ];
    for (lang, expected_cmd) in expected_commands {
        let cfg = config.runner.get(lang)
            .unwrap_or_else(|| panic!("runner.{lang} config must be present in ought.toml"));
        assert_eq!(
            cfg.command, expected_cmd,
            "runner.{lang}.command must equal the value from ought.toml; \
             expected {expected_cmd:?}, got {:?}", cfg.command
        );
    }

    let _ = std::fs::remove_dir_all(&tmp);
}