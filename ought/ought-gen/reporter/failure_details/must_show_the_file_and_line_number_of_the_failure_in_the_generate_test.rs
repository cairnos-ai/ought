/// MUST show the file and line number of the failure in the generated test
#[test]
fn test_reporter__failure_details__must_show_the_file_and_line_number_of_the_failure_in_the_generate() {
    use ought_report::types::{ColorChoice, ReportOptions};
    use ought_run::{RunResult, TestDetails, TestResult, TestStatus};
    use ought_spec::{Clause, ClauseId, Keyword, Section, Severity, Spec, SourceLocation};
    use std::path::PathBuf;
    use std::time::Duration;

    let clause_id = ClauseId("api::response::must_return_json_content_type".to_string());

    // Rust test panics include the source file and line in the failure message.
    // ought-run captures this as part of failure_message so the reporter can display it.
    let failure_msg =
        "thread 'test_reporter__api__response' panicked at tests/generated/api_response.rs:57:5:\nassertion failed: response.headers()[\"content-type\"] == \"application/json\"";

    let clause = Clause {
        id: clause_id.clone(),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: "return JSON content-type header".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("api.md"), line: 22 },
        content_hash: "def456".to_string(),
    };

    let section = Section {
        title: "API Response".to_string(),
        depth: 1,
        prose: String::new(),
        clauses: vec![clause],
        subsections: vec![],
    };

    let spec = Spec {
        name: "API Spec".to_string(),
        metadata: Default::default(),
        sections: vec![section],
        source_path: PathBuf::from("api.md"),
    };

    let run_result = RunResult {
        results: vec![TestResult {
            clause_id: clause_id.clone(),
            status: TestStatus::Failed,
            message: None,
            duration: Duration::from_millis(8),
            details: TestDetails {
                failure_message: Some(failure_msg.to_string()),
                ..Default::default()
            },
        }],
        total_duration: Duration::from_millis(8),
    };

    let options = ReportOptions {
        color: ColorChoice::Never,
        ..Default::default()
    };

    let mut output = Vec::new();
    ought_report::terminal::render_to_writer(&mut output, &run_result, &[spec], &options)
        .expect("render_to_writer should succeed");
    let text = String::from_utf8(output).expect("output should be valid UTF-8");

    assert!(
        text.contains("api_response.rs"),
        "output must show the generated test file name where the failure occurred; got:\n{text}"
    );
    assert!(
        text.contains("57"),
        "output must show the line number of the failure in the generated test; got:\n{text}"
    );
    // Both must appear together — check the same line or adjacent lines contain the path:line pattern
    let has_file_and_line = text
        .lines()
        .any(|l| l.contains("api_response.rs") && l.contains("57"));
    assert!(
        has_file_and_line,
        "the file path and line number must appear together (e.g. 'api_response.rs:57') on the same output line; got:\n{text}"
    );
}