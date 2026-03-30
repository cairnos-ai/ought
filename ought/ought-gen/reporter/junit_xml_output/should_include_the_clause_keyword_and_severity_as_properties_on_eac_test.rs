/// SHOULD include the clause keyword and severity as properties on each `<testcase>`
#[test]
fn test_reporter__junit_xml_output__should_include_the_clause_keyword_and_severity_as_properties_on_eac() {
    use ought_report::junit;
    use ought_run::{RunResult, TestDetails, TestResult, TestStatus};
    use ought_spec::{Clause, ClauseId, Keyword, Section, Severity, SourceLocation, Spec};
    use std::path::PathBuf;
    use std::time::Duration;

    let make_clause_and_result = |id: &str, kw: Keyword| {
        let clause_id = ClauseId(id.to_string());
        let clause = Clause {
            id: clause_id.clone(),
            keyword: kw,
            severity: kw.severity(),
            text: "satisfy this clause".to_string(),
            condition: None,
            otherwise: vec![],
            temporal: None,
            hints: vec![],
            source_location: SourceLocation {
                file: PathBuf::from("spec.ought.md"),
                line: 1,
            },
            content_hash: "00".to_string(),
        };
        let result = TestResult {
            clause_id: clause_id.clone(),
            status: TestStatus::Passed,
            message: None,
            duration: Duration::from_millis(1),
            details: TestDetails::default(),
        };
        (clause, result)
    };

    let (must_clause, must_result) =
        make_clause_and_result("props::section::must_satisfy", Keyword::Must);
    let (should_clause, should_result) =
        make_clause_and_result("props::section::should_satisfy", Keyword::Should);
    let (may_clause, may_result) =
        make_clause_and_result("props::section::may_satisfy", Keyword::May);

    let spec = Spec {
        name: "Properties Spec".to_string(),
        metadata: Default::default(),
        sections: vec![Section {
            title: "Section".to_string(),
            depth: 1,
            prose: String::new(),
            clauses: vec![must_clause, should_clause, may_clause],
            subsections: vec![],
        }],
        source_path: PathBuf::from("spec.ought.md"),
    };

    let run_result = RunResult {
        results: vec![must_result, should_result, may_result],
        total_duration: Duration::from_millis(3),
    };

    let out_path = std::env::temp_dir().join("ought_test_properties.xml");
    let _ = std::fs::remove_file(&out_path);

    junit::report(&run_result, &[spec], &out_path)
        .expect("junit::report must not fail");

    let xml = std::fs::read_to_string(&out_path).unwrap();

    // Each testcase must have a <properties> block.
    assert!(
        xml.contains("<properties>"),
        "<properties> element must be present on testcases"
    );

    // Keyword property must reflect the deontic operator.
    assert!(
        xml.contains("name=\"keyword\" value=\"MUST\""),
        "keyword property must be MUST for a Must clause"
    );
    assert!(
        xml.contains("name=\"keyword\" value=\"SHOULD\""),
        "keyword property must be SHOULD for a Should clause"
    );
    assert!(
        xml.contains("name=\"keyword\" value=\"MAY\""),
        "keyword property must be MAY for a May clause"
    );

    // Severity property must reflect the keyword's severity.
    assert!(
        xml.contains("name=\"severity\" value=\"required\""),
        "severity must be 'required' for MUST clauses"
    );
    assert!(
        xml.contains("name=\"severity\" value=\"recommended\""),
        "severity must be 'recommended' for SHOULD clauses"
    );
    assert!(
        xml.contains("name=\"severity\" value=\"optional\""),
        "severity must be 'optional' for MAY clauses"
    );

    // clause_id property must be present.
    assert!(
        xml.contains("name=\"clause_id\""),
        "clause_id property must be present on each testcase"
    );

    let _ = std::fs::remove_file(&out_path);
}