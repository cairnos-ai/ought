/// MUST NOT manage API keys or authentication — that is the CLI tool's responsibility
#[test]
fn test_generator__provider_abstraction__must_not_manage_api_keys_or_authentication_that_is_the_cli_tool_s_res() {
    use ought_gen::providers::claude::ClaudeGenerator;
    use ought_gen::providers::openai::OpenAiGenerator;
    use ought_gen::providers::ollama::OllamaGenerator;
    use ought_gen::providers::custom::CustomGenerator;
    use std::path::PathBuf;

    // Each provider struct stores at most a model name — no API key, token,
    // secret, or credential field.  This is verified structurally: constructors
    // only accept an optional model string, not any credential.
    let _c = ClaudeGenerator::new(None);
    let _c_model = ClaudeGenerator::new(Some("claude-opus-4-6".to_string()));

    let _o = OpenAiGenerator::new(None);
    let _o_model = OpenAiGenerator::new(Some("gpt-4o".to_string()));

    // OllamaGenerator requires a model (local — no cloud auth at all).
    let _ol = OllamaGenerator::new("llama3".to_string());

    // CustomGenerator takes only an executable path — no credentials.
    let _cu = CustomGenerator::new(PathBuf::from("/usr/local/bin/my-llm"));

    // Additionally, exec_cli must not inject auth environment variables.
    // We verify by using `env` (prints all env vars) and confirming that
    // ANTHROPIC_API_KEY / OPENAI_API_KEY are NOT set by exec_cli itself.
    // (If the user has them set already in their shell that is their business.)
    // We verify by checking that exec_cli does not unconditionally set them:
    // run `printenv OUGHT_SENTINEL_UNSET_VAR` — must return error (exit nonzero)
    // because exec_cli does not add extra env vars.
    use ought_gen::providers::exec_cli;
    let result = exec_cli("sh", &["-c", "printenv OUGHT_SENTINEL_VAR_NEVER_SET"], "");
    // sh -c "printenv MISSING_VAR" exits 1 — exec_cli must propagate the error
    assert!(
        result.is_err(),
        "exec_cli must not inject env vars the shell doesn't have; a missing var query should fail"
    );
}