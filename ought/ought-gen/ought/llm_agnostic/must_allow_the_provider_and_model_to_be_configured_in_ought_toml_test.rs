/// MUST allow the provider and model to be configured in `ought.toml`
#[test]
fn test_ought__llm_agnostic__must_allow_the_provider_and_model_to_be_configured_in_ought_toml() {
    use std::fs;
    use ought_spec::config::Config;

    let dir = std::env::temp_dir().join(format!("ought_llm_agnostic_config_test_{}", std::process::id()));
    fs::create_dir_all(&dir).expect("must create temp dir");

    let cases: &[(&str, Option<&str>)] = &[
        ("anthropic",  Some("claude-sonnet-4-6")),
        ("claude",     Some("claude-opus-4-6")),
        ("openai",     Some("gpt-4o")),
        ("chatgpt",    Some("gpt-4o-mini")),
        ("ollama",     Some("llama3")),
        ("ollama",     None),
        ("anthropic",  None),
        ("openai",     None),
    ];

    for (provider, model) in cases {
        let model_line = match model {
            Some(m) => format!("model = \"{m}\""),
            None => String::new(),
        };
        let toml = format!(
            "[project]\nname = \"test\"\n\n[generator]\nprovider = \"{provider}\"\n{model_line}\n\n[runner.rust]\ncommand = \"cargo test\"\ntest_dir = \"tests/\"\n"
        );
        let path = dir.join("ought.toml");
        fs::write(&path, &toml).expect("must write ought.toml");

        let config = Config::load(&path).unwrap_or_else(|e| {
            panic!("Config::load must succeed for provider=\"{provider}\" model={model:?}; got: {e}")
        });

        assert_eq!(
            config.generator.provider, *provider,
            "generator.provider must round-trip through ought.toml for \"{provider}\""
        );

        match model {
            Some(m) => assert_eq!(
                config.generator.model.as_deref(),
                Some(*m),
                "generator.model must round-trip for provider=\"{provider}\" model=\"{m}\""
            ),
            None => assert!(
                config.generator.model.is_none(),
                "generator.model must be None when omitted from ought.toml"
            ),
        }
    }

    fs::remove_dir_all(&dir).ok();
}