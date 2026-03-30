/// MUST parse keywords case-insensitively but require them to appear in bold (`**MUST**`, `**GIVEN**`, etc.)
#[test]
fn test_parser__keywords__must_parse_keywords_case_insensitively_but_require_them_to_appear() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::*;

    // All casing variants in bold — all must be recognised
    let md_bold = r#"# Svc

## Rules

- **must** do something lowercase
- **Must** do something title-case
- **MUST** do something uppercase
- **must not** reject lowercase compound
- **Should** recommend title-case
- **may** allow lowercase optional
- **wont** refuse lowercase
"#;
    let spec = Parser::parse_string(md_bold, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec.sections[0].clauses;
    assert_eq!(clauses.len(), 7, "all bold keyword variants must be parsed regardless of case");
    assert_eq!(clauses[0].keyword, Keyword::Must);
    assert_eq!(clauses[1].keyword, Keyword::Must);
    assert_eq!(clauses[2].keyword, Keyword::Must);
    assert_eq!(clauses[3].keyword, Keyword::MustNot);
    assert_eq!(clauses[4].keyword, Keyword::Should);
    assert_eq!(clauses[5].keyword, Keyword::May);
    assert_eq!(clauses[6].keyword, Keyword::Wont);

    // Non-bold keywords — must produce zero clauses
    let md_bare = r#"# Svc

## Prose

You must restart the service after upgrading.
The system should validate inputs.
"#;
    let spec2 = Parser::parse_string(md_bare, Path::new("test.ought.md")).expect("parse failed");
    assert!(
        spec2.sections[0].clauses.is_empty(),
        "bare (non-bold) keywords in prose must not produce clauses"
    );
}