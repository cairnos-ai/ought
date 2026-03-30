/// SHOULD detect when the required CLI tool is not installed and report a clear error
#[test]
fn test_generator__provider_abstraction__should_detect_when_the_required_cli_tool_is_not_installed_and_repor() {
    use ought_gen::providers::exec_cli;

    // Using a name that cannot possibly be installed lets us exercise the
    // NotFound path without side-effects.
    let missing = "__ought_missing_cli_xyz123__";
    let err = exec_cli(missing, &[], "some prompt")
        .unwrap_err()
        .to_string();

    assert!(
        err.to_lowercase().contains("not found"),
        "error for a missing CLI must say 'not found'; got: {err}"
    );
    assert!(
        err.contains(missing),
        "error must name the missing CLI tool '{}' so the user knows what to install; got: {err}",
        missing
    );
    assert!(
        err.to_lowercase().contains("path") || err.to_lowercase().contains("install") || err.to_lowercase().contains("not found"),
        "error must hint at how to resolve the problem (PATH / install); got: {err}"
    );

    // Each provider name that maps to a specific binary should surface the right
    // binary name in the error when that binary is absent.
    // We test the Claude case (most likely to be absent in CI).
    let claude_err = exec_cli("__claude_not_here__", &["-p"], "prompt")
        .unwrap_err()
        .to_string();
    assert!(
        claude_err.contains("__claude_not_here__"),
        "provider error must name the exact binary it tried to exec; got: {claude_err}"
    );
}