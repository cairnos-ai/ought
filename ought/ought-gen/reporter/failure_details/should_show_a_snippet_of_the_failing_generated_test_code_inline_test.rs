/// SHOULD show a snippet of the failing generated test code inline
#[test]
fn test_reporter__failure_details__should_show_a_snippet_of_the_failing_generated_test_code_inline() {
    use ought_report::types::{ColorChoice, ReportOptions};
    use ought_run::{RunResult, TestDetails, TestResult, TestStatus};
    use ought_spec::{Clause, ClauseId, Keyword, Section, Severity, Spec, SourceLocation};
    use std::path::PathBuf;
    use std::time::Duration;

    let clause_id = ClauseId("storage::must_persist_data_across_restarts".to_string());

    // The failure_message includes a code snippet showing the assertion that failed,
    // as captured by ought-run from the generated test file.
    let failure_msg = concat!(
        "assertion failed: store.get(\"key\") == Some(\"value\")\n",
        "  --> tests/generated/storage.rs:31\n",
        "   |\n",
        "31 |     assert_eq!(store.get(\"key\"), Some(\"value\"));\n",
        "   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n",
    );

    let clause = Clause {
        id: clause_id.clone(),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: "persist data across restarts".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("storage.md"), line: 8 },
        content_hash: "ghi789".to_string(),
    };

    let section = Section {
        title: "Storage".to_string(),
        depth: 1,
        prose: String::new(),
        clauses: vec![clause],
        subsections: vec![],
    };

    let spec = Spec {
        name: "Storage Spec".to_string(),
        metadata: Default::default(),
        sections: vec![section],
        source_path: PathBuf::from("storage.md"),
    };

    let run_result = RunResult {
        results: vec![TestResult {
            clause_id: clause_id.clone(),
            status: TestStatus::Failed,
            message: None,
            duration: Duration::from_millis(12),
            details: TestDetails {
                failure_message: Some(failure_msg.to_string()),
                ..Default::default()
            },
        }],
        total_duration: Duration::from_millis(12),
    };

    let options = ReportOptions {
        color: ColorChoice::Never,
        ..Default::default()
    };

    let mut output = Vec::new();
    ought_report::terminal::render_to_writer(&mut output, &run_result, &[spec], &options)
        .expect("render_to_writer should succeed");
    let text = String::from_utf8(output).expect("output should be valid UTF-8");

    // The inline code snippet (the assert_eq! line) must appear in the output.
    assert!(
        text.contains("assert_eq!"),
        "output must show a snippet of the failing generated test code inline; got:\n{text}"
    );
    // The snippet's visual pointer (^^^) must also be shown so the user sees exactly which
    // expression failed.
    assert!(
        text.contains("^^^"),
        "output must show the caret annotation identifying the failing expression; got:\n{text}"
    );
    // The snippet must be rendered below the failing clause line, not on it.
    let clause_line_pos = text
        .lines()
        .position(|l| l.contains("persist data across restarts"))
        .expect("clause text must appear in output");
    let snippet_line_pos = text
        .lines()
        .position(|l| l.contains("assert_eq!"))
        .expect("code snippet must appear in output");
    assert!(
        snippet_line_pos > clause_line_pos,
        "code snippet must appear after (below) the clause line in the output"
    );
}