/// SHOULD use a workspace structure so components can be used independently
///
/// Verifies that:
///   1. The workspace Cargo.toml declares multiple members.
///   2. Each listed member directory contains its own Cargo.toml (i.e. is a
///      real, independently addressable crate).
#[test]
fn test_ought__implementation__should_use_a_workspace_structure_so_components_can_be_used_independ() {
    use std::path::Path;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set by cargo");

    // Walk up to workspace root.
    let mut workspace_root = Path::new(&manifest_dir).to_path_buf();
    let workspace_toml;
    loop {
        let candidate = workspace_root.join("Cargo.toml");
        if candidate.exists() {
            let contents = std::fs::read_to_string(&candidate)
                .expect("failed to read Cargo.toml");
            if contents.contains("[workspace]") {
                workspace_toml = contents;
                break;
            }
        }
        assert!(
            workspace_root.pop(),
            "Could not find workspace root from CARGO_MANIFEST_DIR"
        );
    }

    // Extract member paths from `members = [...]`.
    // We do a simple line-by-line scan rather than pulling in a TOML parser,
    // keeping the test self-contained.
    let member_lines: Vec<&str> = workspace_toml
        .lines()
        .filter(|l| {
            let trimmed = l.trim();
            // Quoted strings inside the members array look like: "crates/ought-spec",
            trimmed.starts_with('"') && trimmed.ends_with("\",")
                || (trimmed.starts_with('"') && trimmed.ends_with('"'))
        })
        .collect();

    // Parse the member paths (strip quotes and trailing comma).
    let members: Vec<String> = member_lines
        .iter()
        .map(|l| {
            l.trim()
                .trim_matches('"')
                .trim_end_matches(',')
                .trim_matches('"')
                .to_string()
        })
        .filter(|s| !s.is_empty())
        .collect();

    assert!(
        members.len() >= 2,
        "Workspace should declare at least 2 members to enable independent use; \
         found {} member(s). Raw members section may need parser update.",
        members.len()
    );

    // Every declared member must have its own Cargo.toml.
    for member in &members {
        let member_manifest = workspace_root.join(member).join("Cargo.toml");
        assert!(
            member_manifest.exists(),
            "Workspace member '{}' does not have its own Cargo.toml at '{}'",
            member,
            member_manifest.display()
        );
    }
}