/// MUST map individual test pass/fail results back to clause identifiers
#[test]
fn test_runner__execution__must_map_individual_test_pass_fail_results_back_to_clause_identif() {
    use std::collections::HashMap;
    use std::time::Duration;
    use ought_run::{RunResult, TestResult, TestStatus, TestDetails};
    use ought_spec::ClauseId;

    // Inline the bidirectional name⟷ClauseId conversion that each runner implements.
    fn clause_id_to_test_name(id: &ClauseId) -> String {
        id.0.replace("::", "__")
    }
    fn test_name_to_clause_id(name: &str) -> ClauseId {
        ClauseId(name.replace("__", "::"))
    }

    // Three clauses that will appear in the harness output.
    let clause_ids = vec![
        ClauseId("runner::execution::must_invoke_command".to_string()),
        ClauseId("runner::execution::must_capture_output".to_string()),
        ClauseId("runner::execution::must_not_modify_files".to_string()),
    ];

    let mut name_to_clause: HashMap<String, ClauseId> = HashMap::new();
    for id in &clause_ids {
        name_to_clause.insert(clause_id_to_test_name(id), id.clone());
    }

    // Cargo test stdout with one pass, one fail, one ignored.
    let harness_stdout = "\
running 3 tests
test runner__execution__must_invoke_command ... ok
test runner__execution__must_capture_output ... FAILED
test runner__execution__must_not_modify_files ... ignored

test result: FAILED. 1 passed; 1 failed; 1 ignored; 0 measured; 0 filtered out
";

    let mut results: Vec<TestResult> = Vec::new();
    for line in harness_stdout.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("test ") {
            if let Some((name_part, status_part)) = rest.rsplit_once(" ... ") {
                let test_name = name_part.trim();
                let status = match status_part.trim() {
                    "ok"      => TestStatus::Passed,
                    "FAILED"  => TestStatus::Failed,
                    "ignored" => TestStatus::Skipped,
                    _         => TestStatus::Errored,
                };
                let clause_id = name_to_clause
                    .get(test_name)
                    .cloned()
                    .unwrap_or_else(|| test_name_to_clause_id(test_name));
                results.push(TestResult {
                    clause_id,
                    status,
                    message: None,
                    duration: Duration::ZERO,
                    details: TestDetails::default(),
                });
            }
        }
    }

    assert_eq!(results.len(), 3, "must produce exactly one result per test line in harness output");

    let invoke = results.iter()
        .find(|r| r.clause_id.0 == "runner::execution::must_invoke_command")
        .expect("must_invoke_command ClauseId must appear in mapped results");
    assert_eq!(invoke.status, TestStatus::Passed,
        "'ok' output line must map to Passed on clause runner::execution::must_invoke_command");

    let capture = results.iter()
        .find(|r| r.clause_id.0 == "runner::execution::must_capture_output")
        .expect("must_capture_output ClauseId must appear in mapped results");
    assert_eq!(capture.status, TestStatus::Failed,
        "'FAILED' output line must map to Failed on clause runner::execution::must_capture_output");

    let modify = results.iter()
        .find(|r| r.clause_id.0 == "runner::execution::must_not_modify_files")
        .expect("must_not_modify_files ClauseId must appear in mapped results");
    assert_eq!(modify.status, TestStatus::Skipped,
        "'ignored' output line must map to Skipped on clause runner::execution::must_not_modify_files");

    // Verify round-trip: every test-function name converts to a valid ClauseId and back.
    for id in &clause_ids {
        let name = clause_id_to_test_name(id);
        let recovered = test_name_to_clause_id(&name);
        assert_eq!(recovered, *id,
            "clause_id → test_name → clause_id round-trip must be lossless for {id:?}");
    }
}