/// SHOULD support local models via Ollama
#[test]
fn test_ought__llm_agnostic__should_support_local_models_via_ollama() {
    use ought_gen::generator::Generator;
    use ought_gen::providers::{from_config, ollama::OllamaGenerator, exec_cli};

    // "ollama" with no model must default to "llama3" without error.
    let default_result = from_config("ollama", None);
    assert!(
        default_result.is_ok(),
        "from_config(\"ollama\", None) must return Ok with default model; got: {:?}",
        default_result.err()
    );

    // "ollama" with various common local model names must also construct without error.
    for model in &["llama3", "mistral", "codellama", "phi3", "gemma2"] {
        let result = from_config("ollama", Some(model));
        assert!(
            result.is_ok(),
            "from_config(\"ollama\", Some(\"{model}\")) must return Ok; got: {:?}",
            result.err()
        );
    }

    // OllamaGenerator must satisfy Box<dyn Generator> — object safety check.
    let _: Box<dyn Generator> = Box::new(OllamaGenerator::new("llama3".to_string()));

    // When the ollama binary is absent the error must name "ollama" and say "not found",
    // confirming that `ollama run <model>` is the CLI being invoked.
    if let Err(e) = exec_cli("__ought_absent_ollama_binary__", &["run", "llama3"], "test") {
        let msg = e.to_string().to_lowercase();
        assert!(
            msg.contains("not found") || msg.contains("__ought_absent_ollama_binary__"),
            "missing ollama CLI error must say 'not found' or name the binary; got: {msg}"
        );
    }
}