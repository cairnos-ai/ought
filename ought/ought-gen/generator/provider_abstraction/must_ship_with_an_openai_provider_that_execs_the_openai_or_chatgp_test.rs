/// MUST ship with an OpenAI provider that execs the `openai` or `chatgpt` CLI
#[test]
fn test_generator__provider_abstraction__must_ship_with_an_openai_provider_that_execs_the_openai_or_chatgp() {
    use ought_gen::providers::from_config;
    use ought_gen::providers::openai::OpenAiGenerator;
    use ought_gen::generator::Generator;

    // Both "openai" and "chatgpt" must resolve without error.
    let via_openai = from_config("openai", None);
    assert!(
        via_openai.is_ok(),
        "from_config(\"openai\", _) must return Ok; got: {:?}",
        via_openai.err()
    );

    let via_chatgpt = from_config("chatgpt", None);
    assert!(
        via_chatgpt.is_ok(),
        "from_config(\"chatgpt\", _) must return Ok; got: {:?}",
        via_chatgpt.err()
    );

    // Direct construction must also satisfy the trait.
    let _g: Box<dyn Generator> = Box::new(OpenAiGenerator::new(None));
    let _with_model: Box<dyn Generator> = Box::new(OpenAiGenerator::new(Some("gpt-4o".to_string())));
}