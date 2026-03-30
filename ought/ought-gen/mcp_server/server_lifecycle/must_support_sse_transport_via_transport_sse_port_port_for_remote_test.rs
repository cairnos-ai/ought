/// MUST support SSE transport via `--transport sse --port <port>` for remote clients
#[test]
fn test_mcp_server__server_lifecycle__must_support_sse_transport_via_transport_sse_port_port_for_remote() {
    let base = std::env::temp_dir()
        .join(format!("ought_mcp_sse_{}", std::process::id()));
    std::fs::create_dir_all(base.join("specs")).unwrap();
    std::fs::write(
        base.join("ought.toml"),
        "[project]\nname = \"test\"\nversion = \"0.1.0\"\n\n[specs]\nroots = [\"specs\"]\n\n[generator]\nprovider = \"anthropic\"\n",
    )
    .unwrap();

    let bin = option_env!("CARGO_BIN_EXE_ought")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("ought"));

    // Use a high ephemeral port unlikely to be in use.
    let mut child = std::process::Command::new(&bin)
        .args(["mcp", "serve", "--transport", "sse", "--port", "19877"])
        .current_dir(&base)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("`ought mcp serve --transport sse --port 19877` must launch");

    std::thread::sleep(std::time::Duration::from_millis(300));
    let _ = child.kill();
    let output = child.wait_with_output().expect("wait");
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Clap exits 2 for unrecognised arguments. Any other exit code means the
    // flags were accepted.
    assert_ne!(
        output.status.code(),
        Some(2),
        "`--transport sse --port <port>` must be valid arguments for `ought mcp serve`; \
         stderr: {stderr}"
    );
    assert!(
        !stderr.contains("unrecognized argument") && !stderr.contains("unexpected argument"),
        "`--transport sse --port <port>` must be accepted without argument errors; \
         stderr: {stderr}"
    );

    // When SSE transport is active, the server must listen on the given port.
    // We give it 500 ms to bind and then check the port is open.
    let mut child2 = std::process::Command::new(&bin)
        .args(["mcp", "serve", "--transport", "sse", "--port", "19877"])
        .current_dir(&base)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn SSE server");

    std::thread::sleep(std::time::Duration::from_millis(500));

    let port_open = std::net::TcpStream::connect("127.0.0.1:19877").is_ok();

    let _ = child2.kill();
    child2.wait().ok();

    assert!(
        port_open,
        "`ought mcp serve --transport sse --port 19877` must bind to the specified port"
    );

    let _ = std::fs::remove_dir_all(&base);
}