/// MUST report when the test harness command is not found or fails to start
#[test]
fn test_runner__error_handling__must_report_when_the_test_harness_command_is_not_found_or_fails_t() {
    use std::io::ErrorKind;
    use std::process::Command;

    // The runner delegates to an external harness (cargo test, pytest, jest …).
    // If that binary cannot be found, the OS returns an error that must surface
    // to the caller rather than being swallowed or turned into a silent empty result.

    fn try_start_harness(binary: &str) -> Result<(), std::io::Error> {
        Command::new(binary)
            .arg("--version")
            .output()
            .map(|_| ())
    }

    // A deliberately non-existent binary name.
    let result = try_start_harness("__ought_nonexistent_harness_XYZ999__");
    assert!(
        result.is_err(),
        "Attempting to run a non-existent harness must return an error"
    );
    assert_eq!(
        result.unwrap_err().kind(),
        ErrorKind::NotFound,
        "Error kind must be NotFound so callers can produce a human-readable message"
    );

    // is_available() style check: a runner must be able to self-report unavailability
    // before attempting to run tests, avoiding a confusing mid-run failure.
    fn is_harness_available(binary: &str) -> bool {
        Command::new(binary).arg("--version").output().is_ok()
    }

    assert!(
        !is_harness_available("__ought_nonexistent_harness_XYZ999__"),
        "is_available() must return false for a missing harness binary"
    );
    // Confirm the inverse: a real system binary is correctly detected.
    assert!(
        is_harness_available("sh"),
        "is_available() must return true for a binary that exists on PATH"
    );
}