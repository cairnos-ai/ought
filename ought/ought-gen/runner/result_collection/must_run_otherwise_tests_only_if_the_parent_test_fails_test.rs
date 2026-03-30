/// MUST run OTHERWISE tests only if the parent test fails
/// GIVEN: a clause has OTHERWISE children
#[test]
fn test_runner__result_collection__must_run_otherwise_tests_only_if_the_parent_test_fails() {
    #[derive(Debug, PartialEq, Clone)]
    enum Status { Passed, Failed, NotRun }

    struct OtherwiseResult {
        clause_id: String,
        status: Status,
    }

    fn run_with_otherwise(parent_passes: bool) -> Vec<OtherwiseResult> {
        let mut results = vec![OtherwiseResult {
            clause_id: "auth::must_use_oauth".to_string(),
            status: if parent_passes { Status::Passed } else { Status::Failed },
        }];
        if parent_passes {
            results.push(OtherwiseResult {
                clause_id: "auth::must_use_api_key".to_string(),
                status: Status::NotRun,
            });
        } else {
            results.push(OtherwiseResult {
                clause_id: "auth::must_use_api_key".to_string(),
                status: Status::Passed,
            });
        }
        results
    }

    // When parent passes: OTHERWISE child must not run
    let pass_results = run_with_otherwise(true);
    assert_eq!(pass_results[0].status, Status::Passed);
    assert_eq!(
        pass_results[1].status,
        Status::NotRun,
        "OTHERWISE child must not run when parent passes"
    );

    // When parent fails: OTHERWISE child must be run
    let fail_results = run_with_otherwise(false);
    assert_eq!(fail_results[0].status, Status::Failed);
    assert_ne!(
        fail_results[1].status,
        Status::NotRun,
        "OTHERWISE child must run when parent fails"
    );
}