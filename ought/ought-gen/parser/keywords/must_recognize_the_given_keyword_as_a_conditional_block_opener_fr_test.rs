/// MUST recognize the GIVEN keyword as a conditional block opener (from deontic logic)
#[test]
fn test_parser__keywords__must_recognize_the_given_keyword_as_a_conditional_block_opener_fr() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::*;

    let md = r#"# Svc

## Access

- **GIVEN** the user holds an admin role:
  - **MUST** allow deletion of any record
  - **MUST NOT** expose other tenants' data
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec.sections[0].clauses;
    // GIVEN groups its children; they become top-level clauses with the condition set
    assert_eq!(clauses.len(), 2);
    assert_eq!(clauses[0].keyword, Keyword::Must);
    let condition = clauses[0].condition.as_deref().expect("condition must be present");
    assert!(condition.contains("admin role"));
    assert_eq!(clauses[1].keyword, Keyword::MustNot);
    assert_eq!(clauses[1].condition, clauses[0].condition,
        "all children of the same GIVEN block share the same condition");
}