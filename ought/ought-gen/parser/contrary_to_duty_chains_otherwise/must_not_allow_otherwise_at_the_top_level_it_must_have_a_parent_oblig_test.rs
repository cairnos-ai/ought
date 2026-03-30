/// MUST NOT allow OTHERWISE at the top level (it must have a parent obligation)
#[test]
fn test_parser__contrary_to_duty_chains_otherwise__must_not_allow_otherwise_at_the_top_level_it_must_have_a_parent_oblig() {
    use std::path::Path;
    use ought_spec::parser::Parser;

    let md = r#"# Svc

## Section

- **OTHERWISE** this has no parent obligation
"#;
    let result = Parser::parse_string(md, Path::new("test.ought.md"));
    assert!(
        result.is_err(),
        "OTHERWISE at the top level without a parent obligation must be a parse error"
    );
}