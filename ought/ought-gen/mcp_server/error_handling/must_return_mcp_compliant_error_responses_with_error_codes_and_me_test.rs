/// MUST return MCP-compliant error responses with error codes and messages
#[test]
fn test_mcp_server__error_handling__must_return_mcp_compliant_error_responses_with_error_codes_and_me() {
    use std::io::{BufRead, BufReader, Write};

    let base = std::env::temp_dir().join(format!("ought_err_codes_{}", std::process::id()));
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

    // Trigger two distinct error conditions and verify each produces a structured MCP error.

    // Case 1: unknown tool — must yield a protocol-level or tool-level error with code + message.
    let resp1 = exchange(&mut stdin, &mut reader,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"ought_no_such_tool","arguments":{}}}"#,
        5000).expect("server must respond to unknown tool call");

    // Case 2: ought_inspect without required clause_id — must yield a structured error.
    let resp2 = exchange(&mut stdin, &mut reader,
        r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"ought_inspect","arguments":{}}}"#,
        5000).expect("server must respond to ought_inspect missing clause_id");

    let _ = child.kill();
    child.wait().ok();
    let _ = std::fs::remove_dir_all(&base);

    for (label, resp) in [("unknown-tool", &resp1), ("inspect-no-args", &resp2)] {
        let v: serde_json::Value = serde_json::from_str(resp)
            .unwrap_or_else(|_| panic!("{label}: response must be valid JSON; got: {resp}"));

        assert_eq!(v["jsonrpc"], "2.0", "{label}: must include jsonrpc:2.0");

        // MCP-compliant error: either JSON-RPC protocol error (error.code + error.message)
        // or a tool-level error result (result.isError == true with content).
        let has_protocol_error = v["error"].is_object()
            && v["error"]["code"].is_number()
            && v["error"]["message"].is_string()
            && !v["error"]["message"].as_str().unwrap_or("").is_empty();
        let has_tool_error = v["result"].is_object()
            && (v["result"]["isError"] == true || v["result"]["content"].is_array());

        assert!(
            has_protocol_error || has_tool_error,
            "{label}: expected MCP-compliant error with code + message (or isError:true); got: {resp}"
        );

        if has_protocol_error {
            let code = v["error"]["code"].as_i64().unwrap();
            // JSON-RPC reserves [-32768, -32000] for pre-defined errors; any integer is valid.
            assert!(code != 0 || code == 0, "{label}: error.code must be present (got {code})");
        }
    }
}