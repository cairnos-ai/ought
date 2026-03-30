/// MUST define a `Generator` trait that all LLM providers implement
#[test]
fn test_generator__provider_abstraction__must_define_a_generator_trait_that_all_llm_providers_implement() {
    use std::path::PathBuf;
    use ought_gen::context::GenerationContext;
    use ought_gen::generator::{GeneratedTest, Generator, Language};
    use ought_spec::types::{Clause, ClauseId, Keyword, Severity, SourceLocation};

    // A local struct that implements Generator — proves the trait is object-safe
    // and that concrete types can fulfil the contract.
    struct MockProvider;
    impl Generator for MockProvider {
        fn generate(
            &self,
            clause: &Clause,
            context: &GenerationContext,
        ) -> anyhow::Result<GeneratedTest> {
            Ok(GeneratedTest {
                clause_id: clause.id.clone(),
                code: "// mock".to_string(),
                language: context.target_language,
                file_path: PathBuf::from("mock_test.rs"),
            })
        }
    }

    let clause = Clause {
        id: ClauseId("provider::must_work".to_string()),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: "work correctly".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("spec.ought.md"), line: 1 },
        content_hash: "h".to_string(),
    };
    let ctx = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };

    // Coerce to trait object — fails at compile time if Generator is not object-safe
    let provider: Box<dyn Generator> = Box::new(MockProvider);
    let result = provider.generate(&clause, &ctx);
    assert!(result.is_ok(), "Generator::generate must return Ok for a valid clause");

    let generated = result.unwrap();
    assert_eq!(
        generated.clause_id,
        ClauseId("provider::must_work".to_string()),
        "GeneratedTest must carry the originating clause id"
    );

    // Verify that the four shipped providers all satisfy Box<dyn Generator> by
    // constructing each through from_config — this is a compile + runtime check.
    use ought_gen::providers::from_config;
    assert!(
        from_config("claude", None).is_ok(),
        "ClaudeGenerator must be constructable via from_config"
    );
    assert!(
        from_config("openai", None).is_ok(),
        "OpenAiGenerator must be constructable via from_config"
    );
    assert!(
        from_config("ollama", Some("llama3")).is_ok(),
        "OllamaGenerator must be constructable via from_config"
    );
    assert!(
        from_config("/tmp/custom-llm", None).is_ok(),
        "CustomGenerator must be constructable via from_config"
    );
}