/// OTHERWISE the client may assume the request has timed out
/// GIVEN: a tool invocation exceeds 60 seconds (and no progress notification was received)
/// Tests that a client closing the connection after a 60-second timeout is handled gracefully:
/// the server must not crash or produce corrupt output.
/// Run with: cargo test -- --ignored
#[test]
#[ignore]
fn test_mcp_server__error_handling__otherwise_the_client_may_assume_the_request_has_timed_out() {
    use std::io::{BufRead, BufReader, Write};

    let base = std::env::temp_dir().join(format!("ought_client_timeout_{}", std::process::id()));
    std::fs::create_dir_all(base.join("specs")).unwrap();
    std::fs::create_dir_all(base.join("src")).unwrap();
    std::fs::write(
        base.join("ought.toml"),
        "[project]\nname = \"test\"\nversion = \"0.1.0\"\n\n[specs]\nroots = [\"specs\"]\n\n[generator]\nprovider = \"anthropic\"\n",
    ).unwrap();
    for i in 0..5 {
        std::fs::write(
            base.join("specs").join(format!("module{i}.ought.md")),
            format!("# Module {i}\n\n## Feature\n\nMUST do thing {i}a\n\nMUST do thing {i}b\n"),
        ).unwrap();
    }
    std::fs::write(base.join("src").join("lib.rs"), "pub fn run() {}\n").unwrap();

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
        r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"protocolVersion":"2024-11-05","capabilities":{{}},"clientInfo":{{"name":"ought-test","version":"0.0.1"}}}}}}"#
    ).ok();
    let init_deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if std::time::Instant::now() > init_deadline { panic!("no response to initialize"); }
        let mut line = String::new();
        if let Ok(n) = reader.read_line(&mut line) {
            if n == 0 { panic!("stdout closed during init"); }
            if !line.trim().is_empty() { break; }
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    writeln!(stdin, r#"{{"jsonrpc":"2.0","method":"notifications/initialized"}}"#).ok();

    // Start a long-running tool.
    writeln!(stdin,
        r#"{{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{{"name":"ought_audit","arguments":{{}}}}}}"#
    ).ok();
    stdin.flush().ok();

    // Read for up to 60 seconds, watching for a progress notification.
    let timeout_deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    let mut received_progress = false;
    let mut tool_completed = false;

    'read_loop: loop {
        if std::time::Instant::now() > timeout_deadline { break; }
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) if !line.trim().is_empty() => {
                let trimmed = line.trim();
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
                    if v.get("id").is_none() {
                        if let Some(method) = v["method"].as_str() {
                            if method.contains("progress") {
                                received_progress = true;
                                break 'read_loop;
                            }
                        }
                    }
                    if v.get("id").and_then(|id| id.as_u64()) == Some(2) {
                        tool_completed = true;
                        break 'read_loop;
                    }
                }
            }
            _ => std::thread::sleep(std::time::Duration::from_millis(20)),
        }
    }

    // If 60 s elapsed with no progress notification and the tool didn't complete,
    // the client is justified in closing the connection (simulating a client-side timeout).
    let simulating_timeout = !received_progress && !tool_completed;
    if simulating_timeout {
        // Simulate client timeout: close stdin (EOF) to disconnect.
        drop(stdin);

        // The server must exit cleanly within 5 seconds of the client disconnecting.
        let exit_deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            if let Some(status) = child.try_wait().expect("try_wait") {
                // Server exited — verify it did not crash.
                #[cfg(unix)]
                {
                    use std::os::unix::process::ExitStatusExt;
                    let crashed = status.signal().map_or(false, |s| s == 6 || s == 11);
                    assert!(
                        !crashed,
                        "server must not crash (SIGABRT/SIGSEGV) when client disconnects after timeout; \
                         exit: {status}"
                    );
                }
                let _ = std::fs::remove_dir_all(&base);
                return;
            }
            if std::time::Instant::now() > exit_deadline {
                child.kill().ok();
                child.wait().ok();
                let _ = std::fs::remove_dir_all(&base);
                panic!(
                    "server did not exit within 5 s after client disconnected (simulated 60 s timeout); \
                     when a client times out the server must handle the disconnect gracefully"
                );
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    } else {
        // Tool completed or sent a progress notification before the 60-second mark —
        // the OTHERWISE condition did not apply in this run.
        let _ = child.kill();
        child.wait().ok();
        let _ = std::fs::remove_dir_all(&base);
        if received_progress {
            eprintln!("NOTE: server sent a progress notification — SHOULD clause satisfied; OTHERWISE not triggered.");
        } else {
            eprintln!("NOTE: tool completed before 60 s — OTHERWISE clause not triggered in this run.");
        }
    }
}