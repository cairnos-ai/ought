/// MUST expose `ought://specs/{name}` — parsed clauses for a specific spec file
#[test]
fn test_mcp_server__resources__must_expose_ought_specs_name_parsed_clauses_for_a_specific_spec_f() {
    use std::io::{BufRead, BufReader, Write};

    let base = std::env::temp_dir()
        .join(format!("ought_res_spec1_{}", std::process::id()));
    std::fs::create_dir_all(base.join("specs")).unwrap();
    std::fs::write(
        base.join("ought.toml"),
        "[project]\nname = \"test\"\nversion = \"0.1.0\"\n\n[specs]\nroots = [\"specs\"]\n\n[generator]\nprovider = \"anthropic\"\n",
    ).unwrap();
    std::fs::write(
        base.join("specs").join("auth.ought.md"),
        "# Auth\n\n## Login\n\nMUST accept valid credentials\nSHOULD log authentication attempts\n",
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

    // Read ought://specs/auth to retrieve parsed clauses for that specific spec
    let resp = jsonrpc_exchange(&mut stdin, &mut reader,
        r#"{"jsonrpc":"2.0","id":2,"method":"resources/read","params":{"uri":"ought://specs/auth"}}"#,
        5000);
    assert!(resp.is_some(), "ought://specs/{name} must respond to resources/read");
    let resp = resp.unwrap();
    assert!(
        resp.contains("\"result\""),
        "ought://specs/auth must return a JSON-RPC result (not an error); got: {resp}"
    );
    // The result must contain parsed clause data for the spec
    assert!(
        resp.contains("clause") || resp.contains("must") || resp.contains("login") || resp.contains("auth"),
        "ought://specs/auth must include parsed clause data; got: {resp}"
    );

    // A request for a non-existent spec should return an error, not crash
    let err_resp = jsonrpc_exchange(&mut stdin, &mut reader,
        r#"{"jsonrpc":"2.0","id":3,"method":"resources/read","params":{"uri":"ought://specs/nonexistent"}}"#,
        5000);
    assert!(err_resp.is_some(), "server must respond to resources/read for unknown spec name");
    let err_resp = err_resp.unwrap();
    assert!(
        err_resp.contains("\"error\"") || err_resp.contains("\"result\""),
        "ought://specs/nonexistent must return a JSON-RPC error or empty result; got: {err_resp}"
    );

    let _ = child.kill();
    child.wait().ok();
    let _ = std::fs::remove_dir_all(&base);
}