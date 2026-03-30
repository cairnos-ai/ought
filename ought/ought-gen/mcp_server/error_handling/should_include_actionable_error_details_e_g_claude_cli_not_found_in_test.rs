/// SHOULD include actionable error details
/// (e.g. "`claude` CLI not found — install it with `brew install claude`" not just "generation failed")
#[test]
fn test_mcp_server__error_handling__should_include_actionable_error_details_e_g_claude_cli_not_found_in() {
    use std::io::{BufRead, BufReader, Write};

    let base = std::env::temp_dir().join(format!("ought_actionable_err_{}", std::process::id()));
    std::fs::create_dir_all(base.join("specs")).unwrap();
    std::fs::write(
        base.join("ought.toml"),
        "[project]\nname = \"test\"\nversion = \"0.1.0\"\n\n[specs]\nroots = [\"specs\"]\n\n[generator]\nprovider = \"anthropic\"\n",
    ).unwrap();
    std::fs::write(
        base.join("specs").join("auth.ought.md"),
        "# Auth\n\n## Login\n\nMUST accept valid credentials\n",
    ).unwrap();

    // Create an empty bin directory — deliberately contains no `claude` binary.
    let empty_bin = base.join("empty_bin");
    std::fs::create_dir_all(&empty_bin).unwrap();
    // Use a PATH that includes standard system utilities but NOT any directory
    // where `claude` is typically installed (/usr/local/bin, ~/.local/bin, etc.).
    let restricted_path = format!("/usr/bin:/bin:/usr/sbin:/sbin:{}", empty_bin.display());

    let bin = option_env!("CARGO_BIN_EXE_ought")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("ought"));
    let mut child = std::process::Command::new(&bin)
        .args(["mcp", "serve"])
        .current_dir(&base)
        .env("PATH", &restricted_path)
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

    // Attempt generation — with no `claude` binary on PATH this must fail.
    let resp = exchange(&mut stdin, &mut reader,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"ought_generate","arguments":{}}}"#,
        15000).expect("server must respond to ought_generate even when claude is unavailable");

    let _ = child.kill();
    child.wait().ok();
    let _ = std::fs::remove_dir_all(&base);

    // The response must be a valid JSON-RPC envelope.
    let v: serde_json::Value = serde_json::from_str(&resp)
        .unwrap_or_else(|_| panic!("response must be valid JSON; got: {resp}"));
    assert_eq!(v["jsonrpc"], "2.0", "must include jsonrpc:2.0");

    // Extract the error message text from wherever the implementation puts it.
    let error_text = {
        let mut texts = vec![];
        if let Some(msg) = v["error"]["message"].as_str() {
            texts.push(msg.to_string());
        }
        if let Some(arr) = v["result"]["content"].as_array() {
            for item in arr {
                if let Some(t) = item["text"].as_str() {
                    texts.push(t.to_string());
                }
            }
        }
        texts.join(" ").to_lowercase()
    };

    assert!(
        !error_text.is_empty(),
        "error response must contain a human-readable message; got: {resp}"
    );

    // The error must be actionable: mention the missing tool and how to obtain it.
    let mentions_claude = error_text.contains("claude");
    let mentions_action = error_text.contains("install")
        || error_text.contains("brew")
        || error_text.contains("not found")
        || error_text.contains("missing")
        || error_text.contains("cannot find")
        || error_text.contains("could not find");
    assert!(
        mentions_claude,
        "error must identify the missing CLI ('claude'); got message: {error_text}"
    );
    assert!(
        mentions_action,
        "error must include an actionable hint (install / not found / etc.); got: {error_text}"
    );
}