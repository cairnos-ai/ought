/// MUST show the assertion error or failure message for each failed clause
#[test]
fn test_reporter__failure_details__must_show_the_assertion_error_or_failure_message_for_each_failed() {
    use ought_report::types::{ColorChoice, ReportOptions};
    use ought_run::{RunResult, TestDetails, TestResult, TestStatus};
    use ought_spec::{Clause, ClauseId, Keyword, Section, Severity, Spec, SourceLocation};
    use std::path::PathBuf;
    use std::time::Duration;

    let clause_id = ClauseId("auth::login::must_reject_invalid_credentials".to_string());
    let failure_msg = "assertion `left == right` failed\n  left: 200\n right: 401";

    let clause = Clause {
        id: clause_id.clone(),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: "reject invalid credentials with 401".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("spec.md"), line: 10 },
        content_hash: "abc123".to_string(),
    };

    let section = Section {
        title: "Authentication".to_string(),
        depth: 1,
        prose: String::new(),
        clauses: vec![clause],
        subsections: vec![],
    };

    let spec = Spec {
        name: "Auth Spec".to_string(),
        metadata: Default::default(),
        sections: vec![section],
        source_path: PathBuf::from("spec.md"),
    };

    // Test with failure_message in details
    let run_result = RunResult {
        results: vec![TestResult {
            clause_id: clause_id.clone(),
            status: TestStatus::Failed,
            message: None,
            duration: Duration::from_millis(3),
            details: TestDetails {
                failure_message: Some(failure_msg.to_string()),
                ..Default::default()
            },
        }],
        total_duration: Duration::from_millis(3),
    };

    let options = ReportOptions {
        color: ColorChoice::Never,
        ..Default::default()
    };

    let mut output = Vec::new();
    ought_report::terminal::render_to_writer(&mut output, &run_result, &[spec.clone()], &options)
        .expect("render_to_writer should succeed");
    let text = String::from_utf8(output).expect("output should be valid UTF-8");

    assert!(
        text.contains("assertion `left == right` failed"),
        "output must contain the assertion error message for a failed clause; got:\n{text}"
    );
    assert!(
        text.contains("left: 200"),
        "output must include the full multi-line failure message; got:\n{text}"
    );
    assert!(
        text.contains("right: 401"),
        "output must include all lines of the failure message; got:\n{text}"
    );

    // Test fallback: message field used when details.failure_message is absent
    let run_result_msg_only = RunResult {
        results: vec![TestResult {
            clause_id: clause_id.clone(),
            status: TestStatus::Failed,
            message: Some("test panicked: expected 401 got 200".to_string()),
            duration: Duration::from_millis(3),
            details: TestDetails {
                failure_message: None,
                ..Default::default()
            },
        }],
        total_duration: Duration::from_millis(3),
    };

    let mut output2 = Vec::new();
    ought_report::terminal::render_to_writer(
        &mut output2,
        &run_result_msg_only,
        &[spec],
        &options,
    )
    .expect("render_to_writer should succeed");
    let text2 = String::from_utf8(output2).expect("output should be valid UTF-8");

    assert!(
        text2.contains("expected 401 got 200"),
        "output must show the message field as fallback when details.failure_message is absent; got:\n{text2}"
    );
}