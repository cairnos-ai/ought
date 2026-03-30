/// MUST NOT crash the server on a single tool invocation failure
#[test]
fn test_mcp_server__error_handling__must_not_crash_the_server_on_a_single_tool_invocation_failure() {
    use std::io::{BufRead, BufReader, Write};

    let base = std::env::temp_dir().join(format!("ought_no_crash_{}", std::process::id()));
    std::fs::create_dir_all(base.join("specs")).unwrap();
    std::fs::write(
        base.join("ought.toml"),
        "[project]\nname = \"test\"\nversion = \"0.1.0\"\n\n[specs]\nroots = [\"specs\"]\n\n[generator]\nprovider = \"anthropic\"\n",
    ).unwrap();
    std::fs::write(
        base.join("specs").join("auth.ought.md"),
        "# Auth\n\n## Login\n\nMUST accept valid credentials\n",
    ).unwrap();

    let bin = option_env!("CARGO_BIN_EXE_ought")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("ought"));
    let mut child = std::process::Command::new(&bin)
        .args(["mcp", "serve"])
        .current_dir(&base)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("`ought mcp serve` must spawn");

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);

    fn exchange(
        stdin: &mut std::process::ChildStdin,
        reader: &mut BufReader<std::process::ChildStdout>,
        msg: &str,
        timeout_ms: u64,
    ) -> Option<String> {
        writeln!(stdin, "{msg}").ok()?;
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
        let mut line = String::new();
        loop {
            if std::time::Instant::now() > deadline { return None; }
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => return None,
                Ok(_) if !line.trim().is_empty() => return Some(line.trim().to_string()),
                _ => std::thread::sleep(std::time::Duration::from_millis(20)),
            }
        }
    }

    exchange(&mut stdin, &mut reader,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"ought-test","version":"0.0.1"}}}"#,
        5000).expect("server must respond to initialize");
    writeln!(stdin, r#"{{"jsonrpc":"2.0","method":"notifications/initialized"}}"#).ok();

    // Trigger a tool failure: call ought_inspect without required clause_id.
    let fail_resp = exchange(&mut stdin, &mut reader,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"ought_inspect","arguments":{}}}"#,
        5000);
    assert!(fail_resp.is_some(), "server must still respond after a failing tool call");

    // Server must not have crashed — it must still handle a subsequent request.
    let ok_resp = exchange(&mut stdin, &mut reader,
        r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"ought_status","arguments":{}}}"#,
        10000);

    // Verify the server process is still alive before killing.
    let still_alive = child.try_wait().expect("try_wait").is_none();

    let _ = child.kill();
    child.wait().ok();
    let _ = std::fs::remove_dir_all(&base);

    assert!(still_alive, "server process must still be running after a single tool failure");
    assert!(
        ok_resp.is_some(),
        "server must respond to a request that follows a failed tool invocation"
    );
    let ok_resp = ok_resp.unwrap();
    assert!(
        ok_resp.contains("\"result\"") || ok_resp.contains("\"error\""),
        "post-failure response must be a valid JSON-RPC envelope; got: {ok_resp}"
    );
}