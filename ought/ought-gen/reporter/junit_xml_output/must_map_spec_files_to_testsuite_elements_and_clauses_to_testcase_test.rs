/// MUST map spec files to `<testsuite>` elements and clauses to `<testcase>` elements
#[test]
fn test_reporter__junit_xml_output__must_map_spec_files_to_testsuite_elements_and_clauses_to_testcase() {
    use ought_report::junit;
    use ought_run::{RunResult, TestDetails, TestResult, TestStatus};
    use ought_spec::{Clause, ClauseId, Keyword, Section, Severity, SourceLocation, Spec};
    use std::path::PathBuf;
    use std::time::Duration;

    let make_spec = |name: &str, clause_id_str: &str| {
        let clause_id = ClauseId(clause_id_str.to_string());
        let clause = Clause {
            id: clause_id.clone(),
            keyword: Keyword::Must,
            severity: Severity::Required,
            text: "behave correctly".to_string(),
            condition: None,
            otherwise: vec![],
            temporal: None,
            hints: vec![],
            source_location: SourceLocation {
                file: PathBuf::from("spec.ought.md"),
                line: 1,
            },
            content_hash: "abc".to_string(),
        };
        (
            clause_id,
            Spec {
                name: name.to_string(),
                metadata: Default::default(),
                sections: vec![Section {
                    title: "Section".to_string(),
                    depth: 1,
                    prose: String::new(),
                    clauses: vec![clause],
                    subsections: vec![],
                }],
                source_path: PathBuf::from("spec.ought.md"),
            },
        )
    };

    let (id_a, spec_a) = make_spec("Spec Alpha", "alpha::section::must_behave_correctly");
    let (id_b, spec_b) = make_spec("Spec Beta", "beta::section::must_behave_correctly");

    let run_result = RunResult {
        results: vec![
            TestResult {
                clause_id: id_a,
                status: TestStatus::Passed,
                message: None,
                duration: Duration::from_millis(5),
                details: TestDetails::default(),
            },
            TestResult {
                clause_id: id_b,
                status: TestStatus::Passed,
                message: None,
                duration: Duration::from_millis(7),
                details: TestDetails::default(),
            },
        ],
        total_duration: Duration::from_millis(12),
    };

    let out_path = std::env::temp_dir().join("ought_test_mapping.xml");
    let _ = std::fs::remove_file(&out_path);

    junit::report(&run_result, &[spec_a, spec_b], &out_path)
        .expect("junit::report must not fail");

    let xml = std::fs::read_to_string(&out_path).unwrap();

    // Each spec file must produce exactly one <testsuite> with the spec name.
    let suite_count = xml.matches("<testsuite ").count();
    assert_eq!(suite_count, 2, "one <testsuite> per spec file; found {suite_count}");

    assert!(
        xml.contains("name=\"Spec Alpha\""),
        "spec name must appear as testsuite name attribute"
    );
    assert!(
        xml.contains("name=\"Spec Beta\""),
        "second spec name must appear as testsuite name attribute"
    );

    // Each clause must produce a <testcase> element.
    let case_count = xml.matches("<testcase ").count();
    assert_eq!(case_count, 2, "one <testcase> per clause; found {case_count}");

    // testcase must carry a classname matching its spec.
    assert!(
        xml.contains("classname=\"Spec Alpha\""),
        "<testcase> must have classname equal to the spec name"
    );

    let _ = std::fs::remove_file(&out_path);
}