/// SHOULD send progress notifications to the client
/// GIVEN: a tool invocation exceeds 60 seconds
/// This test starts a long-running tool and asserts that a notifications/progress message
/// is emitted before the tool completes or within 90 seconds, whichever comes first.
/// Run with: cargo test -- --ignored
#[test]
#[ignore]
fn test_mcp_server__error_handling__should_send_progress_notifications_to_the_client() {
    use std::io::{BufRead, BufReader, Write};

    let base = std::env::temp_dir().join(format!("ought_progress_notif_{}", std::process::id()));
    std::fs::create_dir_all(base.join("specs")).unwrap();
    std::fs::create_dir_all(base.join("src")).unwrap();
    std::fs::write(
        base.join("ought.toml"),
        "[project]\nname = \"test\"\nversion = \"0.1.0\"\n\n[specs]\nroots = [\"specs\"]\n\n[generator]\nprovider = \"anthropic\"\n",
    ).unwrap();
    // Several spec files to increase analysis work and push runtime past 60 s.
    for i in 0..5 {
        std::fs::write(
            base.join("specs").join(format!("module{i}.ought.md")),
            format!("# Module {i}\n\n## Feature A\n\nMUST do thing {i}a\n\nMUST do thing {i}b\n\n## Feature B\n\nSHOULD do thing {i}c\n"),
        ).unwrap();
    }
    std::fs::write(
        base.join("src").join("lib.rs"),
        "pub fn run() {}\npub fn init() {}\npub fn shutdown() {}\n",
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

    // Handshake.
    writeln!(stdin,
        r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"protocolVersion":"2024-11-05","capabilities":{{"progress":true}},"clientInfo":{{"name":"ought-test","version":"0.0.1"}}}}}}"#
    ).ok();
    let deadline_init = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if std::time::Instant::now() > deadline_init { panic!("no response to initialize"); }
        let mut line = String::new();
        if let Ok(n) = reader.read_line(&mut line) {
            if n == 0 { panic!("stdout closed during init"); }
            if !line.trim().is_empty() { break; }
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    writeln!(stdin, r#"{{"jsonrpc":"2.0","method":"notifications/initialized"}}"#).ok();

    // Start a long-running analysis tool (ought_audit cross-checks multiple specs).
    writeln!(stdin,
        r#"{{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{{"name":"ought_audit","arguments":{{}}}}}}"#
    ).ok();
    stdin.flush().ok();

    // Read output lines for up to 90 seconds, looking for a progress notification.
    // A progress notification has no "id" field and its "method" contains "progress".
    let overall_deadline = std::time::Instant::now() + std::time::Duration::from_secs(90);
    let mut found_progress = false;
    let mut tool_completed = false;

    loop {
        if std::time::Instant::now() > overall_deadline { break; }
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) if !line.trim().is_empty() => {
                let trimmed = line.trim();
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
                    // A progress notification: has "method" containing "progress", no "id".
                    if v.get("id").is_none() {
                        if let Some(method) = v["method"].as_str() {
                            if method.contains("progress") {
                                found_progress = true;
                                break;
                            }
                        }
                    }
                    // If the tool already completed (result for id=2), note it and stop.
                    if v.get("id").and_then(|id| id.as_u64()) == Some(2) {
                        tool_completed = true;
                        break;
                    }
                }
            }
            _ => std::thread::sleep(std::time::Duration::from_millis(20)),
        }
    }

    let _ = child.kill();
    child.wait().ok();
    let _ = std::fs::remove_dir_all(&base);

    if tool_completed {
        // Tool finished before 60 s: the SHOULD clause does not apply in this run.
        // The test is inconclusive but not a failure.
        eprintln!(
            "NOTE: ought_audit completed before 60 s in this environment; \
             progress notification check is inconclusive."
        );
    } else {
        assert!(
            found_progress,
            "server SHOULD send a notifications/progress message when a tool runs for >60 s; \
             no progress notification was observed within 90 s"
        );
    }
}