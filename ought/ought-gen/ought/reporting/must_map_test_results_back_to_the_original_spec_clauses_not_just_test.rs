/// MUST map test results back to the original spec clauses (not just test function names)
#[test]
fn test_ought__reporting__must_map_test_results_back_to_the_original_spec_clauses_not_just() {
    use std::path::PathBuf;
    use std::time::Duration;
    use ought_report::json;
    use ought_run::{RunResult, TestDetails, TestResult, TestStatus};
    use ought_spec::{Clause, ClauseId, Keyword, Metadata, Section, SourceLocation, Spec};

    let clause_id = "auth::login::must_return_jwt_on_success";
    let clause = Clause {
        id: ClauseId(clause_id.to_string()),
        keyword: Keyword::Must,
        severity: Keyword::Must.severity(),
        text: "return a signed JWT on successful login".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("auth.ought.md"), line: 5 },
        content_hash: "abc".to_string(),
    };
    let spec = Spec {
        name: "Auth".to_string(),
        metadata: Metadata::default(),
        sections: vec![Section {
            title: "Login".to_string(),
            depth: 1,
            prose: String::new(),
            clauses: vec![clause],
            subsections: vec![],
        }],
        source_path: PathBuf::from("auth.ought.md"),
    };
    let result = TestResult {
        clause_id: ClauseId(clause_id.to_string()),
        status: TestStatus::Passed,
        message: None,
        duration: Duration::from_millis(12),
        details: TestDetails::default(),
    };
    let run = RunResult { results: vec![result], total_duration: Duration::from_millis(12) };

    let json_out = json::report(&run, &[spec]).unwrap();

    // The report must include the original hierarchical clause ID, not a derived test function name.
    assert!(
        json_out.contains(clause_id),
        "JSON report must embed the original spec clause ID '{}'; got:\n{}",
        clause_id,
        json_out
    );
    // Verify the field name is explicitly "clause_id" — not "test_name" or "function_name".
    assert!(
        json_out.contains("\"clause_id\""),
        "JSON report must have an explicit 'clause_id' field rather than a derived test name"
    );
    // The reported clause_id must not be mangled into a snake_case test function name.
    assert!(
        !json_out.contains("test_auth_login_must_return_jwt_on_success"),
        "report must use the spec clause ID, not a generated test function name"
    );
}