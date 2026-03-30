/// MUST shut down cleanly on SIGTERM or client disconnect
#[test]
fn test_mcp_server__server_lifecycle__must_shut_down_cleanly_on_sigterm_or_client_disconnect() {
    let base = std::env::temp_dir()
        .join(format!("ought_mcp_sigterm_{}", std::process::id()));
    std::fs::create_dir_all(base.join("specs")).unwrap();
    std::fs::write(
        base.join("ought.toml"),
        "[project]\nname = \"test\"\nversion = \"0.1.0\"\n\n[specs]\nroots = [\"specs\"]\n\n[generator]\nprovider = \"anthropic\"\n",
    )
    .unwrap();

    let bin = option_env!("CARGO_BIN_EXE_ought")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("ought"));

    // ── Sub-test 1: clean shutdown on SIGTERM ────────────────────────────────

    let mut child = std::process::Command::new(&bin)
        .args(["mcp", "serve"])
        .current_dir(&base)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn for SIGTERM test");

    // Allow the server to start up.
    std::thread::sleep(std::time::Duration::from_millis(300));

    // Send SIGTERM.
    #[cfg(unix)]
    {
        std::process::Command::new("kill")
            .args(["-TERM", &child.id().to_string()])
            .status()
            .expect("kill -TERM must succeed");
    }
    #[cfg(not(unix))]
    {
        child.kill().expect("terminate process on non-Unix");
    }

    // The server must exit within 5 seconds of receiving SIGTERM.
    let sigterm_deadline =
        std::time::Instant::now() + std::time::Duration::from_secs(5);
    let sigterm_status = loop {
        if let Some(s) = child.try_wait().expect("try_wait") {
            break s;
        }
        if std::time::Instant::now() > sigterm_deadline {
            child.kill().ok();
            panic!("MCP server did not exit within 5 s after SIGTERM");
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    };

    // Accept either explicit exit(0) or signal-terminated (SIGTERM = 15 on Unix).
    // Both constitute a clean shutdown — the process must not crash (SIGABRT/SIGSEGV).
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        let crash_signal = sigterm_status.signal().map_or(false, |s| s == 6 || s == 11); // SIGABRT=6, SIGSEGV=11
        assert!(
            !crash_signal,
            "MCP server must not crash on SIGTERM; exit status: {sigterm_status}"
        );
    }

    // ── Sub-test 2: clean shutdown on client disconnect (stdin EOF) ──────────

    let mut child2 = std::process::Command::new(&bin)
        .args(["mcp", "serve"])
        .current_dir(&base)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn for disconnect test");

    std::thread::sleep(std::time::Duration::from_millis(300));

    // Close stdin to signal EOF — the MCP server must detect the client has disconnected.
    drop(child2.stdin.take());

    let disconnect_deadline =
        std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if child2.try_wait().expect("try_wait").is_some() {
            break;
        }
        if std::time::Instant::now() > disconnect_deadline {
            child2.kill().ok();
            panic!("MCP server did not exit within 5 s after client disconnect (stdin EOF)");
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    let _ = std::fs::remove_dir_all(&base);
}