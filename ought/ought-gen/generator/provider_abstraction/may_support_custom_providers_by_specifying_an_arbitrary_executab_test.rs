/// MAY support custom providers by specifying an arbitrary executable in `ought.toml`
#[test]
fn test_generator__provider_abstraction__may_support_custom_providers_by_specifying_an_arbitrary_executab() {
    use ought_gen::providers::from_config;
    use ought_gen::providers::exec_cli;
    use ought_gen::generator::Generator;

    // Any string that looks like a path (contains '/') or is not a known keyword
    // should be treated as a custom executable path rather than a named provider.
    let custom_cases = &[
        "/usr/local/bin/my-llm",
        "./scripts/generate.sh",
        "/opt/custom/llm-wrapper",
    ];
    for path in custom_cases {
        let result = from_config(path, None);
        assert!(
            result.is_ok(),
            "from_config with path '{}' must return Ok (custom provider); got: {:?}",
            path,
            result.err()
        );
        // Verify it satisfies the trait.
        let _g: Box<dyn Generator> = result.unwrap();
    }

    // A custom provider must pass the prompt via stdin, just like built-in providers.
    // Verify by using `cat` as the custom executable — it echoes stdin to stdout.
    let prompt = "#[test] fn test_custom() { assert!(true); }";
    let output = exec_cli("cat", &[], prompt).expect("cat must succeed as a custom provider stand-in");
    assert!(
        output.contains("test_custom"),
        "custom provider output must be captured from stdout; got: {output}"
    );

    // An unknown non-path token (no slash) that is not a known provider keyword
    // must also fall through to the custom path rather than error at construction.
    let result = from_config("unknown-provider-token", None);
    assert!(
        result.is_ok(),
        "unrecognised provider name must be treated as a custom executable path, not a construction error"
    );
}