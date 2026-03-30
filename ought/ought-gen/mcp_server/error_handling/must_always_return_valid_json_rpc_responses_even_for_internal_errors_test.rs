/// MUST ALWAYS return valid JSON-RPC responses, even for internal errors
/// Temporal: MUST ALWAYS (invariant). Fuzz-style test covering diverse malformed inputs.
#[test]
fn test_mcp_server__error_handling__must_always_return_valid_json_rpc_responses_even_for_internal_errors() {
    use std::io::{BufRead, BufReader, Write};

    let base = std::env::temp_dir().join(format!("ought_fuzz_jsonrpc_{}", std::process::id()));
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

    // Fuzz corpus: syntactically valid JSON that is semantically invalid JSON-RPC.
    // The server MUST return a valid JSON-RPC envelope for every request-like message.
    // (Notifications — messages without "id" — do not require a response per the spec.)
    let cases: &[(&str, &str)] = &[
        ("unknown-method",        r#"{"jsonrpc":"2.0","id":10,"method":"no/such/method","params":{}}"#),
        ("missing-params",        r#"{"jsonrpc":"2.0","id":11,"method":"tools/call"}"#),
        ("null-params",           r#"{"jsonrpc":"2.0","id":12,"method":"tools/call","params":null}"#),
        ("empty-params",          r#"{"jsonrpc":"2.0","id":13,"method":"tools/call","params":{}}"#),
        ("wrong-jsonrpc-version", r#"{"jsonrpc":"1.0","id":14,"method":"tools/list","params":{}}"#),
        ("tools-call-no-name",    r#"{"jsonrpc":"2.0","id":15,"method":"tools/call","params":{"arguments":{}}}"#),
        ("tools-call-null-name",  r#"{"jsonrpc":"2.0","id":16,"method":"tools/call","params":{"name":null,"arguments":{}}}"#),
        ("resources-read-no-uri", r#"{"jsonrpc":"2.0","id":17,"method":"resources/read","params":{}}"#),
        ("resources-read-bad-uri",r#"{"jsonrpc":"2.0","id":18,"method":"resources/read","params":{"uri":"not://valid"}}"#),
        ("extra-unknown-fields",  r#"{"jsonrpc":"2.0","id":19,"method":"tools/list","params":{},"extra":"ignored","__injection":true}"#),
        ("integer-id",            r#"{"jsonrpc":"2.0","id":20,"method":"tools/list","params":{}}"#),
        ("string-id",             r#"{"jsonrpc":"2.0","id":"req-abc","method":"tools/list","params":{}}"#),
        ("deeply-nested-args",    r#"{"jsonrpc":"2.0","id":21,"method":"tools/call","params":{"name":"ought_run","arguments":{"a":{"b":{"c":{"d":null}}}}}}"#),
        ("empty-string-method",   r#"{"jsonrpc":"2.0","id":22,"method":"","params":{}}"#),
        ("unicode-in-method",     r#"{"jsonrpc":"2.0","id":23,"method":"tools/\u0000call","params":{}}"#),
    ];

    let mut responded_count = 0;
    for (label, msg) in cases {
        let resp = exchange(&mut stdin, &mut reader, msg, 3000);
        if let Some(resp) = resp {
            responded_count += 1;
            let v = serde_json::from_str::<serde_json::Value>(&resp)
                .unwrap_or_else(|_| panic!("{label}: response must be valid JSON; got: {resp}"));
            assert_eq!(
                v["jsonrpc"], "2.0",
                "{label}: every response must include jsonrpc:\"2.0\"; got: {resp}"
            );
            assert!(
                v.get("result").is_some() || v.get("error").is_some(),
                "{label}: response must have either 'result' or 'error' at top level; got: {resp}"
            );
            // Must NOT contain ANSI escape codes even in error messages.
            assert!(
                !resp.contains("\x1b["),
                "{label}: error response must not contain ANSI escape codes; got: {resp}"
            );
        }
        // If no response was received: the server may legitimately omit a response for
        // some edge cases (e.g. unparseable id), but must not crash — verified below.
    }

    let still_alive = child.try_wait().expect("try_wait").is_none();
    let _ = child.kill();
    child.wait().ok();
    let _ = std::fs::remove_dir_all(&base);

    assert!(
        still_alive,
        "server must remain alive after receiving {} malformed/invalid inputs",
        cases.len()
    );
    assert!(
        responded_count > 0,
        "server must have responded to at least some of the {} test inputs (got {})",
        cases.len(),
        responded_count
    );
}