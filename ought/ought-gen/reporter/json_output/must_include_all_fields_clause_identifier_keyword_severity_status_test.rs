/// MUST include all fields: clause identifier, keyword, severity, status, failure message, duration
#[test]
fn test_reporter__json_output__must_include_all_fields_clause_identifier_keyword_severity_status() {
    use ought_report::json;
    use ought_run::{RunResult, TestDetails, TestResult, TestStatus};
    use ought_spec::{Clause, ClauseId, Keyword, Section, Severity, SourceLocation, Spec};
    use serde_json::Value;
    use std::path::PathBuf;
    use std::time::Duration;

    let clause_id = ClauseId("auth::login::must_return_401_for_bad_creds".to_string());

    let clause = Clause {
        id: clause_id.clone(),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: "return 401 for invalid credentials".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation {
            file: PathBuf::from("auth.ought.md"),
            line: 11,
        },
        content_hash: "def456".to_string(),
    };

    let spec = Spec {
        name: "Auth Spec".to_string(),
        metadata: Default::default(),
        sections: vec![Section {
            title: "Login".to_string(),
            depth: 1,
            prose: String::new(),
            clauses: vec![clause],
            subsections: vec![],
        }],
        source_path: PathBuf::from("auth.ought.md"),
    };

    let failure_msg = "expected status 401, got 200".to_string();
    let run_result = RunResult {
        results: vec![TestResult {
            clause_id: clause_id.clone(),
            status: TestStatus::Failed,
            message: Some(failure_msg.clone()),
            duration: Duration::from_millis(22),
            details: TestDetails::default(),
        }],
        total_duration: Duration::from_millis(22),
    };

    let output = json::report(&run_result, &[spec])
        .expect("json::report must not fail");

    let parsed: Value = serde_json::from_str(&output)
        .expect("output must be valid JSON");

    let results = parsed["specs"][0]["results"]
        .as_array()
        .expect("specs[0].results must be a JSON array");
    assert_eq!(results.len(), 1, "must emit exactly one clause result entry");

    let entry = &results[0];

    // --- clause identifier ---
    assert_eq!(
        entry["clause_id"].as_str().unwrap_or(""),
        "auth::login::must_return_401_for_bad_creds",
        "clause_id must match the original clause identifier"
    );

    // --- keyword ---
    let keyword = entry["keyword"].as_str().expect("'keyword' must be a string");
    assert_eq!(keyword, "MUST", "keyword must render as 'MUST'");

    // --- severity ---
    let severity = entry["severity"].as_str().expect("'severity' must be a string");
    assert_eq!(severity, "required", "severity for a MUST clause must be 'required'");

    // --- status ---
    let status = entry["status"].as_str().expect("'status' must be a string");
    assert_eq!(status, "failed", "status must reflect the test outcome");

    // --- failure message ---
    let msg = entry["message"]
        .as_str()
        .expect("'message' must be present and non-null for a failed clause");
    assert_eq!(
        msg, failure_msg,
        "failure message must match what was recorded on the TestResult"
    );

    // --- duration ---
    let dur = entry["duration_ms"].as_f64().expect("'duration_ms' must be a number");
    assert!(dur > 0.0, "duration_ms must be positive, got {dur}");
    assert!(
        (dur - 22.0).abs() < 1.0,
        "duration_ms must approximate the run duration (22 ms), got {dur}"
    );
}