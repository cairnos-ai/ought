/// MUST display MUST BY results with the measured duration alongside the deadline
#[test]
fn test_reporter__temporal_result_display__must_display_must_by_results_with_the_measured_duration_alongside() {
    use ought_report::types::{ColorChoice, ReportOptions};
    use ought_run::{RunResult, TestResult, TestStatus, TestDetails};
    use ought_spec::{Clause, ClauseId, Keyword, Section, Severity, Spec, SourceLocation, Temporal};
    use std::path::PathBuf;
    use std::time::Duration;

    let clause_id = ClauseId("perf::must_by_200ms_return_a_response".to_string());

    let deadline = Duration::from_millis(200);
    let measured = Duration::from_millis(47);

    let clause = Clause {
        id: clause_id.clone(),
        keyword: Keyword::MustBy,
        severity: Severity::Required,
        text: "return a response".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: Some(Temporal::Deadline(deadline)),
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("spec.md"), line: 5 },
        content_hash: "def".to_string(),
    };

    let section = Section {
        title: "Performance".to_string(),
        depth: 1,
        prose: String::new(),
        clauses: vec![clause],
        subsections: vec![],
    };

    let spec = Spec {
        name: "Perf Spec".to_string(),
        metadata: Default::default(),
        sections: vec![section],
        source_path: PathBuf::from("spec.md"),
    };

    let run_result = RunResult {
        results: vec![TestResult {
            clause_id: clause_id.clone(),
            status: TestStatus::Passed,
            message: None,
            duration: measured,
            details: TestDetails {
                measured_duration: Some(measured),
                ..Default::default()
            },
        }],
        total_duration: measured,
    };

    let options = ReportOptions {
        color: ColorChoice::Never,
        ..Default::default()
    };

    let mut output = Vec::new();
    ought_report::terminal::render_to_writer(
        &mut output,
        &run_result,
        &[spec],
        &options,
    ).expect("render_to_writer should succeed");

    let text = String::from_utf8(output).expect("output should be valid UTF-8");

    let clause_line = text
        .lines()
        .find(|l| l.contains("return a response"))
        .expect("MUST BY clause line must appear in output");

    // Both the measured duration and the deadline must be visible on the same line.
    assert!(
        clause_line.contains("47ms"),
        "MUST BY line must show measured duration (47ms); got: {:?}",
        clause_line
    );
    assert!(
        clause_line.contains("200ms"),
        "MUST BY line must show the deadline (200ms); got: {:?}",
        clause_line
    );
}