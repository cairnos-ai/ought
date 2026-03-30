/// SHOULD support LLM-powered failure diagnosis that explains why a test failed in terms of the source code
#[test]
fn test_ought__reporting__should_support_llm_powered_failure_diagnosis_that_explains_why_a_te() {
    use std::path::PathBuf;
    use std::time::Duration;
    use ought_report::diagnosis::diagnose;
    use ought_report::types::{Diagnosis, SuggestedFix};
    use ought_run::{RunResult, TestDetails, TestResult, TestStatus};
    use ought_spec::{Clause, ClauseId, Keyword, Metadata, Section, SourceLocation, Spec};

    let clause_id = "payment::checkout::must_charge_correct_amount";
    let spec = Spec {
        name: "Payment".to_string(),
        metadata: Metadata::default(),
        sections: vec![Section {
            title: "Checkout".to_string(),
            depth: 1,
            prose: String::new(),
            clauses: vec![Clause {
                id: ClauseId(clause_id.to_string()),
                keyword: Keyword::Must,
                severity: Keyword::Must.severity(),
                text: "charge the exact order total".to_string(),
                condition: None,
                otherwise: vec![],
                temporal: None,
                hints: vec![],
                source_location: SourceLocation { file: PathBuf::from("payment.ought.md"), line: 10 },
                content_hash: "ff".to_string(),
            }],
            subsections: vec![],
        }],
        source_path: PathBuf::from("payment.ought.md"),
    };
    let run = RunResult {
        results: vec![TestResult {
            clause_id: ClauseId(clause_id.to_string()),
            status: TestStatus::Failed,
            message: Some("expected 4999, got 4900".to_string()),
            duration: Duration::from_millis(5),
            details: TestDetails {
                failure_message: Some("assertion `left == right` failed\n  left: 4999\n right: 4900".to_string()),
                ..Default::default()
            },
        }],
        total_duration: Duration::from_millis(5),
    };

    // diagnose() must accept a RunResult and spec slice and return Ok(Vec<Diagnosis>).
    let result = diagnose(&run, &[spec]);
    assert!(result.is_ok(), "diagnose() must not return Err: {:?}", result.err());

    let diagnoses = result.unwrap();
    // Every returned Diagnosis must carry a non-empty clause_id for mapping back to the spec.
    for d in &diagnoses {
        assert!(!d.clause_id.0.is_empty(),
            "each Diagnosis must have a non-empty clause_id");
        assert!(!d.explanation.is_empty(),
            "each Diagnosis must have a non-empty explanation string");
    }

    // The Diagnosis struct must support an optional SuggestedFix with file, line, and description
    // so the reporter can show actionable guidance alongside the failing clause.
    let example = Diagnosis {
        clause_id: ClauseId(clause_id.to_string()),
        explanation: "charge() applies a discount twice; the net total is 2% too low.".to_string(),
        suggested_fix: Some(SuggestedFix {
            file: PathBuf::from("src/checkout.rs"),
            line: 87,
            description: "Remove the second call to apply_loyalty_discount() on line 87.".to_string(),
        }),
    };
    assert_eq!(example.clause_id.0, clause_id);
    assert!(!example.explanation.is_empty());
    let fix = example.suggested_fix.as_ref().unwrap();
    assert_eq!(fix.line, 87);
    assert!(!fix.description.is_empty());
    assert!(!fix.file.as_os_str().is_empty());
}