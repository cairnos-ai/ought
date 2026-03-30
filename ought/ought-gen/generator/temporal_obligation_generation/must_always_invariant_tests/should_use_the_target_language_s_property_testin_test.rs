/// SHOULD use the target language's property testing library when available (e.g. `proptest` for Rust, `hypothesis` for Python, `fast-check` for JS)
#[test]
fn test_generator__temporal_obligation_generation__must_always_invariant_tests__should_use_the_target_language_s_property_testin() {
    use ought_gen::context::GenerationContext;
    use ought_gen::generator::Language;
    use ought_gen::providers::build_prompt;
    use ought_spec::{Clause, ClauseId, Keyword, Severity, SourceLocation, Temporal};
    use std::path::PathBuf;

    fn invariant_clause() -> Clause {
        Clause {
            id: ClauseId("gen::temporal::must_always_sorted".to_string()),
            keyword: Keyword::MustAlways,
            severity: Severity::Required,
            text: "the result list must always be sorted".to_string(),
            condition: None,
            otherwise: vec![],
            temporal: Some(Temporal::Invariant),
            hints: vec![],
            source_location: SourceLocation {
                file: PathBuf::from("spec.ought.md"),
                line: 30,
            },
            content_hash: "ghi789".to_string(),
        }
    }

    fn ctx(lang: Language) -> GenerationContext {
        GenerationContext {
            spec_context: None,
            source_files: vec![],
            schema_files: vec![],
            target_language: lang,
            verbose: false,
        }
    }

    let clause = invariant_clause();

    // Rust: should recommend proptest
    let rust_prompt = build_prompt(&clause, &ctx(Language::Rust));
    assert!(
        rust_prompt.contains("proptest"),
        "should_use_the_target_language_s_property_testin: \
         build_prompt for a Rust MUST ALWAYS clause should mention 'proptest'. \
         Prompt:\n{rust_prompt}"
    );

    // Python: should recommend hypothesis
    let py_prompt = build_prompt(&clause, &ctx(Language::Python));
    assert!(
        py_prompt.contains("hypothesis"),
        "should_use_the_target_language_s_property_testin: \
         build_prompt for a Python MUST ALWAYS clause should mention 'hypothesis'. \
         Prompt:\n{py_prompt}"
    );

    // TypeScript: should recommend fast-check
    let ts_prompt = build_prompt(&clause, &ctx(Language::TypeScript));
    assert!(
        ts_prompt.contains("fast-check"),
        "should_use_the_target_language_s_property_testin: \
         build_prompt for a TypeScript MUST ALWAYS clause should mention 'fast-check'. \
         Prompt:\n{ts_prompt}"
    );

    // JavaScript: should also recommend fast-check
    let js_prompt = build_prompt(&clause, &ctx(Language::JavaScript));
    assert!(
        js_prompt.contains("fast-check"),
        "should_use_the_target_language_s_property_testin: \
         build_prompt for a JavaScript MUST ALWAYS clause should mention 'fast-check'. \
         Prompt:\n{js_prompt}"
    );
}