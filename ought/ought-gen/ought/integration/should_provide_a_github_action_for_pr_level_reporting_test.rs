/// SHOULD provide a GitHub Action for PR-level reporting
#[test]
fn test_ought__integration__should_provide_a_github_action_for_pr_level_reporting() {
    let manifest_dir = std::path::PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR must be set during cargo test"),
    );

    // Walk up to the workspace root (the directory containing Cargo.lock).
    let workspace_root = manifest_dir
        .ancestors()
        .find(|p| p.join("Cargo.lock").exists())
        .unwrap_or_else(|| manifest_dir.as_path())
        .to_path_buf();

    // The GitHub Action can be declared at the repository root or under .github/actions/ought/.
    let candidates = [
        workspace_root.join("action.yml"),
        workspace_root.join("action.yaml"),
        workspace_root.join(".github").join("actions").join("ought").join("action.yml"),
        workspace_root.join(".github").join("actions").join("ought").join("action.yaml"),
    ];

    let action_exists = candidates.iter().any(|p| p.exists());

    assert!(
        action_exists,
        "A GitHub Action definition (action.yml) must exist at the repository root or under \
        .github/actions/ought/ to enable PR-level reporting. Searched: {candidates:?}",
        candidates = candidates
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
    );

    // If the action exists, verify it is valid YAML containing the required `runs` key.
    for path in candidates.iter().filter(|p| p.exists()) {
        let contents = std::fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("action file at {} must be readable", path.display()));
        assert!(
            contents.contains("runs:"),
            "action file at {} must contain a `runs:` key to be a valid GitHub Action",
            path.display()
        );
    }
}