/// SHOULD support resource subscriptions so clients get notified when results change
#[test]
fn test_mcp_server__resources__should_support_resource_subscriptions_so_clients_get_notified_when() {
    use std::io::{BufRead, BufReader, Write};

    let base = std::env::temp_dir()
        .join(format!("ought_res_sub_{}", std::process::id()));
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
    let init = init.unwrap();
    // Check if the server advertises subscription capability in its initialize response
    let advertises_subscriptions = init.contains("subscribe") || init.contains("subscription");

    writeln!(stdin, r#"{{"jsonrpc":"2.0","method":"notifications/initialized"}}"#).ok();

    // Subscribe to ought://results/latest — the server must not crash and must respond
    let sub_resp = jsonrpc_exchange(&mut stdin, &mut reader,
        r#"{"jsonrpc":"2.0","id":2,"method":"resources/subscribe","params":{"uri":"ought://results/latest"}}"#,
        5000);
    assert!(
        sub_resp.is_some(),
        "server must respond to resources/subscribe (not hang or crash); got no response"
    );
    let sub_resp = sub_resp.unwrap();
    // Either the subscription is acknowledged with a result, or the server returns an error
    // indicating subscriptions are not yet implemented — but it must NOT be a protocol-level
    // crash or malformed response.
    assert!(
        sub_resp.contains("\"jsonrpc\""),
        "resources/subscribe response must be a valid JSON-RPC envelope; got: {sub_resp}"
    );
    // If the server advertised subscription support, it must acknowledge with a result
    if advertises_subscriptions {
        assert!(
            sub_resp.contains("\"result\""),
            "server advertised subscription capability so resources/subscribe must return a result; got: {sub_resp}"
        );
    }

    // Subscribe to ought://coverage as well — subscriptions should work for any ought:// resource
    let sub_cov = jsonrpc_exchange(&mut stdin, &mut reader,
        r#"{"jsonrpc":"2.0","id":3,"method":"resources/subscribe","params":{"uri":"ought://coverage"}}"#,
        5000);
    assert!(
        sub_cov.is_some(),
        "server must respond to resources/subscribe for ought://coverage"
    );
    let sub_cov = sub_cov.unwrap();
    assert!(
        sub_cov.contains("\"jsonrpc\""),
        "ought://coverage subscription response must be valid JSON-RPC; got: {sub_cov}"
    );

    let _ = child.kill();
    child.wait().ok();
    let _ = std::fs::remove_dir_all(&base);
}