/// MUST return structured JSON responses from all tools (not terminal-formatted text)
#[test]
fn test_mcp_server__tools__must_return_structured_json_responses_from_all_tools_not_terminal() {
    use std::io::{BufRead, BufReader, Write};

    let base = std::env::temp_dir().join(format!("ought_tjsn_{}", std::process::id()));
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

    // Call a representative set of tools and verify each returns structured JSON with no ANSI codes
    let tool_calls: &[(&str, &str)] = &[
        ("ought_status",   r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"ought_status","arguments":{}}}"#),
        ("ought_check",    r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"ought_check","arguments":{}}}"#),
        ("ought_run",      r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"ought_run","arguments":{}}}"#),
    ];

    for (tool_name, call_msg) in tool_calls {
        let resp = jsonrpc_exchange(&mut stdin, &mut reader, call_msg, 15000);
        assert!(resp.is_some(), "{tool_name} must respond within the timeout");
        let resp = resp.unwrap();

        // Must be a valid JSON-RPC envelope (no raw terminal text)
        assert!(
            resp.contains("\"jsonrpc\"") && resp.contains("\"result\""),
            "{tool_name} response must be a JSON-RPC result envelope; got: {resp}"
        );
        // Must not contain ANSI terminal escape sequences (e.g. color codes, cursor movement)
        assert!(
            !resp.contains("\x1b["),
            "{tool_name} response must not contain ANSI terminal escape codes; got: {resp}"
        );
        // The MCP content envelope must be present — tool output lives in content[].text
        assert!(
            resp.contains("\"content\"") && resp.contains("\"text\""),
            "{tool_name} response must use MCP content envelope with a text field; got: {resp}"
        );
        // The text value must look like JSON (opens with a brace or bracket when unescaped)
        assert!(
            resp.contains("\"text\":\"{") || resp.contains("\"text\":[") || resp.contains("text\":\"{}") || resp.contains("text\":\"{\\"),
            "{tool_name} content text must be structured JSON, not prose or terminal-formatted text; got: {resp}"
        );
    }

    let _ = child.kill();
    child.wait().ok();
    let _ = std::fs::remove_dir_all(&base);
}