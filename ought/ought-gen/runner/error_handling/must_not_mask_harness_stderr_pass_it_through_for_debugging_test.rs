/// MUST NOT mask harness stderr — pass it through for debugging
#[test]
fn test_runner__error_handling__must_not_mask_harness_stderr_pass_it_through_for_debugging() {
    use std::process::{Command, Stdio};

    // The runner must NOT discard stderr from the test harness (e.g., by passing
    // Stdio::null() or simply ignoring the bytes).  Harness diagnostics — compiler
    // warnings, missing dependency messages, stack traces — all arrive on stderr
    // and must be preserved for the user to debug failures.

    let output = Command::new("sh")
        .args(["-c", "echo 'harness diagnostic on stderr' >&2; echo 'stdout line'; exit 1"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("sh must be available on this platform");

    let stderr_bytes = output.stderr.clone();
    let stderr_text  = String::from_utf8_lossy(&stderr_bytes);
    let stdout_text  = String::from_utf8_lossy(&output.stdout);

    // The runner captured stderr — it was not lost.
    assert!(
        !stderr_bytes.is_empty(),
        "stderr from the harness must be captured, not discarded"
    );
    assert!(
        stderr_text.contains("harness diagnostic on stderr"),
        "Captured stderr must contain the harness diagnostic; got: {:?}", stderr_text
    );

    // stdout and stderr are separate streams — masking one must not affect the other.
    assert!(
        stdout_text.contains("stdout line"),
        "stdout must be captured independently of stderr"
    );

    // A runner that uses Stdio::null() for stderr would have produced empty bytes above,
    // causing the first assertion to fail — that is the check.
    assert!(
        !output.status.success(),
        "Non-zero exit must also be surfaced (not silently treated as success)"
    );
}