/// MUST be agnostic to the programming language of the project under test
///
/// Ought selects a runner via a plain string key in `ought.toml` — it never
/// inspects or depends on the programming language of the project under test.
/// All language runners implement the same `Runner` trait and are treated
/// identically by the orchestrator.
#[test]
fn test_ought__language_agnostic__must_be_agnostic_to_the_programming_language_of_the_project_under() {
    // Runners for completely different languages are created identically —
    // by passing a string name. No project-level language detection occurs.
    let languages = ["rust", "python", "typescript", "go"];
    for lang in languages {
        let runner = ought_run::runners::from_name(lang)
            .unwrap_or_else(|e| panic!("from_name({lang:?}) failed: {e}"));
        // Every runner exposes exactly the same interface regardless of language.
        let name: &str = runner.name();
        let _available: bool = runner.is_available();
        assert_eq!(
            name, lang,
            "runner created for {lang:?} must report that same name back"
        );
    }

    // An unrecognised name returns an error — the tool does not silently
    // invent a runner for an unknown language.
    let unknown = ought_run::runners::from_name("cobol");
    assert!(
        unknown.is_err(),
        "from_name with an unrecognised language must return Err, not a silent default"
    );

    // All supported runner names are distinct — each language is treated as its own
    // independent target with no shared state or cross-language coupling.
    let runners: Vec<_> = languages
        .iter()
        .map(|lang| ought_run::runners::from_name(lang).unwrap())
        .collect();
    let names: std::collections::HashSet<&str> = runners.iter().map(|r| r.name()).collect();
    assert_eq!(
        names.len(),
        languages.len(),
        "all supported runner names must be distinct; found duplicates in {:?}",
        runners.iter().map(|r| r.name()).collect::<Vec<_>>()
    );
}