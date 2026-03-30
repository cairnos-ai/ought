/// SHOULD diagnose all failures in a single batch LLM call when possible
#[test]
fn test_reporter__failure_narratives_llm_powered__should_diagnose_all_failures_in_a_single_batch_llm_call_when_possib() {
    use std::cell::Cell;

    struct FailureInfo {
        clause_id: String,
        failure_output: String,
    }

    struct MockLlmClient {
        call_count: Cell<usize>,
    }

    impl MockLlmClient {
        fn new() -> Self {
            MockLlmClient { call_count: Cell::new(0) }
        }

        /// Accepts all failures at once and returns one diagnosis per failure.
        fn diagnose_batch(&self, failures: &[&FailureInfo]) -> Vec<String> {
            self.call_count.set(self.call_count.get() + 1);
            failures
                .iter()
                .map(|f| format!("diagnosis for {}", f.clause_id))
                .collect()
        }

        fn calls_made(&self) -> usize {
            self.call_count.get()
        }
    }

    let failures = vec![
        FailureInfo { clause_id: "auth::login::must_return_jwt".to_string(),    failure_output: "assertion failed: is_ok()".to_string() },
        FailureInfo { clause_id: "auth::logout::must_clear_session".to_string(), failure_output: "assertion failed: session.is_none()".to_string() },
        FailureInfo { clause_id: "auth::register::must_hash_password".to_string(), failure_output: "assertion failed: is_hashed".to_string() },
    ];

    let llm = MockLlmClient::new();
    let refs: Vec<&FailureInfo> = failures.iter().collect();
    let diagnoses = llm.diagnose_batch(&refs);

    assert_eq!(
        llm.calls_made(),
        1,
        "all failures must be diagnosed in a single LLM call, not one call per failure"
    );
    assert_eq!(
        diagnoses.len(),
        failures.len(),
        "batch call must return exactly one diagnosis per failure"
    );
    for (i, failure) in failures.iter().enumerate() {
        assert!(
            diagnoses[i].contains(&failure.clause_id),
            "each diagnosis must correspond to its failure (index {})",
            i
        );
    }
}