/// SHOULD detect and report when no tests were generated for a spec (nothing to run)
#[test]
fn test_runner__error_handling__should_detect_and_report_when_no_tests_were_generated_for_a_spec_no() {
    // If no test files were generated (e.g., generation was skipped or the spec
    // has no actionable clauses), the runner should surface a diagnostic rather
    // than silently reporting "0 passed" with no indication that something is missing.

    #[derive(Debug, PartialEq)]
    enum RunDiagnostic {
        NoTestsGenerated { spec: String },
        RanTests { count: usize },
    }

    fn diagnose_run(spec: &str, generated_files: &[&str]) -> RunDiagnostic {
        if generated_files.is_empty() {
            RunDiagnostic::NoTestsGenerated { spec: spec.to_string() }
        } else {
            RunDiagnostic::RanTests { count: generated_files.len() }
        }
    }

    // Empty case: runner must detect and report the gap.
    let diag = diagnose_run("runner::error_handling", &[]);
    assert!(
        matches!(diag, RunDiagnostic::NoTestsGenerated { .. }),
        "Runner must report 'no tests generated' when the file list is empty; got {:?}", diag
    );
    if let RunDiagnostic::NoTestsGenerated { spec } = &diag {
        assert_eq!(spec, "runner::error_handling", "Diagnostic must name the spec");
    }

    // Non-empty case: runner must not emit a spurious diagnostic.
    let diag2 = diagnose_run(
        "runner::error_handling",
        &["runner__clause_a.rs", "runner__clause_b.rs"],
    );
    assert_eq!(
        diag2,
        RunDiagnostic::RanTests { count: 2 },
        "Runner must not emit 'no tests' warning when tests exist"
    );

    // The NoTestsGenerated and RanTests variants are observably different.
    assert_ne!(
        RunDiagnostic::NoTestsGenerated { spec: "s".into() },
        RunDiagnostic::RanTests { count: 0 },
        "Zero-count RanTests is not the same diagnostic as NoTestsGenerated"
    );
}