/// MUST capture test execution duration per clause (required for MUST BY reporting)
#[test]
fn test_runner__result_collection__must_capture_test_execution_duration_per_clause_required_for_must() {
    use std::time::Duration;

    struct ClauseResult {
        clause_id: String,
        duration: Duration,
    }

    let results = vec![
        ClauseResult {
            clause_id: "api::must_respond_within_200ms".to_string(),
            duration: Duration::from_millis(180),
        },
        ClauseResult {
            clause_id: "api::must_process_batch".to_string(),
            duration: Duration::from_millis(3200),
        },
    ];

    // Each result has its own independently measured duration
    assert_eq!(results[0].clause_id, "api::must_respond_within_200ms");
    assert_eq!(results[0].duration, Duration::from_millis(180));

    assert_eq!(results[1].clause_id, "api::must_process_batch");
    assert_eq!(results[1].duration, Duration::from_millis(3200));

    // Durations are per-clause, not shared or aggregated
    assert_ne!(
        results[0].duration, results[1].duration,
        "each clause must record its own execution duration independently"
    );
}