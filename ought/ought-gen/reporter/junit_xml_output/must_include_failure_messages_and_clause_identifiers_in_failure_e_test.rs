/// MUST include failure messages and clause identifiers in `<failure>` elements
#[test]
fn test_reporter__junit_xml_output__must_include_failure_messages_and_clause_identifiers_in_failure_e() {
    use ought_report::junit;
    use ought_run::{RunResult, TestDetails, TestResult, TestStatus};
    use ought_spec::{Clause, ClauseId, Keyword, Section, Severity, SourceLocation, Spec};
    use std::path::PathBuf;
    use std::time::Duration;

    let clause_id = ClauseId("payment::checkout::must_charge_correct_amount".to_string());

    let clause = Clause {
        id: clause_id.clone(),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: "charge the correct amount".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation {
            file: PathBuf::from("payment.ought.md"),
            line: 8,
        },
        content_hash: "ff00".to_string(),
    };

    let spec = Spec {
        name: "Payment Spec".to_string(),
        metadata: Default::default(),
        sections: vec![Section {
            title: "Checkout".to_string(),
            depth: 1,
            prose: String::new(),
            clauses: vec![clause],
            subsections: vec![],
        }],
        source_path: PathBuf::from("payment.ought.md"),
    };

    let failure_msg = "expected charge 42.00 but got 0.00";

    let run_result = RunResult {
        results: vec![TestResult {
            clause_id: clause_id.clone(),
            status: TestStatus::Failed,
            message: Some(failure_msg.to_string()),
            duration: Duration::from_millis(3),
            details: TestDetails {
                failure_message: Some(failure_msg.to_string()),
                ..Default::default()
            },
        }],
        total_duration: Duration::from_millis(3),
    };

    let out_path = std::env::temp_dir().join("ought_test_failure_elem.xml");
    let _ = std::fs::remove_file(&out_path);

    junit::report(&run_result, &[spec], &out_path)
        .expect("junit::report must not fail");

    let xml = std::fs::read_to_string(&out_path).unwrap();

    // A <failure> element must be emitted for the failed testcase.
    assert!(
        xml.contains("<failure "),
        "a <failure> element must be present for a failed clause; xml:\n{xml}"
    );

    // The failure message must appear in the element.
    assert!(
        xml.contains("expected charge 42.00 but got 0.00"),
        "failure message must be included in <failure> element"
    );

    // The clause identifier must appear as a property within the testcase.
    assert!(
        xml.contains("payment::checkout::must_charge_correct_amount"),
        "clause identifier must appear in the output for the failing testcase"
    );

    let _ = std::fs::remove_file(&out_path);
}