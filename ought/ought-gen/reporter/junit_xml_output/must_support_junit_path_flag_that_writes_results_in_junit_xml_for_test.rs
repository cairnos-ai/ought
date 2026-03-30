/// MUST support `--junit <path>` flag that writes results in JUnit XML format
#[test]
fn test_reporter__junit_xml_output__must_support_junit_path_flag_that_writes_results_in_junit_xml_for() {
    use ought_report::junit;
    use ought_run::{RunResult, TestDetails, TestResult, TestStatus};
    use ought_spec::{Clause, ClauseId, Keyword, Section, Severity, SourceLocation, Spec};
    use std::path::PathBuf;
    use std::time::Duration;

    let clause_id = ClauseId("auth::login::must_return_jwt".to_string());

    let clause = Clause {
        id: clause_id.clone(),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: "return a JWT on successful login".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation {
            file: PathBuf::from("auth.ought.md"),
            line: 3,
        },
        content_hash: "deadbeef".to_string(),
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

    let run_result = RunResult {
        results: vec![TestResult {
            clause_id: clause_id.clone(),
            status: TestStatus::Passed,
            message: None,
            duration: Duration::from_millis(12),
            details: TestDetails::default(),
        }],
        total_duration: Duration::from_millis(12),
    };

    let dir = std::env::temp_dir();
    let out_path = dir.join("ought_test_junit_flag.xml");
    let _ = std::fs::remove_file(&out_path);

    // Invoking junit::report (as triggered by --junit <path>) must succeed and produce a file.
    junit::report(&run_result, &[spec], &out_path)
        .expect("junit::report must not fail");

    assert!(
        out_path.exists(),
        "--junit flag must produce a file at the given path"
    );

    let contents = std::fs::read_to_string(&out_path)
        .expect("must be able to read the written JUnit XML file");

    assert!(
        !contents.is_empty(),
        "JUnit XML file must not be empty"
    );

    // File must be well-formed XML starting with the XML declaration.
    assert!(
        contents.starts_with("<?xml"),
        "JUnit XML file must start with an XML declaration; got: {contents}"
    );

    assert!(
        contents.contains("<testsuites>"),
        "JUnit XML file must contain a <testsuites> root element"
    );

    let _ = std::fs::remove_file(&out_path);
}