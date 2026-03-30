/// MUST recognize RFC 2119 keywords: MUST, MUST NOT, SHOULD, SHOULD NOT, MAY
#[test]
fn test_parser__keywords__must_recognize_rfc_2119_keywords_must_must_not_should_should_not() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::*;

    let md = r#"# Svc

## Rules

- **MUST** perform authentication
- **MUST NOT** expose internal errors
- **SHOULD** log failed attempts
- **SHOULD NOT** cache sensitive tokens
- **MAY** support remember-me sessions
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec.sections[0].clauses;
    assert_eq!(clauses.len(), 5);
    assert_eq!(clauses[0].keyword, Keyword::Must);
    assert!(clauses[0].text.contains("authentication"));
    assert_eq!(clauses[1].keyword, Keyword::MustNot);
    assert!(clauses[1].text.contains("internal errors"));
    assert_eq!(clauses[2].keyword, Keyword::Should);
    assert!(clauses[2].text.contains("failed attempts"));
    assert_eq!(clauses[3].keyword, Keyword::ShouldNot);
    assert!(clauses[3].text.contains("sensitive tokens"));
    assert_eq!(clauses[4].keyword, Keyword::May);
    assert!(clauses[4].text.contains("remember-me"));
}