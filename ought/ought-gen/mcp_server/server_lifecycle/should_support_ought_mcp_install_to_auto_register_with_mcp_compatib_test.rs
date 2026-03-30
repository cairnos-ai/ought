/// SHOULD support `ought mcp install` to auto-register with MCP-compatible coding agents
/// (Claude Code, Codex, OpenCode)
#[test]
fn test_mcp_server__server_lifecycle__should_support_ought_mcp_install_to_auto_register_with_mcp_compatib() {
    let bin = option_env!("CARGO_BIN_EXE_ought")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("ought"));

    // `ought mcp install --help` must be a recognised subcommand.
    let help_out = std::process::Command::new(&bin)
        .args(["mcp", "install", "--help"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .expect("`ought mcp install --help` must run");

    let help = format!(
        "{}{}",
        String::from_utf8_lossy(&help_out.stdout),
        String::from_utf8_lossy(&help_out.stderr)
    );

    assert_ne!(
        help_out.status.code(),
        Some(2),
        "`ought mcp install` must be a recognised subcommand (clap exit 2 means parse error); \
         got:\n{help}"
    );
    assert!(
        !help.contains("unrecognized subcommand"),
        "`ought mcp install` must be a recognised subcommand; got:\n{help}"
    );

    // The help text should reference at least one of the supported coding agents.
    let help_lower = help.to_lowercase();
    let mentions_agent = help_lower.contains("claude")
        || help_lower.contains("codex")
        || help_lower.contains("opencode")
        || help_lower.contains("register")
        || help_lower.contains("agent");
    assert!(
        mentions_agent,
        "`ought mcp install --help` must describe registration with MCP-compatible coding \
         agents (Claude Code, Codex, OpenCode); got:\n{help}"
    );

    // Running `ought mcp install` in an isolated home directory must not crash.
    // We point HOME at a fresh temp dir so the command cannot accidentally modify
    // the real user's agent configurations.
    let fake_home = std::env::temp_dir()
        .join(format!("ought_mcp_install_home_{}", std::process::id()));
    std::fs::create_dir_all(&fake_home).unwrap();

    let install_out = std::process::Command::new(&bin)
        .args(["mcp", "install"])
        .env("HOME", &fake_home)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .expect("`ought mcp install` must run without panicking");

    // Must not exit with code 2 (argument parse failure) — that would mean the
    // subcommand is not registered.
    assert_ne!(
        install_out.status.code(),
        Some(2),
        "`ought mcp install` must not produce a usage/parse error; stderr: {}",
        String::from_utf8_lossy(&install_out.stderr)
    );

    let _ = std::fs::remove_dir_all(&fake_home);
}