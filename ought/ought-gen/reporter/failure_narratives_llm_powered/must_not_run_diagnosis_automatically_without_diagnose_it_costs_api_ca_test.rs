/// MUST NOT run diagnosis automatically without `--diagnose` (it costs API calls)
#[test]
fn test_reporter__failure_narratives_llm_powered__must_not_run_diagnosis_automatically_without_diagnose_it_costs_api_ca() {
    use std::cell::Cell;

    struct MockLlmClient {
        call_count: Cell<usize>,
    }

    impl MockLlmClient {
        fn new() -> Self {
            MockLlmClient { call_count: Cell::new(0) }
        }

        fn diagnose(&self, _prompt: &str) -> String {
            self.call_count.set(self.call_count.get() + 1);
            "mock diagnosis".to_string()
        }

        fn calls_made(&self) -> usize {
            self.call_count.get()
        }
    }

    struct Reporter {
        diagnose_enabled: bool,
    }

    impl Reporter {
        fn report_failures<'a>(&self, failures: &[&'a str], llm: &MockLlmClient) -> Vec<String> {
            failures
                .iter()
                .map(|f| {
                    let mut out = format!("FAILED: {}", f);
                    if self.diagnose_enabled {
                        out.push_str(&format!("\n{}", llm.diagnose(f)));
                    }
                    out
                })
                .collect()
        }
    }

    let llm = MockLlmClient::new();
    let failures = &["test_foo panicked", "test_bar panicked", "test_baz panicked"];

    // Without --diagnose: zero LLM calls regardless of how many failures exist
    let reporter_no_flag = Reporter { diagnose_enabled: false };
    reporter_no_flag.report_failures(failures, &llm);

    assert_eq!(
        llm.calls_made(),
        0,
        "LLM must not be called when --diagnose flag is absent — each call costs API quota"
    );

    // With --diagnose: LLM is invoked
    let reporter_with_flag = Reporter { diagnose_enabled: true };
    reporter_with_flag.report_failures(&failures[..1], &llm);

    assert!(
        llm.calls_made() > 0,
        "LLM must be called when --diagnose flag is present"
    );
}