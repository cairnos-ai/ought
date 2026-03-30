/// MUST parse `context:` as free-text context for the LLM
#[test]
fn test_parser__context_metadata__must_parse_context_as_free_text_context_for_the_llm() {
    let md = r#"# MySpec

context: Handles user authentication and session management for the web API

## Rules

- **MUST** do something
"#;
    let spec = parse(md);
    let ctx = spec
        .metadata
        .context
        .expect("`context:` field should be Some");
    // The full free-text value must be preserved verbatim
    assert_eq!(
        ctx,
        "Handles user authentication and session management for the web API"
    );
}