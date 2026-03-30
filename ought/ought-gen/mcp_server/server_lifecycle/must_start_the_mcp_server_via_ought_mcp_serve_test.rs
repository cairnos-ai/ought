/// MUST start the MCP server via `ought mcp serve`
#[test]
fn test_mcp_server__server_lifecycle__must_start_the_mcp_server_via_ought_mcp_serve() {
    let base = std::env::temp_dir()
        .join(format!("ought_mcp_start_{}", std::process::id()));
    std::fs::create_dir_all(base.join("specs")).unwrap();
    std::fs::write(
        base.join("ought.toml"),
        "[project]\nname = \"test\"\nversion = \"0.1.0\"\n\n[specs]\nroots = [\"specs\"]\n\n[generator]\nprovider = \"anthropic\"\n",
    )
    .unwrap();

    let bin = option_env!("CARGO_BIN_EXE_ought")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("ought"));

    // Spawn the server and close stdin immediately to simulate a disconnecting client.
    let mut child = std::process::Command::new(&bin)
        .args(["mcp", "serve"])
        .current_dir(&base)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("`ought mcp serve` must launch without error");

    // Give the process time to either start successfully or fail during arg parsing.
    std::thread::sleep(std::time::Duration::from_millis(300));

    // Terminate and collect output.
    let _ = child.kill();
    let output = child.wait_with_output().expect("wait for ought mcp serve");
    let stderr = String::from_utf8_lossy(&output.stderr);

    // clap exits 2 on unrecognised subcommands; any other exit code means the
    // command was accepted by the argument parser.
    assert_ne!(
        output.status.code(),
        Some(2),
        "`ought mcp serve` must be a recognised subcommand; stderr: {stderr}"
    );
    assert!(
        !stderr.contains("unrecognized subcommand") && !stderr.contains("error: unexpected argument"),
        "`ought mcp serve` must be a recognised subcommand; stderr: {stderr}"
    );

    let _ = std::fs::remove_dir_all(&base);
}