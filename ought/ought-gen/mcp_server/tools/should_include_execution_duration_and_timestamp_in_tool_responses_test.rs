/// SHOULD include execution duration and timestamp in tool responses
#[test]
fn test_mcp_server__tools__should_include_execution_duration_and_timestamp_in_tool_responses() {
    use std::io::{BufRead, BufReader, Write};

    let base = std::env::temp_dir().join(format!("ought_tdur_{}", std::process::id()));
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
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("`ought mcp serve` must spawn");

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);

    fn jsonrpc_exchange(
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

    let init = jsonrpc_exchange(&mut stdin, &mut reader,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"ought-test","version":"0.0.1"}}}"#,
        5000);
    assert!(init.is_some(), "server must respond to initialize");
    writeln!(stdin, r#"{{"jsonrpc":"2.0","method":"notifications/initialized"}}"#).ok();

    // Call ought_status — a fast tool — and verify the response includes duration and timestamp metadata
    let resp = jsonrpc_exchange(&mut stdin, &mut reader,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"ought_status","arguments":{}}}"#,
        10000);
    assert!(resp.is_some(), "ought_status must respond");
    let resp = resp.unwrap();
    assert!(resp.contains("\"result\""), "ought_status must return a JSON-RPC result; got: {resp}");

    // Duration field: must appear as duration_ms, duration, or elapsed (numeric milliseconds)
    assert!(
        resp.contains("duration_ms") || resp.contains("\"duration\"") || resp.contains("elapsed_ms") || resp.contains("elapsed"),
        "tool response must include an execution duration field (e.g. duration_ms); got: {resp}"
    );

    // Timestamp field: must appear as timestamp, executed_at, or started_at (ISO-8601 or Unix epoch)
    assert!(
        resp.contains("timestamp") || resp.contains("executed_at") || resp.contains("started_at"),
        "tool response must include a timestamp field (e.g. timestamp or executed_at); got: {resp}"
    );

    // Also verify ought_run includes these fields, since it is a longer-running tool where timing matters most
    let resp_run = jsonrpc_exchange(&mut stdin, &mut reader,
        r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"ought_run","arguments":{}}}"#,
        15000);
    assert!(resp_run.is_some(), "ought_run must respond");
    let resp_run = resp_run.unwrap();
    assert!(resp_run.contains("\"result\""), "ought_run must return a JSON-RPC result; got: {resp_run}");
    assert!(
        resp_run.contains("duration_ms") || resp_run.contains("\"duration\"") || resp_run.contains("elapsed_ms") || resp_run.contains("elapsed"),
        "ought_run response must include an execution duration field; got: {resp_run}"
    );
    assert!(
        resp_run.contains("timestamp") || resp_run.contains("executed_at") || resp_run.contains("started_at"),
        "ought_run response must include a timestamp field; got: {resp_run}"
    );

    let _ = child.kill();
    child.wait().ok();
    let _ = std::fs::remove_dir_all(&base);
}