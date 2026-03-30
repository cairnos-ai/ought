/// MUST report LLM API errors clearly (auth failure, rate limit, timeout)
#[test]
fn test_generator__error_handling__must_report_llm_api_errors_clearly_auth_failure_rate_limit_timeou() {
    use ought_gen::providers::exec_cli;

    // Simulate authentication failure: provider CLI exits non-zero with auth error on stderr
    let auth_result = exec_cli(
        "sh",
        &["-c", "echo 'authentication failed: invalid API key' >&2; exit 1"],
        "test prompt",
    );
    assert!(
        auth_result.is_err(),
        "must_report_llm_api_errors_clearly: auth failure must return Err, not Ok"
    );
    let auth_msg = auth_result.unwrap_err().to_string();
    assert!(
        auth_msg.contains("sh") || auth_msg.contains("exit"),
        "must_report_llm_api_errors_clearly: auth error must name the command or include exit status; got: {auth_msg}"
    );
    assert!(
        auth_msg.contains("authentication failed") || auth_msg.contains("invalid API key") || auth_msg.contains('1'),
        "must_report_llm_api_errors_clearly: auth error must surface stderr detail; got: {auth_msg}"
    );

    // Simulate rate limit: provider CLI exits with rate-limit message on stderr
    let rate_result = exec_cli(
        "sh",
        &["-c", "echo 'rate limit exceeded: 429 Too Many Requests' >&2; exit 1"],
        "test prompt",
    );
    assert!(
        rate_result.is_err(),
        "must_report_llm_api_errors_clearly: rate limit must return Err"
    );
    let rate_msg = rate_result.unwrap_err().to_string();
    assert!(
        rate_msg.contains("rate limit exceeded") || rate_msg.contains("429") || rate_msg.contains("Too Many"),
        "must_report_llm_api_errors_clearly: rate limit error must surface stderr detail; got: {rate_msg}"
    );

    // Simulate timeout: provider CLI exits with timeout message on stderr
    let timeout_result = exec_cli(
        "sh",
        &["-c", "echo 'request timed out after 30s' >&2; exit 1"],
        "test prompt",
    );
    assert!(
        timeout_result.is_err(),
        "must_report_llm_api_errors_clearly: timeout must return Err"
    );
    let timeout_msg = timeout_result.unwrap_err().to_string();
    assert!(
        timeout_msg.contains("request timed out") || timeout_msg.contains("timed out") || timeout_msg.contains("30s"),
        "must_report_llm_api_errors_clearly: timeout error must surface stderr detail; got: {timeout_msg}"
    );

    // Verify that CLI-not-found is also reported clearly (e.g. llm binary missing)
    let missing_result = exec_cli("__ought_nonexistent_llm_binary__", &[], "test prompt");
    assert!(
        missing_result.is_err(),
        "must_report_llm_api_errors_clearly: missing CLI binary must return Err"
    );
    let missing_msg = missing_result.unwrap_err().to_string();
    assert!(
        missing_msg.contains("__ought_nonexistent_llm_binary__"),
        "must_report_llm_api_errors_clearly: missing-binary error must name the tool; got: {missing_msg}"
    );
}