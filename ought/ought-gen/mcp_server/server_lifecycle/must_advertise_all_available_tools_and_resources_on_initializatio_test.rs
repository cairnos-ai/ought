/// MUST advertise all available tools and resources on initialization
#[test]
fn test_mcp_server__server_lifecycle__must_advertise_all_available_tools_and_resources_on_initializatio() {
    use std::io::{BufRead, BufReader, Write};

    let base = std::env::temp_dir()
        .join(format!("ought_mcp_adv_{}", std::process::id()));
    std::fs::create_dir_all(base.join("specs")).unwrap();
    std::fs::write(
        base.join("ought.toml"),
        "[project]\nname = \"test\"\nversion = \"0.1.0\"\n\n[specs]\nroots = [\"specs\"]\n\n[generator]\nprovider = \"anthropic\"\n",
    )
    .unwrap();

    let bin = option_env!("CARGO_BIN_EXE_ought")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("ought"));

    let mut child = std::process::Command::new(&bin)
        .args(["mcp", "serve"])
        .current_dir(&base)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("`ought mcp serve` must spawn for MCP handshake");

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);

    /// Send one newline-delimited JSON-RPC message and read back the first non-empty
    /// response line, waiting up to `timeout_ms` milliseconds.
    fn jsonrpc_exchange(
        stdin: &mut std::process::ChildStdin,
        reader: &mut BufReader<std::process::ChildStdout>,
        msg: &str,
        timeout_ms: u64,
    ) -> Option<String> {
        writeln!(stdin, "{msg}").ok()?;
        let deadline =
            std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
        let mut line = String::new();
        loop {
            if std::time::Instant::now() > deadline {
                return None;
            }
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => return None, // EOF
                Ok(_) if !line.trim().is_empty() => return Some(line.trim().to_string()),
                _ => std::thread::sleep(std::time::Duration::from_millis(20)),
            }
        }
    }

    // Step 1: MCP initialize handshake.
    let init_resp = jsonrpc_exchange(
        &mut stdin,
        &mut reader,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"ought-test","version":"0.0.1"}}}"#,
        5000,
    );
    assert!(
        init_resp.is_some(),
        "MCP server must respond to the initialize request within 5 s"
    );
    let init_resp = init_resp.unwrap();
    assert!(
        init_resp.contains("\"jsonrpc\"") && init_resp.contains("\"result\""),
        "initialize response must be a valid JSON-RPC result; got: {init_resp}"
    );

    // Step 2: Send the initialized notification (client signals readiness, no response).
    writeln!(stdin, r#"{{"jsonrpc":"2.0","method":"notifications/initialized"}}"#).ok();

    // Step 3: Request the tools list.
    let tools_resp = jsonrpc_exchange(
        &mut stdin,
        &mut reader,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
        5000,
    );
    assert!(
        tools_resp.is_some(),
        "MCP server must respond to tools/list"
    );
    let tools_resp = tools_resp.unwrap();
    for expected_tool in &[
        "ought_run",
        "ought_generate",
        "ought_check",
        "ought_inspect",
        "ought_status",
        "ought_survey",
        "ought_audit",
        "ought_blame",
        "ought_bisect",
    ] {
        assert!(
            tools_resp.contains(expected_tool),
            "tools/list must advertise the '{expected_tool}' tool; got: {tools_resp}"
        );
    }

    // Step 4: Request the resources list.
    let res_resp = jsonrpc_exchange(
        &mut stdin,
        &mut reader,
        r#"{"jsonrpc":"2.0","id":3,"method":"resources/list"}"#,
        5000,
    );
    assert!(
        res_resp.is_some(),
        "MCP server must respond to resources/list"
    );
    let res_resp = res_resp.unwrap();
    for expected_uri in &[
        "ought://specs",
        "ought://results/latest",
        "ought://coverage",
        "ought://manifest",
    ] {
        assert!(
            res_resp.contains(expected_uri),
            "resources/list must advertise '{expected_uri}'; got: {res_resp}"
        );
    }

    let _ = child.kill();
    child.wait().ok();
    let _ = std::fs::remove_dir_all(&base);
}