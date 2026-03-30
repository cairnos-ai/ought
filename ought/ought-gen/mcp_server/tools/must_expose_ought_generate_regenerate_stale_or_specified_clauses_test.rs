/// MUST expose `ought_generate` — regenerate stale or specified clauses
#[test]
fn test_mcp_server__tools__must_expose_ought_generate_regenerate_stale_or_specified_clauses() {
    use std::io::{BufRead, BufReader, Write};

    let base = std::env::temp_dir().join(format!("ought_tgen_{}", std::process::id()));
    std::fs::create_dir_all(base.join("specs")).unwrap();
    std::fs::write(
        base.join("ought.toml"),
        "[project]\nname = \"test\"\nversion = \"0.1.0\"\n\n[specs]\nroots = [\"specs\"]\n\n[generator]\nprovider = \"anthropic\"\n",
    ).unwrap();
    std::fs::write(
        base.join("specs").join("auth.ought.md"),
        "# Auth\n\n## Login\n\nMUST accept valid credentials\nMUST reject invalid credentials with 401\n",
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

    // ought_generate with no arguments must regenerate all stale clauses
    let resp_stale = jsonrpc_exchange(&mut stdin, &mut reader,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"ought_generate","arguments":{}}}"#,
        30000);
    assert!(resp_stale.is_some(), "ought_generate must respond when called without arguments (regenerate stale)");
    let resp_stale = resp_stale.unwrap();
    assert!(
        resp_stale.contains("\"result\""),
        "ought_generate with no args must return a JSON-RPC result; got: {resp_stale}"
    );

    // ought_generate with specific clause_ids must regenerate only those clauses
    let resp_specific = jsonrpc_exchange(&mut stdin, &mut reader,
        r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"ought_generate","arguments":{"clause_ids":["auth::login::must_accept_valid_credentials"]}}}"#,
        30000);
    assert!(resp_specific.is_some(), "ought_generate must respond when called with a specific clause_ids list");
    let resp_specific = resp_specific.unwrap();
    assert!(
        resp_specific.contains("\"result\""),
        "ought_generate with clause_ids must return a JSON-RPC result; got: {resp_specific}"
    );

    let _ = child.kill();
    child.wait().ok();
    let _ = std::fs::remove_dir_all(&base);
}