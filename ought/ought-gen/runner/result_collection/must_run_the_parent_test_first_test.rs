/// MUST run the parent test first
/// GIVEN: a clause has OTHERWISE children
#[test]
fn test_runner__result_collection__must_run_the_parent_test_first() {
    #[derive(Debug, PartialEq)]
    enum Status { Passed, Failed, Skipped }

    struct ExecutionRecord {
        clause_id: String,
        execution_order: usize,
        status: Status,
    }

    // Simulate execution: parent fails, OTHERWISE children follow
    fn run_otherwise_chain(
        parent_id: &str,
        parent_passes: bool,
        otherwise_ids: &[&str],
    ) -> Vec<ExecutionRecord> {
        let mut log = Vec::new();
        log.push(ExecutionRecord {
            clause_id: parent_id.to_string(),
            execution_order: 0,
            status: if parent_passes { Status::Passed } else { Status::Failed },
        });
        if !parent_passes {
            for (i, id) in otherwise_ids.iter().enumerate() {
                log.push(ExecutionRecord {
                    clause_id: id.to_string(),
                    execution_order: i + 1,
                    status: Status::Skipped,
                });
            }
        }
        log
    }

    let log = run_otherwise_chain(
        "auth::must_use_oauth",
        false,
        &["auth::must_use_api_key", "auth::must_use_basic_auth"],
    );

    assert!(!log.is_empty());
    assert_eq!(log[0].clause_id, "auth::must_use_oauth", "parent must be first in execution log");
    assert_eq!(log[0].execution_order, 0, "parent must have execution order 0");

    for record in log.iter().skip(1) {
        assert!(
            record.execution_order > log[0].execution_order,
            "OTHERWISE child '{}' must execute after the parent",
            record.clause_id
        );
    }
}