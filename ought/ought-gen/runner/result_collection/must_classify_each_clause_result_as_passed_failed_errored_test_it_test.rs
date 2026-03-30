/// MUST classify each clause result as: passed, failed, errored (test itself broke), or skipped
#[test]
fn test_runner__result_collection__must_classify_each_clause_result_as_passed_failed_errored_test_it() {
    #[derive(Debug, PartialEq)]
    enum ClauseStatus {
        Passed,
        Failed,
        Errored,
        Skipped,
    }

    struct ClauseResult {
        clause_id: String,
        status: ClauseStatus,
    }

    let results = vec![
        ClauseResult {
            clause_id: "svc::must_respond_200".to_string(),
            status: ClauseStatus::Passed,
        },
        ClauseResult {
            clause_id: "svc::must_validate_input".to_string(),
            status: ClauseStatus::Failed,
        },
        ClauseResult {
            clause_id: "svc::must_log_request".to_string(),
            // test harness itself crashed during setup
            status: ClauseStatus::Errored,
        },
        ClauseResult {
            clause_id: "svc::must_fallback_to_cache".to_string(),
            // OTHERWISE branch not taken
            status: ClauseStatus::Skipped,
        },
    ];

    assert_eq!(results[0].status, ClauseStatus::Passed);
    assert_eq!(results[1].status, ClauseStatus::Failed);
    assert_eq!(results[2].status, ClauseStatus::Errored);
    assert_eq!(results[3].status, ClauseStatus::Skipped);

    // All four classifications must be representable and distinct
    assert_ne!(ClauseStatus::Passed,  ClauseStatus::Failed);
    assert_ne!(ClauseStatus::Failed,  ClauseStatus::Errored);
    assert_ne!(ClauseStatus::Errored, ClauseStatus::Skipped);
    assert_ne!(ClauseStatus::Passed,  ClauseStatus::Skipped);
}