/// SHOULD include any code blocks immediately following a clause as "hints" attached to that clause
#[test]
fn test_parser__clause_ir__should_include_any_code_blocks_immediately_following_a_clause_as_hi() {
    use std::path::Path;
    use ought_spec::parser::Parser;

    // Code block immediately after a clause becomes a hint on that clause
    let md = concat!(
        "# Svc\n\n## API\n\n",
        "- **MUST** return valid JSON\n\n",
        "```json\n{\"status\": \"ok\"}\n```\n"
    );
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let clause = &spec.sections[0].clauses[0];
    assert_eq!(clause.hints.len(), 1);
    assert!(
        clause.hints[0].contains("status"),
        "hint should contain code block content"
    );

    // Clause with no following code block has empty hints
    let md_no_hint = "# Svc\n\n## API\n\n- **MUST** return valid JSON\n";
    let spec_no_hint =
        Parser::parse_string(md_no_hint, Path::new("test.ought.md")).expect("parse failed");
    assert!(spec_no_hint.sections[0].clauses[0].hints.is_empty());

    // Code block appearing before any clause (as prose) is NOT attached as a hint
    let md_prose_code = concat!(
        "# Svc\n\n## API\n\n",
        "Some introductory text.\n\n",
        "```json\n{\"example\": true}\n```\n\n",
        "- **MUST** return valid JSON\n"
    );
    let spec_prose =
        Parser::parse_string(md_prose_code, Path::new("test.ought.md")).expect("parse failed");
    assert!(
        spec_prose.sections[0].clauses[0].hints.is_empty(),
        "code block before clause should not become a hint"
    );
    // That code block ends up in prose instead
    assert!(
        spec_prose.sections[0].prose.contains("example"),
        "code block before clause should appear in section prose"
    );
}