/// MAY be combined with other output modes (e.g. human-readable to terminal + JUnit XML to file)
#[test]
fn test_reporter__junit_xml_output__may_be_combined_with_other_output_modes_e_g_human_readable_to_te() {
    use ought_report::junit;
    use ought_report::terminal;
    use ought_run::{RunResult, TestDetails, TestResult, TestStatus};
    use ought_spec::{Clause, ClauseId, Keyword, Section, Severity, SourceLocation, Spec};
    use std::path::PathBuf;
    use std::time::Duration;

    // Build a single spec/result that will be fed to both reporters.
    let clause_id = ClauseId("combined::section::must_work".to_string());

    let clause = Clause {
        id: clause_id.clone(),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: "work when multiple reporters are active".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation {
            file: PathBuf::from("combined.ought.md"),
            line: 2,
        },
        content_hash: "comb01".to_string(),
    };

    let spec = Spec {
        name: "Combined Spec".to_string(),
        metadata: Default::default(),
        sections: vec![Section {
            title: "Section".to_string(),
            depth: 1,
            prose: String::new(),
            clauses: vec![clause],
            subsections: vec![],
        }],
        source_path: PathBuf::from("combined.ought.md"),
    };

    let run_result = RunResult {
        results: vec![TestResult {
            clause_id: clause_id.clone(),
            status: TestStatus::Passed,
            message: None,
            duration: Duration::from_millis(4),
            details: TestDetails::default(),
        }],
        total_duration: Duration::from_millis(4),
    };

    let out_path = std::env::temp_dir().join("ought_test_combined.xml");
    let _ = std::fs::remove_file(&out_path);

    // JUnit XML reporter writes to file.
    junit::report(&run_result, &[spec.clone()], &out_path)
        .expect("junit::report must succeed when combined with other output modes");

    // Terminal reporter writes to a buffer (simulating stdout) independently.
    let opts = ought_report::ReportOptions {
        diagnose: false,
        grade: false,
        quiet: false,
        color: ought_report::ColorChoice::Never,
    };
    let mut buf: Vec<u8> = Vec::new();
    terminal::report(&run_result, &[spec], &opts, &mut buf)
        .expect("terminal::report must succeed when combined with JUnit output");

    // Both outputs must have been produced without interfering with each other.
    assert!(
        out_path.exists(),
        "JUnit XML file must exist after combined reporting"
    );

    let xml = std::fs::read_to_string(&out_path).unwrap();
    assert!(
        xml.contains("<testsuites>"),
        "JUnit XML must still be well-formed when combined with terminal output"
    );

    assert!(
        !buf.is_empty(),
        "terminal output must not be empty when combined with JUnit output"
    );

    let _ = std::fs::remove_file(&out_path);
}