/// MUST preserve all non-clause markdown as documentation (context for the LLM, readable for humans)
#[test]
fn test_parser__spec_file_structure__must_preserve_all_non_clause_markdown_as_documentation_context_fo() {
    let md = r#"# Svc

## Security

This section describes the security requirements for the service.
Access control is enforced at the API boundary.

- Background: authentication uses bearer tokens
- Note: tokens expire after 24 hours

- **MUST** reject unauthenticated requests
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let section = &spec.sections[0];

    // The MUST clause is parsed correctly
    assert_eq!(section.clauses.len(), 1);
    assert_eq!(section.clauses[0].keyword, Keyword::Must);

    // All surrounding non-clause markdown is preserved in section.prose
    assert!(
        !section.prose.is_empty(),
        "non-clause markdown must be preserved in section.prose"
    );
    assert!(
        section.prose.contains("security requirements") || section.prose.contains("API boundary"),
        "paragraph text must appear in section.prose"
    );
}