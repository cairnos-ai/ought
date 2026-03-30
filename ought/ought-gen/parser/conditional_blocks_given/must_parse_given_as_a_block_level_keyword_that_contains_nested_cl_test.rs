/// MUST parse `**GIVEN**` as a block-level keyword that contains nested clauses
#[test]
fn test_parser__conditional_blocks_given__must_parse_given_as_a_block_level_keyword_that_contains_nested_cl() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::*;

    fn parse(md: &str) -> Spec {
        Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed")
    }

    let md = r#"# Svc

## Access

- **GIVEN** the user is authenticated:
  - **MUST** return their profile data
  - **SHOULD** include last-login timestamp
"#;
    let spec = parse(md);
    let clauses = &spec.sections[0].clauses;
    // GIVEN is a block-level grouping; its two children become the clauses
    assert_eq!(clauses.len(), 2);
    assert_eq!(clauses[0].keyword, Keyword::Must);
    assert!(clauses[0].text.contains("return their profile data"));
    assert_eq!(clauses[1].keyword, Keyword::Should);
    assert!(clauses[1].text.contains("include last-login timestamp"));
}