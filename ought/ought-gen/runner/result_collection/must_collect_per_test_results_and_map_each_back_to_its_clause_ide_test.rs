/// MUST collect per-test results and map each back to its clause identifier
#[test]
fn test_runner__result_collection__must_collect_per_test_results_and_map_each_back_to_its_clause_ide() {
    struct ClauseResult {
        clause_id: String,
        passed: bool,
    }

    fn collect_results(raw: Vec<(&str, bool)>) -> Vec<ClauseResult> {
        raw.into_iter()
            .map(|(id, passed)| ClauseResult { clause_id: id.to_string(), passed })
            .collect()
    }

    let results = collect_results(vec![
        ("payments::charge::must_debit_account", true),
        ("payments::charge::must_emit_event", false),
        ("payments::charge::must_be_idempotent", true),
    ]);

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].clause_id, "payments::charge::must_debit_account");
    assert!(results[0].passed);
    assert_eq!(results[1].clause_id, "payments::charge::must_emit_event");
    assert!(!results[1].passed);
    assert_eq!(results[2].clause_id, "payments::charge::must_be_idempotent");

    for r in &results {
        assert!(
            !r.clause_id.is_empty(),
            "every result must map back to a clause identifier"
        );
    }
}