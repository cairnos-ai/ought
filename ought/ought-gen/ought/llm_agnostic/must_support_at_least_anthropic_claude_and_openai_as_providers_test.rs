/// MUST support at least Anthropic (Claude) and OpenAI as providers
#[test]
fn test_ought__llm_agnostic__must_support_at_least_anthropic_claude_and_openai_as_providers() {
    use ought_gen::generator::Generator;
    use ought_gen::providers::{from_config, claude::ClaudeGenerator, openai::OpenAiGenerator};

    // All recognised aliases for Anthropic must construct without error.
    for alias in &["anthropic", "claude", "ANTHROPIC", "Claude"] {
        let result = from_config(alias, None);
        assert!(
            result.is_ok(),
            "from_config(\"{alias}\", None) must return Ok for Anthropic alias; got: {:?}",
            result.err()
        );
    }

    // All recognised aliases for OpenAI must construct without error.
    for alias in &["openai", "chatgpt", "OpenAI", "ChatGPT"] {
        let result = from_config(alias, None);
        assert!(
            result.is_ok(),
            "from_config(\"{alias}\", None) must return Ok for OpenAI alias; got: {:?}",
            result.err()
        );
    }

    // Each provider must also accept an explicit model name.
    assert!(
        from_config("anthropic", Some("claude-sonnet-4-6")).is_ok(),
        "Anthropic provider must accept a model override"
    );
    assert!(
        from_config("openai", Some("gpt-4o")).is_ok(),
        "OpenAI provider must accept a model override"
    );

    // Both provider types must satisfy Box<dyn Generator> (object safety check).
    let _: Box<dyn Generator> = Box::new(ClaudeGenerator::new(None));
    let _: Box<dyn Generator> = Box::new(ClaudeGenerator::new(Some("claude-opus-4-6".to_string())));
    let _: Box<dyn Generator> = Box::new(OpenAiGenerator::new(None));
    let _: Box<dyn Generator> = Box::new(OpenAiGenerator::new(Some("gpt-4o-mini".to_string())));
}