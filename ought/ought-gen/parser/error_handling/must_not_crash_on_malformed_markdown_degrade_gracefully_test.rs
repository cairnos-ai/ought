/// MUST NOT crash on malformed markdown — degrade gracefully
#[test]
fn test_parser__error_handling__must_not_crash_on_malformed_markdown_degrade_gracefully() {
    // Empty document — must return a default Spec, not panic or error.
    let result = Parser::parse_string("", Path::new("empty.ought.md"));
    assert!(result.is_ok(), "empty input must parse without error");
    let spec = result.unwrap();
    assert_eq!(spec.name, "Untitled", "empty doc must default to 'Untitled'");
    assert!(spec.sections.is_empty(), "empty doc must have no sections");

    // Whitespace-only input.
    let result = Parser::parse_string("   \n\n   \n", Path::new("ws.ought.md"));
    assert!(result.is_ok(), "whitespace-only input must not fail");

    // H1-only, no sections.
    let result = Parser::parse_string("# Just a Title\n", Path::new("title_only.ought.md"));
    assert!(result.is_ok(), "H1-only doc must not fail");
    assert_eq!(result.unwrap().name, "Just a Title");

    // Missing H1 — valid clauses inside a section must still be parsed.
    let result = Parser::parse_string(
        "## Section\n\n- **MUST** do the thing\n",
        Path::new("no_h1.ought.md"),
    );
    assert!(result.is_ok(), "missing H1 must not crash the parser");
    let spec = result.unwrap();
    assert!(!spec.sections.is_empty(), "section must be parsed even without H1");
    assert_eq!(
        spec.sections[0].clauses.len(),
        1,
        "clause must be parsed when H1 is absent"
    );

    // Unclosed bold delimiter — CommonMark treats it as literal text.
    // The valid clause after the broken item must still be collected.
    let result = Parser::parse_string(
        "# Svc\n\n## S\n\n- **MUST unclosed bold marker\n- **MUST** valid after unclosed\n",
        Path::new("unclosed_bold.ought.md"),
    );
    // Must not panic; valid clause after the bad item must survive.
    if let Ok(spec) = result {
        assert!(
            !spec.sections[0].clauses.is_empty(),
            "valid clause after unclosed bold must be parsed"
        );
    }

    // Unclosed fenced code block.
    let _ = Parser::parse_string(
        "# Svc\n\n## S\n\n- **MUST** clause\n\n```\nno closing fence\n",
        Path::new("unclosed_fence.ought.md"),
    );

    // Markdown escape sequences and unusual characters inside clause text.
    let _ = Parser::parse_string(
        "# Svc\n\n## S\n\n- **MUST** handle \\* \\` \\[ edge chars\n",
        Path::new("escapes.ought.md"),
    );

    // Deeply nested lists beyond normal spec usage.
    let deep = format!(
        "# Svc\n\n## S\n\n- **MUST** top\n{}- nested\n{}- deeper\n{}- deepest\n",
        "  ".repeat(1),
        "  ".repeat(2),
        "  ".repeat(3),
    );
    let _ = Parser::parse_string(&deep, Path::new("deep_nesting.ought.md"));

    // Section heading with no body.
    let result = Parser::parse_string("# Svc\n\n## Empty Section\n\n## Next\n\n- **MUST** clause\n", Path::new("empty_sec.ought.md"));
    assert!(result.is_ok(), "empty section followed by valid section must not fail");
    let spec = result.unwrap();
    assert_eq!(
        spec.sections.iter().map(|s| s.clauses.len()).sum::<usize>(),
        1,
        "clause in section after an empty section must still be parsed"
    );
}