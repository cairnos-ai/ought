/// SHOULD support provider-specific configuration in `ought.toml` under `[generator]`
#[test]
fn test_generator__provider_abstraction__should_support_provider_specific_configuration_in_ought_toml_under() {
    use ought_gen::providers::from_config;

    // from_config is the bridge between ought.toml [generator] settings and the
    // concrete provider. Verify that provider + model combinations all construct
    // without error — these reflect what a user can write in ought.toml.

    let cases: &[(&str, Option<&str>)] = &[
        ("claude",     Some("claude-sonnet-4-6")),
        ("anthropic",  Some("claude-opus-4-6")),
        ("openai",     Some("gpt-4o")),
        ("chatgpt",    Some("gpt-4o-mini")),
        ("ollama",     Some("mistral")),
        ("ollama",     Some("codellama")),
        ("claude",     None),
        ("openai",     None),
        ("ollama",     None),
    ];

    for (provider, model) in cases {
        let result = from_config(provider, *model);
        assert!(
            result.is_ok(),
            "[generator] provider=\"{provider}\" model={model:?} must construct without error; got: {:?}",
            result.err()
        );
    }
}