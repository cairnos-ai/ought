/// MUST ship with a Claude provider that execs the `claude` CLI
#[test]
fn test_generator__provider_abstraction__must_ship_with_a_claude_provider_that_execs_the_claude_cli() {
    use ought_gen::providers::from_config;
    use ought_gen::providers::claude::ClaudeGenerator;
    use ought_gen::generator::Generator;

    // Both canonical aliases must resolve to a ClaudeGenerator without error.
    let via_anthropic = from_config("anthropic", None);
    assert!(
        via_anthropic.is_ok(),
        "from_config(\"anthropic\", _) must return Ok; got: {:?}",
        via_anthropic.err()
    );

    let via_claude = from_config("claude", None);
    assert!(
        via_claude.is_ok(),
        "from_config(\"claude\", _) must return Ok; got: {:?}",
        via_claude.err()
    );

    // The ClaudeGenerator can be constructed directly and satisfies Generator.
    let _g: Box<dyn Generator> = Box::new(ClaudeGenerator::new(None));
    let _with_model: Box<dyn Generator> = Box::new(ClaudeGenerator::new(Some("claude-sonnet-4-6".to_string())));

    // When the `claude` binary is absent the error message must name the tool,
    // not produce a cryptic OS error. Simulate by using exec_cli directly.
    use ought_gen::providers::exec_cli;
    let err = exec_cli("__ought_absent_claude_binary__", &["-p"], "prompt")
        .unwrap_err()
        .to_string();
    assert!(
        err.to_lowercase().contains("not found"),
        "missing CLI error must say 'not found'; got: {err}"
    );
    assert!(
        err.contains("__ought_absent_claude_binary__"),
        "missing CLI error must name the binary; got: {err}"
    );
}