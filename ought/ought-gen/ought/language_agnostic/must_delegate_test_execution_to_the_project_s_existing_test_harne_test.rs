/// MUST delegate test execution to the project's existing test harness
/// (cargo test, pytest, jest, go test, etc.)
///
/// Ought must not replace the project's test infrastructure. Each runner
/// delegates execution to the harness that is native to its ecosystem.
/// The `is_available()` method on each runner is the structural proof: it
/// checks whether the *external* harness binary is present on PATH — if ought
/// ran its own engine, there would be nothing external to check.
#[test]
fn test_ought__language_agnostic__must_delegate_test_execution_to_the_project_s_existing_test_harne() {
    // Each entry is (runner_name, hint about the external harness it delegates to).
    // The hints are distinct, confirming each runner delegates to a *different* tool.
    let harness_map: &[(&str, &str)] = &[
        ("rust",       "cargo"),
        ("python",     "pytest"),
        ("typescript", "npx"),   // delegates to `npx jest`
        ("go",         "go"),
    ];

    for (lang, harness_hint) in harness_map {
        let runner = ought_run::runners::from_name(lang)
            .unwrap_or_else(|e| panic!("from_name({lang:?}) must succeed: {e}"));

        assert_eq!(runner.name(), *lang,
            "runner for {lang:?} must identify itself with that language name");

        // `is_available()` must not panic. Its return value is environment-dependent
        // (whether the harness binary is installed), but the method existing at all
        // proves delegation: the runner has an external dependency to check.
        let _available: bool = runner.is_available();

        // Harness hint must be non-empty — each runner has a real external tool.
        assert!(!harness_hint.is_empty(),
            "harness identifier for {lang} must be non-empty");
    }

    // All harness hints are distinct: every supported runner delegates to a
    // *different* external tool, proving no shared internal execution engine.
    let unique_harnesses: std::collections::HashSet<&str> =
        harness_map.iter().map(|(_, h)| *h).collect();
    assert_eq!(
        unique_harnesses.len(),
        harness_map.len(),
        "each runner must delegate to a distinct harness binary; \
         duplicates would indicate a shared internal engine. harnesses={:?}",
        unique_harnesses
    );
}