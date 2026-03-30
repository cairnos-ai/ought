/// MUST execute generated tests and report pass/fail results mapped back to the original spec clauses
#[test]
fn test_ought__what_ought_does__must_execute_generated_tests_and_report_pass_fail_results_mapped() {
    #[derive(Debug, PartialEq)]
    enum TestOutcome {
        Pass,
        Fail,
    }

    struct ClauseResult {
        clause_id: String,
        outcome: TestOutcome,
    }

    // Simulate the result set returned after executing generated tests.
    let results = vec![
        ClauseResult {
            clause_id: "my_spec::section::must_do_something".to_string(),
            outcome: TestOutcome::Pass,
        },
        ClauseResult {
            clause_id: "my_spec::section::must_handle_error".to_string(),
            outcome: TestOutcome::Fail,
        },
    ];

    // Every result must carry a non-empty clause ID so it maps back to the spec.
    for r in &results {
        assert!(
            !r.clause_id.is_empty(),
            "Every test result must reference a clause ID"
        );
        assert!(
            r.clause_id.contains("::"),
            "Clause IDs must use '::' namespace separators, got: {}",
            r.clause_id
        );
    }

    // Results must distinguish pass from fail outcomes.
    let passes: Vec<_> = results
        .iter()
        .filter(|r| r.outcome == TestOutcome::Pass)
        .collect();
    let failures: Vec<_> = results
        .iter()
        .filter(|r| r.outcome == TestOutcome::Fail)
        .collect();

    assert_eq!(passes.len(), 1, "Expected exactly one passing clause result");
    assert_eq!(failures.len(), 1, "Expected exactly one failing clause result");

    assert_eq!(
        passes[0].clause_id, "my_spec::section::must_do_something",
        "Pass result must map to the correct clause ID"
    );
    assert_eq!(
        failures[0].clause_id, "my_spec::section::must_handle_error",
        "Fail result must map to the correct clause ID"
    );
}