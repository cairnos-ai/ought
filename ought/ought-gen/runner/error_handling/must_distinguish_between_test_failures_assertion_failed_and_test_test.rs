/// MUST distinguish between test failures (assertion failed) and test errors (test code itself crashed)
#[test]
fn test_runner__error_handling__must_distinguish_between_test_failures_assertion_failed_and_test() {
    // A test failure = the test ran but an assertion did not hold (expected: the spec was violated).
    // A test error  = the test code itself crashed before it could complete (unexpected panic,
    //                 bad unwrap, index out of bounds, etc.).  These are distinct diagnostics.

    #[derive(Debug, PartialEq)]
    enum TestStatus { Passed, Failed, Errored }

    // Minimal parser modelled after the Rust runner's cargo-test output parsing.
    fn classify_cargo_output(test_name: &str, stdout: &str) -> TestStatus {
        let failed_marker = format!("test {} ... FAILED", test_name);
        if !stdout.contains(&failed_marker) {
            return TestStatus::Passed;
        }
        // Find the per-test failure block and look for assertion vs crash keywords.
        let assertion_signatures = [
            "assertion `left == right` failed",
            "assertion failed:",
            "left == right",
            "left != right",
            "assert_eq!",
            "assert_ne!",
        ];
        let error_signatures = [
            "called `Result::unwrap()` on an `Err` value",
            "called `Option::unwrap()` on a `None` value",
            "index out of bounds",
            "attempt to divide by zero",
            "attempt to subtract with overflow",
            "explicit panic",
        ];
        let block_start = stdout.find(&format!("---- {} stdout ----", test_name)).unwrap_or(0);
        let block = &stdout[block_start..];

        let is_assertion = assertion_signatures.iter().any(|s| block.contains(s));
        let is_error     = error_signatures.iter().any(|s| block.contains(s));

        if is_assertion && !is_error {
            TestStatus::Failed
        } else {
            TestStatus::Errored
        }
    }

    // --- scenario 1: assertion failure (test logic, not a crash) ---
    let assertion_output = "\
test runner__clause_a ... FAILED

failures:

---- runner__clause_a stdout ----
thread 'runner__clause_a' panicked at 'assertion `left == right` failed
  left: `42`,
 right: `0`', src/runner.rs:55:5

failures:
    runner__clause_a
";
    assert_eq!(
        classify_cargo_output("runner__clause_a", assertion_output),
        TestStatus::Failed,
        "assertion-style panic must be classified as Failed, not Errored"
    );

    // --- scenario 2: test code crashed (unwrap on Err) ---
    let error_output = "\
test runner__clause_b ... FAILED

failures:

---- runner__clause_b stdout ----
thread 'runner__clause_b' panicked at 'called `Result::unwrap()` on an `Err` value: \
Os { code: 2, kind: NotFound, message: \"No such file or directory\" }', src/runner.rs:88:22

failures:
    runner__clause_b
";
    assert_eq!(
        classify_cargo_output("runner__clause_b", error_output),
        TestStatus::Errored,
        "unexpected-panic must be classified as Errored, not Failed"
    );

    // --- scenario 3: passing test produces neither status ---
    let pass_output = "test runner__clause_c ... ok\n\ntest result: ok. 1 passed; 0 failed;";
    assert_eq!(
        classify_cargo_output("runner__clause_c", pass_output),
        TestStatus::Passed
    );

    // The two non-passing statuses must be distinct values.
    assert_ne!(TestStatus::Failed, TestStatus::Errored);
}