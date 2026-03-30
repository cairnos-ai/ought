/// MUST be written in Rust
///
/// Verifies that the ought project is a Rust project by checking that
/// a Cargo.toml workspace file exists at the repository root.
#[test]
fn test_ought__implementation__must_be_written_in_rust() {
    use std::path::Path;

    // Locate the workspace root relative to this test's manifest directory.
    // CARGO_MANIFEST_DIR points to the crate containing this test; walk up
    // until we find a Cargo.toml that declares a workspace.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set by cargo");

    let mut dir = Path::new(&manifest_dir).to_path_buf();
    let mut found_workspace = false;

    loop {
        let candidate = dir.join("Cargo.toml");
        if candidate.exists() {
            let contents = std::fs::read_to_string(&candidate)
                .expect("failed to read Cargo.toml");
            if contents.contains("[workspace]") {
                found_workspace = true;
                break;
            }
        }
        if !dir.pop() {
            break;
        }
    }

    assert!(
        found_workspace,
        "Expected to find a Cargo.toml with a [workspace] section, \
         confirming the project is written in Rust"
    );
}