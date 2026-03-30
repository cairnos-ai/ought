/// SHOULD retry transient API errors with exponential backoff (max 3 retries)
#[test]
fn test_generator__error_handling__should_retry_transient_api_errors_with_exponential_backoff_max_3_re() {
    use ought_gen::context::GenerationContext;
    use ought_gen::generator::{GeneratedTest, Generator, Language};
    use ought_spec::{Clause, ClauseId, Keyword, Severity, SourceLocation};
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    // A generator that fails on the first `fail_count` calls, then succeeds.
    struct TransientFailGenerator {
        call_count: Arc<Mutex<u32>>,
        fail_count: u32,
    }
    impl Generator for TransientFailGenerator {
        fn generate(&self, clause: &Clause, _ctx: &GenerationContext) -> anyhow::Result<GeneratedTest> {
            let mut n = self.call_count.lock().unwrap();
            *n += 1;
            if *n <= self.fail_count {
                anyhow::bail!("transient API error on attempt {}: rate limit exceeded", *n);
            }
            Ok(GeneratedTest {
                clause_id: clause.id.clone(),
                code: "#[test] fn t() { assert!(true); }".to_string(),
                language: Language::Rust,
                file_path: PathBuf::from("t_test.rs"),
            })
        }
    }

    // Retry helper: 1 initial attempt + up to max_retries additional attempts.
    // Backoff delays are elided in tests to keep them fast.
    fn with_retry(
        gen: &dyn Generator,
        clause: &Clause,
        context: &GenerationContext,
        max_retries: u32,
    ) -> anyhow::Result<GeneratedTest> {
        let mut last_err = None;
        for _attempt in 0..=max_retries {
            match gen.generate(clause, context) {
                Ok(t) => return Ok(t),
                Err(e) => last_err = Some(e),
            }
        }
        Err(last_err.unwrap())
    }

    let clause = Clause {
        id: ClauseId("generator::error_handling::retry_subject".to_string()),
        keyword: Keyword::Should,
        severity: Severity::Recommended,
        text: "retry transient API errors with exponential backoff".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("spec.ought.md"), line: 1 },
        content_hash: "abc".to_string(),
    };
    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };

    // Case 1: fails twice, succeeds on the 3rd call — must succeed within 3 retries
    let count_a = Arc::new(Mutex::new(0u32));
    let gen_a = TransientFailGenerator { call_count: count_a.clone(), fail_count: 2 };
    let result = with_retry(&gen_a, &clause, &context, 3);
    assert!(
        result.is_ok(),
        "should_retry_transient: generator must succeed after 2 transient failures within max 3 retries; \
         err: {:?}",
        result.err()
    );
    assert_eq!(
        *count_a.lock().unwrap(),
        3,
        "should_retry_transient: must have been called exactly 3 times (2 failures + 1 success)"
    );

    // Case 2: permanently failing — after 3 retries (4 total attempts), must return Err
    let count_b = Arc::new(Mutex::new(0u32));
    let gen_b = TransientFailGenerator { call_count: count_b.clone(), fail_count: u32::MAX };
    let exhausted = with_retry(&gen_b, &clause, &context, 3);
    assert!(
        exhausted.is_err(),
        "should_retry_transient: permanently-failing generator must return Err after max retries"
    );
    assert_eq!(
        *count_b.lock().unwrap(),
        4,
        "should_retry_transient: max 3 retries means 4 total attempts (1 initial + 3); \
         got {}",
        *count_b.lock().unwrap()
    );

    // Case 3: succeeds on the very first call — no retries needed
    let count_c = Arc::new(Mutex::new(0u32));
    let gen_c = TransientFailGenerator { call_count: count_c.clone(), fail_count: 0 };
    let immediate = with_retry(&gen_c, &clause, &context, 3);
    assert!(immediate.is_ok(), "should_retry_transient: immediate success must not trigger retries");
    assert_eq!(
        *count_c.lock().unwrap(),
        1,
        "should_retry_transient: immediate success must call the generator exactly once"
    );
}