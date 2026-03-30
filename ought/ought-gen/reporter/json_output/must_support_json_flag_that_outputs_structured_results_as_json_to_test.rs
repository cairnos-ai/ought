/// MUST support `--json` flag that outputs structured results as JSON to stdout
#[test]
fn test_reporter__json_output__must_support_json_flag_that_outputs_structured_results_as_json_to() {
    use ought_report::json;
    use ought_run::{RunResult, TestDetails, TestResult, TestStatus};
    use ought_spec::{Clause, ClauseId, Keyword, Section, Severity, SourceLocation, Spec};
    use serde_json::Value;
    use std::path::PathBuf;
    use std::time::Duration;

    let clause_id = ClauseId("api::endpoints::must_return_200".to_string());

    let clause = Clause {
        id: clause_id.clone(),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: "return HTTP 200 for valid requests".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation {
            file: PathBuf::from("api.ought.md"),
            line: 4,
        },
        content_hash: "abc123".to_string(),
    };

    let spec = Spec {
        name: "API Spec".to_string(),
        metadata: Default::default(),
        sections: vec![Section {
            title: "Endpoints".to_string(),
            depth: 1,
            prose: String::new(),
            clauses: vec![clause],
            subsections: vec![],
        }],
        source_path: PathBuf::from("api.ought.md"),
    };

    let run_result = RunResult {
        results: vec![TestResult {
            clause_id: clause_id.clone(),
            status: TestStatus::Passed,
            message: None,
            duration: Duration::from_millis(8),
            details: TestDetails::default(),
        }],
        total_duration: Duration::from_millis(8),
    };

    // Invoking the JSON reporter (as triggered by --json) must return a JSON-formatted string.
    let output = json::report(&run_result, &[spec])
        .expect("json::report must not fail");

    assert!(!output.is_empty(), "JSON output must not be empty");

    // Output must be parseable as valid JSON — not human-readable text.
    let parsed: Value = serde_json::from_str(&output)
        .expect("--json output must be valid JSON");

    assert!(
        parsed.is_object(),
        "--json output must be a JSON object at the top level; got: {output}"
    );

    // Top-level report object must carry a specs array, a summary object, and a duration.
    assert!(
        parsed.get("specs").map_or(false, |v| v.is_array()),
        "JSON report must contain a 'specs' array"
    );
    assert!(
        parsed.get("summary").map_or(false, |v| v.is_object()),
        "JSON report must contain a 'summary' object"
    );
    assert!(
        parsed.get("total_duration_ms").map_or(false, |v| v.is_number()),
        "JSON report must contain a 'total_duration_ms' number"
    );
}