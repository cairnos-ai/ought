/// SHOULD be installable via cargo, Homebrew, and as a standalone binary
#[test]
fn test_ought__integration__should_be_installable_via_cargo_homebrew_and_as_a_standalone_binary() {
    // 1. Standalone binary: `ought --version` must succeed with a non-empty version string.
    let bin = option_env!("CARGO_BIN_EXE_ought")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("ought"));

    let version_out = std::process::Command::new(&bin)
        .arg("--version")
        .output()
        .expect("`ought` binary must run as a standalone executable without additional runtime");

    assert!(
        version_out.status.success(),
        "`ought --version` must exit 0 to confirm it runs as a standalone binary; stderr: {}",
        String::from_utf8_lossy(&version_out.stderr)
    );
    let version_str = String::from_utf8_lossy(&version_out.stdout);
    assert!(
        !version_str.trim().is_empty(),
        "`ought --version` must print a version string (required by package managers)"
    );

    // 2. cargo install: the CLI crate's Cargo.toml must declare a [[bin]] target named "ought"
    //    with package name = "ought" so that `cargo install ought` resolves to this binary.
    let manifest_dir = std::path::PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR must be set during cargo test"),
    );
    let workspace_root = manifest_dir
        .ancestors()
        .find(|p| p.join("Cargo.lock").exists())
        .unwrap_or_else(|| manifest_dir.as_path())
        .to_path_buf();

    let cli_toml_path = workspace_root.join("crates").join("ought-cli").join("Cargo.toml");
    assert!(
        cli_toml_path.exists(),
        "crates/ought-cli/Cargo.toml must exist so `cargo install ought` works"
    );

    let cli_toml = std::fs::read_to_string(&cli_toml_path)
        .expect("ought-cli/Cargo.toml must be readable");

    assert!(
        cli_toml.contains("name = \"ought\""),
        "ought-cli/Cargo.toml must set `name = \"ought\"` so the crate is published as 'ought' on crates.io"
    );
    assert!(
        cli_toml.contains("[[bin]]"),
        "ought-cli/Cargo.toml must declare a [[bin]] target for `cargo install` to produce the `ought` binary"
    );

    // 3. Homebrew compatibility: verify the binary produces output on --help,
    //    which Homebrew formulae rely on for bottle validation.
    let help_out = std::process::Command::new(&bin)
        .arg("--help")
        .output()
        .expect("`ought --help` must succeed for Homebrew formula validation");

    let help_combined = format!(
        "{}{}",
        String::from_utf8_lossy(&help_out.stdout),
        String::from_utf8_lossy(&help_out.stderr)
    );
    assert!(
        !help_combined.is_empty(),
        "`ought --help` must produce usage output (required for Homebrew bottle validation)"
    );
}