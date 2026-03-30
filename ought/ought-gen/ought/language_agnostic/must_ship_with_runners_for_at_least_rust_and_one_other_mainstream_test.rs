/// MUST ship with runners for at least Rust and one other mainstream language
///
/// The tool ships with built-in runners so users can begin testing popular
/// languages without writing custom configuration. At minimum, a Rust runner
/// and one additional mainstream language runner must be included out of the box.
#[test]
fn test_ought__language_agnostic__must_ship_with_runners_for_at_least_rust_and_one_other_mainstream() {
    // Rust runner is a hard MUST — it must always be present.
    let rust_runner = ought_run::runners::from_name("rust")
        .expect("Rust runner must be included — it is a hard MUST requirement");
    assert_eq!(rust_runner.name(), "rust",
        "Rust runner must report name \"rust\"");

    // At least one other mainstream language runner must ship alongside Rust.
    let other_mainstream = ["python", "typescript", "go", "javascript"];
    let available_others: Vec<&str> = other_mainstream
        .iter()
        .copied()
        .filter(|lang| ought_run::runners::from_name(lang).is_ok())
        .collect();

    assert!(
        !available_others.is_empty(),
        "at least one non-Rust mainstream runner must ship; tried {:?}, all returned Err",
        other_mainstream
    );

    // Verify the first available non-Rust runner has the correct name and is
    // distinct from the Rust runner.
    let other_lang = available_others[0];
    let other_runner = ought_run::runners::from_name(other_lang)
        .unwrap_or_else(|e| panic!("runner for {other_lang:?} must succeed: {e}"));

    assert_eq!(other_runner.name(), other_lang,
        "non-Rust runner must report the expected language name");
    assert_ne!(
        other_runner.name(),
        rust_runner.name(),
        "the second built-in runner must cover a different language than Rust"
    );

    // Bonus: confirm the full set of expected mainstream runners all resolve,
    // since the implementation ships with Rust, Python, TypeScript, and Go.
    let mandatory = ["rust", "python", "typescript", "go"];
    for lang in mandatory {
        let result = ought_run::runners::from_name(lang);
        assert!(
            result.is_ok(),
            "built-in runner for {lang:?} must be present; got: {:?}",
            result.err()
        );
    }
}