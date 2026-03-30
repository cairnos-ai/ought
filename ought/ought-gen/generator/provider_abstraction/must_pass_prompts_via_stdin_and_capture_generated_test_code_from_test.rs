/// MUST pass prompts via stdin and capture generated test code from stdout
#[test]
fn test_generator__provider_abstraction__must_pass_prompts_via_stdin_and_capture_generated_test_code_from() {
    use ought_gen::providers::exec_cli;

    // `cat` with no args echoes stdin to stdout — perfect for verifying the
    // stdin-passing contract without needing a real LLM installed.
    let prompt = "fn test_example() { assert!(true); }";
    let result = exec_cli("cat", &[], prompt);
    assert!(
        result.is_ok(),
        "exec_cli must succeed when the command exits 0; got: {:?}",
        result.err()
    );
    let output = result.unwrap();
    assert_eq!(
        output.trim(),
        prompt.trim(),
        "stdout captured by exec_cli must exactly match the data written to stdin"
    );

    // Multi-line prompt must also survive the round-trip intact.
    let multiline = "line one\nline two\nline three";
    let out = exec_cli("cat", &[], multiline).expect("cat must succeed");
    assert!(
        out.contains("line one") && out.contains("line two") && out.contains("line three"),
        "multi-line prompt must be passed to stdin and captured from stdout intact; got: {out}"
    );
}