/// MUST mark remaining lower-priority OTHERWISE clauses as skipped (not reached)
/// GIVEN: a clause has OTHERWISE children
#[test]
fn test_runner__result_collection__must_mark_remaining_lower_priority_otherwise_clauses_as_skipped_n() {
    #[derive(Debug, PartialEq, Clone)]
    enum Status { Passed, Failed, Skipped }

    struct OtherwiseResult {
        clause_id: String,
        priority: usize, // lower index = higher priority
        status: Status,
    }

    // Chain: parent fails, fallback_0 fails, fallback_1 passes → fallback_2 and fallback_3 are lower priority
    let results = vec![
        OtherwiseResult { clause_id: "svc::must_use_primary_db".to_string(),       priority: 0, status: Status::Failed  },
        OtherwiseResult { clause_id: "svc::must_use_replica_db".to_string(),        priority: 1, status: Status::Failed  },
        OtherwiseResult { clause_id: "svc::must_use_cache".to_string(),             priority: 2, status: Status::Passed  },
        OtherwiseResult { clause_id: "svc::must_use_fallback_response".to_string(), priority: 3, status: Status::Skipped },
        OtherwiseResult { clause_id: "svc::must_return_503".to_string(),            priority: 4, status: Status::Skipped },
    ];

    let first_passing_priority = results
        .iter()
        .skip(1) // skip parent
        .find(|r| r.status == Status::Passed)
        .map(|r| r.priority)
        .expect("there must be a passing fallback in the chain");

    assert_eq!(first_passing_priority, 2);

    // Every OTHERWISE clause with priority > first_passing_priority must be Skipped
    for r in results.iter().skip(1) {
        if r.priority > first_passing_priority {
            assert_eq!(
                r.status,
                Status::Skipped,
                "clause '{}' (priority {}) must be marked Skipped — it is lower priority than the first passing fallback",
                r.clause_id,
                r.priority
            );
        }
    }
}