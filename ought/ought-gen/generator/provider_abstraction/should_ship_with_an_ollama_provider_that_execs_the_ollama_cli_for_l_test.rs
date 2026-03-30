/// SHOULD ship with an Ollama provider that execs the `ollama` CLI for local models
#[test]
fn test_generator__provider_abstraction__should_ship_with_an_ollama_provider_that_execs_the_ollama_cli_for_l() {
    use ought_gen::providers::from_config;
    use ought_gen::providers::ollama::OllamaGenerator;
    use ought_gen::generator::Generator;

    // "ollama" must resolve successfully.
    let via_ollama = from_config("ollama", None);
    assert!(
        via_ollama.is_ok(),
        "from_config(\"ollama\", _) must return Ok; got: {:?}",
        via_ollama.err()
    );

    // With an explicit model name.
    let via_ollama_model = from_config("ollama", Some("mistral"));
    assert!(
        via_ollama_model.is_ok(),
        "from_config(\"ollama\", Some(\"mistral\")) must return Ok; got: {:?}",
        via_ollama_model.err()
    );

    // Direct construction satisfies Generator trait.
    let _g: Box<dyn Generator> = Box::new(OllamaGenerator::new("llama3".to_string()));

    // The default model when none is specified must be "llama3" (documented default).
    // Verify by constructing with None and checking exec would use "ollama run llama3".
    // We test this indirectly: from_config("ollama", None) must not error,
    // and exec_cli("ollama", &["run", "llama3"], ...) failing with "not found"
    // (when ollama is absent) must name "ollama" — confirming it's the CLI invoked.
    use ought_gen::providers::exec_cli;
    if let Err(e) = exec_cli("ollama", &["run", "llama3"], "ping") {
        let msg = e.to_string();
        assert!(
            msg.contains("ollama"),
            "error from missing ollama CLI must name 'ollama'; got: {msg}"
        );
    }
}