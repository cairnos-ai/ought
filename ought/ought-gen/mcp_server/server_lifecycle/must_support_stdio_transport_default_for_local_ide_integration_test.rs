/// MUST support stdio transport (default, for local IDE integration)
#[test]
fn test_mcp_server__server_lifecycle__must_support_stdio_transport_default_for_local_ide_integration() {
    let bin = option_env!("CARGO_BIN_EXE_ought")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("ought"));

    // --help must document stdio as the default transport value.
    let output = std::process::Command::new(&bin)
        .args(["mcp", "serve", "--help"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .expect("`ought mcp serve --help` must run");

    let help = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        help.to_lowercase().contains("stdio"),
        "`ought mcp serve --help` must document 'stdio' as the default transport; got:\n{help}"
    );

    // Verify invoking without --transport does not produce a usage error,
    // confirming stdio is the default (no flag required).
    let base = std::env::temp_dir()
        .join(format!("ought_mcp_stdio_default_{}", std::process::id()));
    std::fs::create_dir_all(base.join("specs")).unwrap();
    std::fs::write(
        base.join("ought.toml"),
        "[project]\nname = \"test\"\nversion = \"0.1.0\"\n\n[specs]\nroots = [\"specs\"]\n\n[generator]\nprovider = \"anthropic\"\n",
    )
    .unwrap();

    let mut child = std::process::Command::new(&bin)
        .args(["mcp", "serve"])
        .current_dir(&base)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("`ought mcp serve` (no --transport) must spawn");

    std::thread::sleep(std::time::Duration::from_millis(200));
    let _ = child.kill();
    let out = child.wait_with_output().expect("wait");

    assert_ne!(
        out.status.code(),
        Some(2),
        "omitting --transport must not produce a usage error (stdio must be the default); \
         stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let _ = std::fs::remove_dir_all(&base);
}