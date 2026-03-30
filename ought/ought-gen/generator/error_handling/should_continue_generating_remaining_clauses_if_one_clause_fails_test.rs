/// SHOULD continue generating remaining clauses if one clause fails
#[test]
fn test_generator__error_handling__should_continue_generating_remaining_clauses_if_one_clause_fails() {
    use ought_gen::context::GenerationContext;
    use ought_gen::generator::{GeneratedTest, Generator, Language};
    use ought_spec::{Clause, ClauseId, Keyword, Severity, SourceLocation};
    use std::path::PathBuf;

    // A generator that fails for one specific clause ID, succeeds for all others.
    struct SelectiveFail {
        fail_id: &'static str,
    }
    impl Generator for SelectiveFail {
        fn generate(&self, clause: &Clause, _ctx: &GenerationContext) -> anyhow::Result<GeneratedTest> {
            if clause.id.0 == self.fail_id {
                anyhow::bail!(
                    "simulated LLM API error for clause '{}'",
                    clause.id
                );
            }
            Ok(GeneratedTest {
                clause_id: clause.id.clone(),
                code: format!(
                    "#[test] fn test_{}() {{ assert!(true); }}",
                    clause.id.0.replace("::", "__")
                ),
                language: Language::Rust,
                file_path: PathBuf::from(format!("{}_test.rs", clause.id.0.replace("::", "/"))),
            })
        }
    }

    fn mk(id: &str, kw: Keyword, sev: Severity) -> Clause {
        Clause {
            id: ClauseId(id.to_string()),
            keyword: kw,
            severity: sev,
            text: id.to_string(),
            condition: None,
            otherwise: vec![],
            temporal: None,
            hints: vec![],
            source_location: SourceLocation { file: PathBuf::from("spec.ought.md"), line: 1 },
            content_hash: "x".to_string(),
        }
    }

    let clauses = vec![
        mk("gen::error_handling::ok_first",  Keyword::Should, Severity::Recommended),
        mk("gen::error_handling::will_fail", Keyword::Should, Severity::Recommended),
        mk("gen::error_handling::ok_last",   Keyword::Should, Severity::Recommended),
    ];
    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };
    let gen = SelectiveFail { fail_id: "gen::error_handling::will_fail" };

    // Lenient per-clause loop: collect successes and failures without stopping early.
    // This is the expected orchestration behaviour for "continue on failure".
    let mut successes: Vec<GeneratedTest> = Vec::new();
    let mut failures: Vec<(ClauseId, String)> = Vec::new();
    for clause in &clauses {
        match gen.generate(clause, &context) {
            Ok(t) => successes.push(t),
            Err(e) => failures.push((clause.id.clone(), e.to_string())),
        }
    }

    assert_eq!(
        successes.len(),
        2,
        "should_continue_on_clause_failure: 2 of 3 clauses must succeed; got {}",
        successes.len()
    );
    assert_eq!(
        failures.len(),
        1,
        "should_continue_on_clause_failure: exactly 1 clause must fail; got {}",
        failures.len()
    );

    // ok_first must have been generated
    assert_eq!(
        successes[0].clause_id,
        ClauseId("gen::error_handling::ok_first".to_string()),
        "should_continue_on_clause_failure: first success must be ok_first"
    );
    // ok_last must also have been generated, proving generation continued past the failure
    assert_eq!(
        successes[1].clause_id,
        ClauseId("gen::error_handling::ok_last".to_string()),
        "should_continue_on_clause_failure: ok_last must be generated even though will_fail errored"
    );

    // The failing clause must be identified in the failure list
    assert_eq!(
        failures[0].0,
        ClauseId("gen::error_handling::will_fail".to_string()),
        "should_continue_on_clause_failure: failures list must identify the failed clause"
    );
    assert!(
        failures[0].1.contains("gen::error_handling::will_fail"),
        "should_continue_on_clause_failure: failure message must reference the clause that failed; \
         got: {}",
        failures[0].1
    );

    // Generated test code for successful clauses must not bleed across the failed clause
    assert!(
        !successes[0].code.contains("will_fail"),
        "should_continue_on_clause_failure: ok_first test code must not reference will_fail"
    );
    assert!(
        !successes[1].code.contains("will_fail"),
        "should_continue_on_clause_failure: ok_last test code must not reference will_fail"
    );
}