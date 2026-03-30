/// MUST distinguish failure severity — MUST failures are errors, SHOULD failures are warnings
#[test]
fn test_ought__reporting__must_distinguish_failure_severity_must_failures_are_errors_should() {
    use std::path::PathBuf;
    use std::time::Duration;
    use ought_report::json;
    use ought_run::{RunResult, TestDetails, TestResult, TestStatus};
    use ought_spec::{Clause, ClauseId, Keyword, Metadata, Section, Severity, SourceLocation, Spec};

    // Keyword → Severity mapping must be stable and correct.
    assert_eq!(Keyword::Must.severity(), Severity::Required,
        "MUST keyword must map to Required severity");
    assert_eq!(Keyword::MustNot.severity(), Severity::Required,
        "MUST NOT keyword must map to Required severity");
    assert_eq!(Keyword::MustAlways.severity(), Severity::Required,
        "MUST ALWAYS keyword must map to Required severity");
    assert_eq!(Keyword::MustBy.severity(), Severity::Required,
        "MUST BY keyword must map to Required severity");
    assert_eq!(Keyword::Should.severity(), Severity::Recommended,
        "SHOULD keyword must map to Recommended severity");
    assert_eq!(Keyword::ShouldNot.severity(), Severity::Recommended,
        "SHOULD NOT keyword must map to Recommended severity");

    // Required > Recommended in severity ordering (MUST failures are more severe).
    assert!(Severity::Required > Severity::Recommended,
        "Required (MUST) severity must rank higher than Recommended (SHOULD)");

    // JSON report must expose "required" for MUST failures and "recommended" for SHOULD failures.
    let must_id = "report::severity::must_clause";
    let should_id = "report::severity::should_clause";

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
            source_location: SourceLocation { file: PathBuf::from("s.ought.md"), line: 1 },
            content_hash: "x".to_string(),
        }
    }

    let spec = Spec {
        name: "Severity Test".to_string(),
        metadata: Metadata::default(),
        sections: vec![Section {
            title: "Clauses".to_string(),
            depth: 1,
            prose: String::new(),
            clauses: vec![
                make_clause(must_id, Keyword::Must, "always succeed"),
                make_clause(should_id, Keyword::Should, "prefer to succeed"),
            ],
            subsections: vec![],
        }],
        source_path: PathBuf::from("s.ought.md"),
    };
    let run = RunResult {
        results: vec![
            TestResult {
                clause_id: ClauseId(must_id.to_string()),
                status: TestStatus::Failed,
                message: None,
                duration: Duration::from_millis(1),
                details: TestDetails::default(),
            },
            TestResult {
                clause_id: ClauseId(should_id.to_string()),
                status: TestStatus::Failed,
                message: None,
                duration: Duration::from_millis(1),
                details: TestDetails::default(),
            },
        ],
        total_duration: Duration::from_millis(2),
    };

    let json_out = json::report(&run, &[spec]).unwrap();

    // Both severity labels must appear in the report to distinguish the two failure classes.
    assert!(json_out.contains("\"required\""),
        "Failed MUST clause must appear with severity 'required' in JSON output");
    assert!(json_out.contains("\"recommended\""),
        "Failed SHOULD clause must appear with severity 'recommended' in JSON output");
    // Sanity: both show as "failed".
    assert_eq!(json_out.matches("\"failed\"").count(), 2,
        "Both clauses should appear with status 'failed'");
}