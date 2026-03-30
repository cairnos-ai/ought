/// MUST produce visually attractive terminal output that makes specs and their status easy to scan
#[test]
fn test_ought__reporting__must_produce_visually_attractive_terminal_output_that_makes_specs() {
    use std::path::PathBuf;
    use std::time::Duration;
    use ought_report::json;
    use ought_report::terminal;
    use ought_report::types::{ColorChoice, ReportOptions};
    use ought_run::{RunResult, TestDetails, TestResult, TestStatus};
    use ought_spec::{Clause, ClauseId, Keyword, Metadata, Section, SourceLocation, Spec};

    fn make_clause(id: &str, kw: Keyword, text: &str) -> Clause {
        Clause {
            id: ClauseId(id.to_string()),
            keyword: kw,
            severity: kw.severity(),
            text: text.to_string(),
            condition: None,
            otherwise: vec![],
            temporal: None,
            hints: vec![],
            source_location: SourceLocation { file: PathBuf::from("t.ought.md"), line: 1 },
            content_hash: "x".to_string(),
        }
    }

    let passed_id = "terminal::output::must_return_200";
    let failed_id = "terminal::output::should_include_request_id";
    let errored_id = "terminal::output::must_not_leak_secrets";

    let spec = Spec {
        name: "Terminal Display".to_string(),
        metadata: Metadata::default(),
        sections: vec![Section {
            title: "HTTP API".to_string(),
            depth: 1,
            prose: String::new(),
            clauses: vec![
                make_clause(passed_id, Keyword::Must, "return 200 on success"),
                make_clause(failed_id, Keyword::Should, "include X-Request-Id header"),
                make_clause(errored_id, Keyword::MustNot, "leak secrets in response body"),
            ],
            subsections: vec![],
        }],
        source_path: PathBuf::from("t.ought.md"),
    };
    let run = RunResult {
        results: vec![
            TestResult {
                clause_id: ClauseId(passed_id.to_string()),
                status: TestStatus::Passed,
                message: None,
                duration: Duration::from_millis(8),
                details: TestDetails::default(),
            },
            TestResult {
                clause_id: ClauseId(failed_id.to_string()),
                status: TestStatus::Failed,
                message: Some("header absent".to_string()),
                duration: Duration::from_millis(3),
                details: TestDetails { failure_message: Some("header absent".to_string()), ..Default::default() },
            },
            TestResult {
                clause_id: ClauseId(errored_id.to_string()),
                status: TestStatus::Errored,
                message: Some("panicked at 'index out of bounds'".to_string()),
                duration: Duration::from_millis(1),
                details: TestDetails::default(),
            },
        ],
        total_duration: Duration::from_millis(12),
    };

    // Terminal renderer must not return an error on well-formed input.
    let options = ReportOptions { color: ColorChoice::Never, ..Default::default() };
    assert!(
        terminal::report(&run, &[spec.clone()], &options).is_ok(),
        "terminal::report must complete without error on valid input"
    );

    // JSON reporter surfaces the same data — verify every field required for visual display.
    let json_out = json::report(&run, &[spec]).unwrap();
    assert!(json_out.contains("\"clause_id\""),
        "output must carry clause_id so each line maps to a spec clause");
    assert!(json_out.contains("\"keyword\""),
        "output must carry keyword (MUST/SHOULD/…) for visual label");
    assert!(json_out.contains("\"severity\""),
        "output must carry severity to drive color-coding");
    assert!(json_out.contains("\"status\""),
        "output must carry status for pass/fail/error indicator");
    // Status icons: passed → ✓, failed → ✗, errored → !
    assert!(json_out.contains("\"passed\""),
        "passed clause must be represented in output");
    assert!(json_out.contains("\"failed\""),
        "failed clause must be represented in output");
    assert!(json_out.contains("\"errored\""),
        "errored clause must be represented in output");
    // Summary must include MUST coverage percentage for at-a-glance health.
    assert!(json_out.contains("must_coverage_pct"),
        "summary must include MUST coverage percentage");
    // With one MUST passed and one MUST errored, coverage must be < 100 %.
    let parsed: serde_json::Value = serde_json::from_str(&json_out).unwrap();
    let pct = parsed["summary"]["must_coverage_pct"].as_f64().unwrap();
    assert!(pct < 100.0, "MUST coverage must be < 100% when a MUST clause errored");
}