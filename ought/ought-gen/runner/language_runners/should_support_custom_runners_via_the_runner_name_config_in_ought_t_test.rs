/// SHOULD support custom runners via the `[runner.<name>]` config in `ought.toml`
///
/// Any arbitrary runner name used as `[runner.<name>]` in `ought.toml` must be
/// preserved in the parsed `Config::runner` map with its `command` and `test_dir`
/// values intact.  The runner map is keyed by the name from the TOML table header,
/// so custom names are fully round-tripped through config parsing.
#[test]
fn test_runner__language_runners__should_support_custom_runners_via_the_runner_name_config_in_ought_t() {
    use ought_spec::config::Config;

    // A config that exercises multiple custom runner names alongside the
    // built-in ones, including names that are not in the `from_name` factory.
    let toml = r#"
[project]
name = "custom-runner-test"
version = "0.1.0"

[generator]
provider = "anthropic"

[runner.rust]
command = "cargo test"
test_dir = "ought/ought-gen/"

[runner.my-ruby-runner]
command = "bundle exec rspec"
test_dir = "spec/ought/"

[runner.dotnet]
command = "dotnet test"
test_dir = "tests/ought/"
"#;

    let tmp = std::env::temp_dir()
        .join(format!("ought_custom_runner_{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    let cfg_path = tmp.join("ought.toml");
    std::fs::write(&cfg_path, toml).unwrap();

    let config = Config::load(&cfg_path)
        .expect("ought.toml with custom [runner.*] sections must load without error");

    // Built-in runner name is preserved.
    let rust_cfg = config.runner.get("rust")
        .expect("runner.rust must be present in parsed config");
    assert_eq!(rust_cfg.command, "cargo test");

    // Fully custom runner name is preserved with its command.
    let ruby_cfg = config.runner.get("my-ruby-runner")
        .expect("runner.my-ruby-runner must be present — custom runner names must be supported");
    assert_eq!(
        ruby_cfg.command, "bundle exec rspec",
        "custom runner command must round-trip through config parsing"
    );
    assert_eq!(
        ruby_cfg.test_dir.to_string_lossy(),
        "spec/ought/",
        "custom runner test_dir must round-trip through config parsing"
    );

    // A second custom name also round-trips correctly.
    let dotnet_cfg = config.runner.get("dotnet")
        .expect("runner.dotnet must be present — arbitrary runner names must be supported");
    assert_eq!(dotnet_cfg.command, "dotnet test");

    // The total number of runners matches what was declared.
    assert_eq!(
        config.runner.len(),
        3,
        "all three [runner.*] sections must be parsed; found {:?}",
        config.runner.keys().collect::<Vec<_>>()
    );

    let _ = std::fs::remove_dir_all(&tmp);
}