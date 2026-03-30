/// SHOULD include surrounding prose/markdown in the clause's context field for the LLM
#[test]
fn test_parser__clause_ir__should_include_surrounding_prose_markdown_in_the_clause_s_context_f() {
    use std::path::Path;
    use ought_spec::parser::Parser;

    // Prose surrounding clauses is preserved in the section's prose field (LLM context)
    let md = concat!(
        "# Svc\n\n## Auth\n\n",
        "This section describes the authentication flow.\n",
        "Tokens are signed with RS256.\n\n",
        "- **MUST** validate token signature\n\n",
        "Additional notes about expiry edge cases.\n"
    );
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let section = &spec.sections[0];

    // Section prose is non-empty and contains the surrounding text
    assert!(!section.prose.is_empty());
    assert!(
        section.prose.contains("authentication flow"),
        "prose should include text before the clause"
    );
    assert!(
        section.prose.contains("RS256"),
        "prose should include all surrounding markdown content"
    );

    // The clause itself is still present alongside the prose
    assert_eq!(section.clauses.len(), 1);
    assert!(section.clauses[0].text.contains("validate token signature"));

    // A section containing only clauses and no surrounding text has empty prose
    let md_no_prose = "# Svc\n\n## Rules\n\n- **MUST** do something\n";
    let spec_no_prose =
        Parser::parse_string(md_no_prose, Path::new("test.ought.md")).expect("parse failed");
    assert!(spec_no_prose.sections[0].prose.is_empty());
}