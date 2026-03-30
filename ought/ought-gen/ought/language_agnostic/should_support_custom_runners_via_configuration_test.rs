/// SHOULD support custom runners via configuration
///
/// Users must be able to specify arbitrary `[runner.<name>]` sections in
/// `ought.toml` to extend the tool to languages or test frameworks not
/// included in the built-in set. The runner name and command must round-trip
/// through config parsing unchanged.
#[test]
fn test_ought__language_agnostic__should_support_custom_runners_via_configuration() {
    use ought_spec::config::Config;

    let toml = r#"
[project]
name = "custom-runner-test"
version = "0.1.0"

[generator]
provider = "anthropic"

[runner.elixir]
command = "mix test"
test_dir = "test/ought/"

[runner.ruby]
command = "bundle exec rspec"
test_dir = "spec/ought/"

[runner.dotnet]
command = "dotnet test"
test_dir = "tests/ought/"
"#;

    let tmp = std::env::temp_dir()
        .join(format!("ought_custom_runner_cfg_{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    let cfg_path = tmp.join("ought.toml");
    std::fs::write(&cfg_path, toml).unwrap();

    let config = Config::load(&cfg_path)
        .expect("ought.toml with custom [runner.*] sections must parse without error");

    // All three custom sections must survive parsing.
    assert_eq!(
        config.runner.len(), 3,
        "all custom runner sections must be present after parsing; found keys: {:?}",
        config.runner.keys().collect::<Vec<_>>()
    );

    // Each custom runner's command and test_dir must round-trip exactly.
    let elixir = config.runner.get("elixir")
        .expect("runner 'elixir' (non-built-in) must be accepted by config parser");
    assert_eq!(elixir.command, "mix test",
        "custom runner command must survive config parsing unchanged");
    assert_eq!(elixir.test_dir.to_string_lossy(), "test/ought/",
        "custom runner test_dir must survive config parsing unchanged");

    let ruby = config.runner.get("ruby")
        .expect("runner 'ruby' (non-built-in) must be accepted by config parser");
    assert_eq!(ruby.command, "bundle exec rspec",
        "multi-word custom command must round-trip unchanged");

    let dotnet = config.runner.get("dotnet")
        .expect("runner 'dotnet' (non-built-in) must be accepted by config parser");
    assert_eq!(dotnet.command, "dotnet test");

    let _ = std::fs::remove_dir_all(&tmp);
}