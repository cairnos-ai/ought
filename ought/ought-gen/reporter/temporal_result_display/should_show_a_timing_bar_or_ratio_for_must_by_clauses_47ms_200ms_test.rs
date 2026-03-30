/// SHOULD show a timing bar or ratio for MUST BY clauses: `[47ms / 200ms]`
#[test]
fn test_reporter__temporal_result_display__should_show_a_timing_bar_or_ratio_for_must_by_clauses_47ms_200ms() {
    use ought_report::types::{ColorChoice, ReportOptions};
    use ought_run::{RunResult, TestResult, TestStatus, TestDetails};
    use ought_spec::{Clause, ClauseId, Keyword, Section, Severity, Spec, SourceLocation, Temporal};
    use std::path::PathBuf;
    use std::time::Duration;

    // Test both a passing (within deadline) and a failing (over deadline) MUST BY clause
    // to confirm the ratio format appears regardless of pass/fail.
    let passing_id = ClauseId("perf::must_by_200ms_return_a_response".to_string());
    let failing_id = ClauseId("perf::must_by_100ms_acknowledge_the_write".to_string());

    let clauses = vec![
        Clause {
            id: passing_id.clone(),
            keyword: Keyword::MustBy,
            severity: Severity::Required,
            text: "return a response".to_string(),
            condition: None,
            otherwise: vec![],
            temporal: Some(Temporal::Deadline(Duration::from_millis(200))),
            hints: vec![],
            source_location: SourceLocation { file: PathBuf::from("spec.md"), line: 1 },
            content_hash: "aaa".to_string(),
        },
        Clause {
            id: failing_id.clone(),
            keyword: Keyword::MustBy,
            severity: Severity::Required,
            text: "acknowledge the write".to_string(),
            condition: None,
            otherwise: vec![],
            temporal: Some(Temporal::Deadline(Duration::from_millis(100))),
            hints: vec![],
            source_location: SourceLocation { file: PathBuf::from("spec.md"), line: 2 },
            content_hash: "bbb".to_string(),
        },
    ];

    let section = Section {
        title: "Performance".to_string(),
        depth: 1,
        prose: String::new(),
        clauses,
        subsections: vec![],
    };

    let spec = Spec {
        name: "Timing Spec".to_string(),
        metadata: Default::default(),
        sections: vec![section],
        source_path: PathBuf::from("spec.md"),
    };

    let run_result = RunResult {
        results: vec![
            TestResult {
                clause_id: passing_id.clone(),
                status: TestStatus::Passed,
                message: None,
                duration: Duration::from_millis(47),
                details: TestDetails {
                    measured_duration: Some(Duration::from_millis(47)),
                    ..Default::default()
                },
            },
            TestResult {
                clause_id: failing_id.clone(),
                status: TestStatus::Failed,
                message: None,
                duration: Duration::from_millis(230),
                details: TestDetails {
                    measured_duration: Some(Duration::from_millis(230)),
                    ..Default::default()
                },
            },
        ],
        total_duration: Duration::from_millis(277),
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

    // Passing clause: [47ms / 200ms]
    let passing_line = text
        .lines()
        .find(|l| l.contains("return a response"))
        .expect("passing MUST BY clause line must appear in output");

    assert!(
        passing_line.contains("[47ms") && passing_line.contains("200ms]"),
        "passing MUST BY line must show ratio in bracket form '[47ms / 200ms]'; got: {:?}",
        passing_line
    );

    // Failing clause: [230ms / 100ms]
    let failing_line = text
        .lines()
        .find(|l| l.contains("acknowledge the write"))
        .expect("failing MUST BY clause line must appear in output");

    assert!(
        failing_line.contains("[230ms") && failing_line.contains("100ms]"),
        "failing MUST BY line must show ratio in bracket form '[230ms / 100ms]'; got: {:?}",
        failing_line
    );

    // The separator between measured and deadline should be present (/ or similar).
    assert!(
        passing_line.contains('/'),
        "ratio must include a '/' separator; got: {:?}",
        passing_line
    );
    assert!(
        failing_line.contains('/'),
        "ratio must include a '/' separator; got: {:?}",
        failing_line
    );
}