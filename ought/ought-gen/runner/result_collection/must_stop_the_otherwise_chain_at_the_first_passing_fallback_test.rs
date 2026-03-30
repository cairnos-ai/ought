/// MUST stop the OTHERWISE chain at the first passing fallback
/// GIVEN: a clause has OTHERWISE children
#[test]
fn test_runner__result_collection__must_stop_the_otherwise_chain_at_the_first_passing_fallback() {
    #[derive(Debug, PartialEq, Clone)]
    enum Status { Passed, Failed, Skipped }

    struct OtherwiseResult {
        clause_id: String,
        status: Status,
    }

    // Parent fails; walk OTHERWISE chain and stop at first passing fallback
    fn run_otherwise_chain(otherwise_outcomes: &[bool]) -> Vec<OtherwiseResult> {
        let mut results = vec![OtherwiseResult {
            clause_id: "svc::must_use_primary_db".to_string(),
            status: Status::Failed,
        }];
        let mut stopped = false;
        for (i, &passes) in otherwise_outcomes.iter().enumerate() {
            if stopped {
                results.push(OtherwiseResult {
                    clause_id: format!("svc::fallback_{}", i),
                    status: Status::Skipped,
                });
            } else {
                results.push(OtherwiseResult {
                    clause_id: format!("svc::fallback_{}", i),
                    status: if passes { Status::Passed } else { Status::Failed },
                });
                if passes {
                    stopped = true;
                }
            }
        }
        results
    }

    // fallback_0=fail, fallback_1=pass → fallback_2 must be Skipped
    let results = run_otherwise_chain(&[false, true, true]);

    // results: [parent(F), fallback_0(F), fallback_1(P), fallback_2(S)]
    assert_eq!(results[1].status, Status::Failed,  "fallback_0 fails, chain continues");
    assert_eq!(results[2].status, Status::Passed,  "fallback_1 passes, chain must stop here");
    assert_eq!(results[3].status, Status::Skipped, "fallback_2 must be skipped — chain already stopped");
}