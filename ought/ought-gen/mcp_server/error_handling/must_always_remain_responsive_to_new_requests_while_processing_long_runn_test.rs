/// MUST ALWAYS remain responsive to new requests while processing long-running tools (survey, audit, bisect)
/// Temporal: MUST ALWAYS (invariant). Tests that a quick request is served concurrently with a slow one.
#[test]
fn test_mcp_server__error_handling__must_always_remain_responsive_to_new_requests_while_processing_long_runn() {
    use std::io::{BufRead, BufReader, Write};

    let base = std::env::temp_dir().join(format!("ought_concurrent_{}", std::process::id()));
    std::fs::create_dir_all(base.join("specs")).unwrap();
    std::fs::create_dir_all(base.join("src")).unwrap();
    std::fs::write(
        base.join("ought.toml"),
        "[project]\nname = \"test\"\nversion = \"0.1.0\"\n\n[specs]\nroots = [\"specs\"]\n\n[generator]\nprovider = \"anthropic\"\n",
    ).unwrap();
    std::fs::write(
        base.join("specs").join("auth.ought.md"),
        "# Auth\n\n## Login\n\nMUST accept valid credentials\n",
    ).unwrap();
    std::fs::write(
        base.join("src").join("lib.rs"),
        "pub fn authenticate(user: &str, pass: &str) -> bool { !user.is_empty() && !pass.is_empty() }\n",
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

    // Reads all available lines until the deadline, returning any whose parsed id matches target_id.
    fn find_response_for_id(
        reader: &mut BufReader<std::process::ChildStdout>,
        target_id: u64,
        timeout_ms: u64,
    ) -> Option<String> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
        loop {
            if std::time::Instant::now() > deadline { return None; }
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => return None,
                Ok(_) if !line.trim().is_empty() => {
                    let trimmed = line.trim().to_string();
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&trimmed) {
                        if v.get("id").and_then(|id| id.as_u64()) == Some(target_id) {
                            return Some(trimmed);
                        }
                        // Otherwise: skip — it was a notification or a different response.
                    }
                }
                _ => std::thread::sleep(std::time::Duration::from_millis(20)),
            }
        }
    }

    // Handshake.
    writeln!(stdin,
        r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"protocolVersion":"2024-11-05","capabilities":{{}},"clientInfo":{{"name":"ought-test","version":"0.0.1"}}}}}}"#
    ).ok();
    find_response_for_id(&mut reader, 1, 5000).expect("server must respond to initialize");
    writeln!(stdin, r#"{{"jsonrpc":"2.0","method":"notifications/initialized"}}"#).ok();

    // Pipeline both requests without waiting: first a long-running tool, then a cheap one.
    // The server MUST handle ought_status (id=3) concurrently with ought_survey (id=2).
    writeln!(stdin,
        r#"{{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{{"name":"ought_survey","arguments":{{}}}}}}"#
    ).ok();
    writeln!(stdin,
        r#"{{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{{"name":"ought_status","arguments":{{}}}}}}"#
    ).ok();
    stdin.flush().ok();

    // The quick ought_status request (id=3) must receive a response within 10 seconds,
    // regardless of how long ought_survey (id=2) takes.
    let status_resp = find_response_for_id(&mut reader, 3, 10000);

    let _ = child.kill();
    child.wait().ok();
    let _ = std::fs::remove_dir_all(&base);

    assert!(
        status_resp.is_some(),
        "ought_status (id=3) must respond within 10 s even while ought_survey (id=2) is running; \
         the server must process requests concurrently, not sequentially"
    );
    let status_resp = status_resp.unwrap();
    assert!(
        status_resp.contains("\"result\"") || status_resp.contains("\"error\""),
        "ought_status response must be a valid JSON-RPC envelope; got: {status_resp}"
    );
}