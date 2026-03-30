/// MUST provide an MCP server so AI assistants and IDE extensions can interact with ought programmatically
#[test]
fn test_ought__integration__must_provide_an_mcp_server_so_ai_assistants_and_ide_extensions_ca() {
    let bin = option_env!("CARGO_BIN_EXE_ought")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("ought"));

    let base = std::env::temp_dir()
        .join(format!("ought_integration_mcp_{}", std::process::id()));
    std::fs::create_dir_all(base.join("specs")).unwrap();
    std::fs::write(
        base.join("ought.toml"),
        "[project]\nname = \"test\"\nversion = \"0.1.0\"\n\n[specs]\nroots = [\"specs\"]\n\n[generator]\nprovider = \"anthropic\"\n",
    )
    .unwrap();

    // `ought mcp serve` must be recognised so AI assistants and IDEs can connect.
    let mut child = std::process::Command::new(&bin)
        .args(["mcp", "serve"])
        .current_dir(&base)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("`ought mcp serve` must launch without error");

    std::thread::sleep(std::time::Duration::from_millis(300));
    let _ = child.kill();
    let output = child.wait_with_output().expect("wait for ought mcp serve");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_ne!(
        output.status.code(),
        Some(2),
        "`ought mcp serve` must be a recognised subcommand (clap exit 2 = unrecognised); stderr: {stderr}"
    );
    assert!(
        !stderr.contains("unrecognized subcommand") && !stderr.contains("unexpected argument"),
        "`ought mcp serve` must be a recognised subcommand; stderr: {stderr}"
    );

    // `ought mcp install` must also be recognised so IDE extensions can auto-register the server.
    let install_out = std::process::Command::new(&bin)
        .args(["mcp", "install", "--help"])
        .current_dir(&base)
        .output();

    if let Ok(out) = install_out {
        let install_stderr = String::from_utf8_lossy(&out.stderr);
        assert_ne!(
            out.status.code(),
            Some(2),
            "`ought mcp install` must be a recognised subcommand for IDE auto-registration; stderr: {install_stderr}"
        );
        assert!(
            !install_stderr.contains("unrecognized subcommand"),
            "`ought mcp install` must be a recognised subcommand; stderr: {install_stderr}"
        );
    }

    let _ = std::fs::remove_dir_all(&base);
}