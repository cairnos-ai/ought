/// SHOULD show the original clause text alongside the failure for easy comparison
#[test]
fn test_reporter__failure_details__should_show_the_original_clause_text_alongside_the_failure_for_easy() {
    use ought_report::types::{ColorChoice, ReportOptions};
    use ought_run::{RunResult, TestDetails, TestResult, TestStatus};
    use ought_spec::{Clause, ClauseId, Keyword, Section, Severity, Spec, SourceLocation};
    use std::path::PathBuf;
    use std::time::Duration;

    let clause_id = ClauseId("billing::must_charge_correct_amount".to_string());
    let clause_text = "charge the correct amount in the user's currency";
    let failure_msg = "assertion failed: invoice.amount_cents == 999\n  left: 1099\n right: 999";

    let clause = Clause {
        id: clause_id.clone(),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: clause_text.to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("billing.md"), line: 15 },
        content_hash: "jkl012".to_string(),
    };

    let section = Section {
        title: "Billing".to_string(),
        depth: 1,
        prose: String::new(),
        clauses: vec![clause],
        subsections: vec![],
    };

    let spec = Spec {
        name: "Billing Spec".to_string(),
        metadata: Default::default(),
        sections: vec![section],
        source_path: PathBuf::from("billing.md"),
    };

    let run_result = RunResult {
        results: vec![TestResult {
            clause_id: clause_id.clone(),
            status: TestStatus::Failed,
            message: None,
            duration: Duration::from_millis(6),
            details: TestDetails {
                failure_message: Some(failure_msg.to_string()),
                ..Default::default()
            },
        }],
        total_duration: Duration::from_millis(6),
    };

    let options = ReportOptions {
        color: ColorChoice::Never,
        ..Default::default()
    };

    let mut output = Vec::new();
    ought_report::terminal::render_to_writer(&mut output, &run_result, &[spec], &options)
        .expect("render_to_writer should succeed");
    let text = String::from_utf8(output).expect("output should be valid UTF-8");

    // The original clause text must be visible in the output so the reader can see
    // exactly which obligation failed.
    assert!(
        text.contains(clause_text),
        "output must show the original clause text alongside the failure; got:\n{text}"
    );
    // The failure message must also be present.
    assert!(
        text.contains("assertion failed"),
        "output must show the failure message alongside the clause text; got:\n{text}"
    );
    // Clause text must appear before the failure detail — the clause line comes first,
    // then the indented error beneath it.
    let clause_pos = text
        .lines()
        .position(|l| l.contains(clause_text))
        .expect("clause text must appear in output");
    let failure_pos = text
        .lines()
        .position(|l| l.contains("assertion failed"))
        .expect("failure message must appear in output");
    assert!(
        clause_pos < failure_pos,
        "clause text must appear before (above) the failure detail in the output for easy comparison"
    );
    // They must be within a few lines of each other — not separated by unrelated content.
    let proximity = failure_pos - clause_pos;
    assert!(
        proximity <= 5,
        "clause text and failure detail must be visually proximate (within 5 lines), but they are {} lines apart",
        proximity
    );
}