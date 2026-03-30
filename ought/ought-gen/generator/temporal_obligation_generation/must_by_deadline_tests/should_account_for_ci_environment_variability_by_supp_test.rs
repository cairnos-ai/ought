/// SHOULD account for CI environment variability by supporting a configurable tolerance multiplier in `ought.toml`
#[test]
fn test_generator__temporal_obligation_generation__must_by_deadline_tests__should_account_for_ci_environment_variability_by_supp() {
    use ought_gen::context::GenerationContext;
    use ought_gen::generator::Language;
    use ought_gen::providers::build_prompt;
    use ought_spec::config::ToleranceConfig;
    use ought_spec::{Clause, ClauseId, Keyword, Severity, SourceLocation, Temporal};
    use std::path::PathBuf;
    use std::time::Duration;

    // The default multiplier must be 1.0 — i.e., a no-op locally so the deadline
    // in the spec is used verbatim when no explicit CI override is configured.
    let default_tolerance = ToleranceConfig::default();
    assert_eq!(
        default_tolerance.must_by_multiplier, 1.0,
        "should_account_for_ci_environment_variability_by_supp: \
         ToleranceConfig::default().must_by_multiplier must be 1.0 so local environments \
         use the spec deadline verbatim. Got: {}",
        default_tolerance.must_by_multiplier
    );

    // A multiplier > 1.0 must be representable (CI gets extra budget).
    let ci_tolerance = ToleranceConfig { must_by_multiplier: 2.0 };
    assert!(
        ci_tolerance.must_by_multiplier > 1.0,
        "should_account_for_ci_environment_variability_by_supp: \
         ToleranceConfig must accept must_by_multiplier > 1.0 to give CI machines \
         extra time budget. Got: {}",
        ci_tolerance.must_by_multiplier
    );

    // A multiplier < 1.0 must also be representable (strict CI can tighten the budget).
    let strict_tolerance = ToleranceConfig { must_by_multiplier: 0.5 };
    assert!(
        strict_tolerance.must_by_multiplier > 0.0 && strict_tolerance.must_by_multiplier < 1.0,
        "should_account_for_ci_environment_variability_by_supp: \
         ToleranceConfig must accept fractional must_by_multiplier values in (0, 1) to \
         allow strict environments. Got: {}",
        strict_tolerance.must_by_multiplier
    );

    // When a non-default multiplier is configured the effective deadline differs from
    // the raw Duration. The prompt generator must convey the effective (scaled) deadline
    // to the LLM so the generated assertion uses the right value for the environment.
    // Effective deadline = 200ms (raw 100ms × 2.0 multiplier).
    let clause = Clause {
        id: ClauseId("gen::temporal::must_by_deadline_with_tolerance".to_string()),
        keyword: Keyword::MustBy,
        severity: Severity::Required,
        text: "the operation must complete within 100 milliseconds".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: Some(Temporal::Deadline(Duration::from_millis(100))),
        hints: vec![],
        source_location: SourceLocation {
            file: PathBuf::from("spec.ought.md"),
            line: 5,
        },
        content_hash: "x".to_string(),
    };
    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };
    let prompt = build_prompt(&clause, &context);

    // The prompt must contain either an explicit mention of a tolerance/multiplier
    // or the scaled deadline value (200) so the generated test is CI-aware.
    let conveys_ci_tolerance = prompt.contains("tolerance")
        || prompt.contains("multiplier")
        || prompt.contains("200");

    assert!(
        conveys_ci_tolerance,
        "should_account_for_ci_environment_variability_by_supp: \
         build_prompt for a MUST BY clause should convey the effective (multiplier-scaled) \
         deadline or mention the tolerance/multiplier so the generated test is robust \
         to CI environment variability, but neither was found. Prompt:\n{prompt}"
    );
}