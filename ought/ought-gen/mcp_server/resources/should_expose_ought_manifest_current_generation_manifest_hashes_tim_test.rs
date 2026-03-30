/// SHOULD expose `ought://manifest` — current generation manifest (hashes, timestamps, staleness)
#[test]
fn test_mcp_server__resources__should_expose_ought_manifest_current_generation_manifest_hashes_tim() {
    use std::io::{BufRead, BufReader, Write};

    let base = std::env::temp_dir()
        .join(format!("ought_res_mfst_{}", std::process::id()));
    std::fs::create_dir_all(base.join("specs")).unwrap();
    std::fs::write(
        base.join("ought.toml"),
        "[project]\nname = \"test\"\nversion = \"0.1.0\"\n\n[specs]\nroots = [\"specs\"]\n\n[generator]\nprovider = \"anthropic\"\n",
    ).unwrap();
    std::fs::write(
        base.join("specs").join("auth.ought.md"),
        "# Auth\n\n## Login\n\nMUST accept valid credentials\n",
    ).unwrap();
    // Write a minimal manifest so the resource has data to return
    std::fs::create_dir_all(base.join("ought-gen")).unwrap();
    std::fs::write(
        base.join("ought-gen").join("manifest.toml"),
        "[\"auth::login::must_accept_valid_credentials\"]\nclause_hash = \"abc123\"\nsource_hash = \"\"\ngenerated_at = \"2026-01-01T00:00:00Z\"\nmodel = \"claude-sonnet-4-6\"\n",
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

    // Verify ought://manifest appears in the resources list
    let list_resp = jsonrpc_exchange(&mut stdin, &mut reader,
        r#"{"jsonrpc":"2.0","id":2,"method":"resources/list"}"#,
        5000);
    assert!(list_resp.is_some(), "server must respond to resources/list");
    let list_resp = list_resp.unwrap();
    assert!(
        list_resp.contains("ought://manifest"),
        "resources/list must advertise 'ought://manifest'; got: {list_resp}"
    );

    // Read the manifest resource and verify it contains hashes, timestamps, and staleness info
    let resp = jsonrpc_exchange(&mut stdin, &mut reader,
        r#"{"jsonrpc":"2.0","id":3,"method":"resources/read","params":{"uri":"ought://manifest"}}"#,
        5000);
    assert!(resp.is_some(), "ought://manifest must respond to resources/read");
    let resp = resp.unwrap();
    assert!(
        resp.contains("\"result\""),
        "ought://manifest must return a JSON-RPC result (not an error); got: {resp}"
    );
    assert!(
        resp.contains("hash") || resp.contains("generated_at") || resp.contains("stale") || resp.contains("model") || resp.contains("timestamp"),
        "ought://manifest must include generation hashes, timestamps, and staleness information; got: {resp}"
    );

    let _ = child.kill();
    child.wait().ok();
    let _ = std::fs::remove_dir_all(&base);
}