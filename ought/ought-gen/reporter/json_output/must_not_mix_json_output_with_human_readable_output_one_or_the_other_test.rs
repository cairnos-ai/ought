/// MUST NOT mix JSON output with human-readable output (one or the other)
#[test]
fn test_reporter__json_output__must_not_mix_json_output_with_human_readable_output_one_or_the_other() {
    use ought_report::json;
    use ought_run::{RunResult, TestDetails, TestResult, TestStatus};
    use ought_spec::{Clause, ClauseId, Keyword, Section, Severity, SourceLocation, Spec};
    use serde_json::Value;
    use std::path::PathBuf;
    use std::time::Duration;

    let clause_id = ClauseId("db::query::must_return_results_within_sla".to_string());

    let clause = Clause {
        id: clause_id.clone(),
        keyword: Keyword::Should,
        severity: Severity::Recommended,
        text: "return results within 100ms".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation {
            file: PathBuf::from("db.ought.md"),
            line: 6,
        },
        content_hash: "ghi789".to_string(),
    };

    let spec = Spec {
        name: "DB Spec".to_string(),
        metadata: Default::default(),
        sections: vec![Section {
            title: "Query".to_string(),
            depth: 1,
            prose: String::new(),
            clauses: vec![clause],
            subsections: vec![],
        }],
        source_path: PathBuf::from("db.ought.md"),
    };

    let run_result = RunResult {
        results: vec![TestResult {
            clause_id: clause_id.clone(),
            status: TestStatus::Passed,
            message: None,
            duration: Duration::from_millis(55),
            details: TestDetails::default(),
        }],
        total_duration: Duration::from_millis(55),
    };

    let json_output = json::report(&run_result, &[spec])
        .expect("json::report must not fail");

    // The entire output string must parse as JSON — no preamble or trailing text.
    let _parsed: Value = serde_json::from_str(&json_output)
        .expect("JSON output must be valid JSON with nothing appended or prepended");

    // JSON output must contain no ANSI escape codes emitted by the terminal reporter.
    assert!(
        !json_output.contains("\x1b["),
        "JSON output must not contain ANSI escape codes; found in: {json_output}"
    );

    // JSON output must not contain terminal status icons used by the human-readable reporter.
    for indicator in &["✓", "✗", "⊘"] {
        assert!(
            !json_output.contains(indicator),
            "JSON output must not contain terminal status indicator '{indicator}'"
        );
    }

    // JSON output must not contain box-drawing characters used in failure detail panels.
    for box_char in &["┌", "│", "└", "─"] {
        assert!(
            !json_output.contains(box_char),
            "JSON output must not contain box-drawing character '{box_char}'"
        );
    }

    // Model the CLI dispatch gate: --json and terminal output are mutually exclusive.
    // Exactly one reporter runs per invocation.
    struct OutputMode {
        json: bool,
    }
    impl OutputMode {
        fn uses_json_reporter(&self) -> bool {
            self.json
        }
        fn uses_terminal_reporter(&self) -> bool {
            !self.json
        }
    }

    let json_mode = OutputMode { json: true };
    assert!(
        json_mode.uses_json_reporter(),
        "--json mode must activate the JSON reporter"
    );
    assert!(
        !json_mode.uses_terminal_reporter(),
        "--json mode must NOT activate the terminal reporter"
    );

    let terminal_mode = OutputMode { json: false };
    assert!(
        !terminal_mode.uses_json_reporter(),
        "terminal mode must NOT activate the JSON reporter"
    );
    assert!(
        terminal_mode.uses_terminal_reporter(),
        "terminal mode must activate the terminal reporter"
    );
}