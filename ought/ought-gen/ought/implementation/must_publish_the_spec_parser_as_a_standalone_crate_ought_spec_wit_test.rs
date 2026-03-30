/// MUST publish the spec parser as a standalone crate (`ought-spec`) with no LLM dependencies
///
/// Verifies that:
///   1. A crate named `ought-spec` exists in the workspace.
///   2. Its Cargo.toml does not declare any known LLM-related dependencies
///      (e.g. openai, anthropic, mistral, llm, langchain, rig, llama).
#[test]
fn test_ought__implementation__must_publish_the_spec_parser_as_a_standalone_crate_ought_spec_wit() {
    use std::path::Path;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set by cargo");

    // Walk up to workspace root.
    let mut workspace_root = Path::new(&manifest_dir).to_path_buf();
    loop {
        let candidate = workspace_root.join("Cargo.toml");
        if candidate.exists() {
            let contents = std::fs::read_to_string(&candidate)
                .expect("failed to read Cargo.toml");
            if contents.contains("[workspace]") {
                break;
            }
        }
        assert!(
            workspace_root.pop(),
            "Could not find workspace root from CARGO_MANIFEST_DIR"
        );
    }

    // Locate ought-spec crate — accept crates/ought-spec or ought-spec at root.
    let candidates = [
        workspace_root.join("crates").join("ought-spec").join("Cargo.toml"),
        workspace_root.join("ought-spec").join("Cargo.toml"),
    ];

    let spec_manifest_path = candidates
        .iter()
        .find(|p| p.exists())
        .expect("ought-spec crate not found; expected at crates/ought-spec/Cargo.toml");

    let spec_manifest = std::fs::read_to_string(spec_manifest_path)
        .expect("failed to read ought-spec/Cargo.toml");

    // Confirm the crate is named ought-spec.
    assert!(
        spec_manifest.contains("name = \"ought-spec\""),
        "ought-spec/Cargo.toml should declare name = \"ought-spec\""
    );

    // Ensure no LLM-related dependencies are present.
    let llm_deps = [
        "openai",
        "anthropic",
        "mistral",
        "\"llm\"",
        "langchain",
        "rig",
        "llama",
        "ollama",
        "genai",
        "async-openai",
    ];

    for dep in &llm_deps {
        assert!(
            !spec_manifest.contains(dep),
            "ought-spec/Cargo.toml must not depend on LLM library '{}', \
             but it was found in the manifest",
            dep
        );
    }
}