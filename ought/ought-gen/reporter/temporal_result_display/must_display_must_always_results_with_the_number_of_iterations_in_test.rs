/// MUST display MUST ALWAYS results with the number of iterations/inputs tested
#[test]
fn test_reporter__temporal_result_display__must_display_must_always_results_with_the_number_of_iterations_in() {
    use ought_report::types::{ColorChoice, ReportOptions};
    use ought_run::{RunResult, TestResult, TestStatus, TestDetails};
    use ought_spec::{Clause, ClauseId, Keyword, Section, Severity, Spec, SourceLocation, Temporal};
    use std::path::PathBuf;
    use std::time::Duration;

    let clause_id = ClauseId("invariant::must_always_return_valid_json".to_string());

    let clause = Clause {
        id: clause_id.clone(),
        keyword: Keyword::MustAlways,
        severity: Severity::Required,
        text: "return valid JSON".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: Some(Temporal::Invariant),
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("spec.md"), line: 1 },
        content_hash: "abc".to_string(),
    };

    let section = Section {
        title: "Invariants".to_string(),
        depth: 1,
        prose: String::new(),
        clauses: vec![clause],
        subsections: vec![],
    };

    let spec = Spec {
        name: "Test Spec".to_string(),
        metadata: Default::default(),
        sections: vec![section],
        source_path: PathBuf::from("spec.md"),
    };

    let run_result = RunResult {
        results: vec![TestResult {
            clause_id: clause_id.clone(),
            status: TestStatus::Passed,
            message: None,
            duration: Duration::from_millis(50),
            details: TestDetails {
                iterations: Some(1000),
                ..Default::default()
            },
        }],
        total_duration: Duration::from_millis(50),
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
        .find(|l| l.contains("return valid JSON"))
        .expect("MUST ALWAYS clause line must appear in output");

    assert!(
        clause_line.contains("1000"),
        "MUST ALWAYS line must include iteration count; got: {:?}",
        clause_line
    );
    assert!(
        clause_line.contains("tested") || clause_line.contains("inputs"),
        "MUST ALWAYS line must describe what was tested (e.g. 'tested 1000 inputs'); got: {:?}",
        clause_line
    );
}