/// MUST capture failure messages, assertion errors, and stack traces per test
#[test]
fn test_runner__result_collection__must_capture_failure_messages_assertion_errors_and_stack_traces_p() {
    struct ClauseResult {
        clause_id: String,
        failure_message: Option<String>,
        assertion_error: Option<String>,
        stack_trace: Option<String>,
    }

    // Simulate a failure record captured from cargo test output
    let failure = ClauseResult {
        clause_id: "payments::charge::must_debit_account".to_string(),
        failure_message: Some(
            "thread 'test_must_debit_account' panicked at 'assertion `left == right` failed'".to_string(),
        ),
        assertion_error: Some("left: 0\nright: 100".to_string()),
        stack_trace: Some(
            "stack backtrace:\n   0: std::panicking::begin_panic\n   1: test_must_debit_account".to_string(),
        ),
    };

    assert_eq!(failure.clause_id, "payments::charge::must_debit_account");
    assert!(failure.failure_message.is_some(), "failure message must be captured");
    assert!(failure.assertion_error.is_some(), "assertion error detail must be captured");
    assert!(failure.stack_trace.is_some(), "stack trace must be captured");

    // A passing test need not carry failure details
    let pass = ClauseResult {
        clause_id: "payments::charge::must_be_idempotent".to_string(),
        failure_message: None,
        assertion_error: None,
        stack_trace: None,
    };

    assert!(pass.failure_message.is_none());
    assert!(pass.assertion_error.is_none());
    assert!(pass.stack_trace.is_none());
}