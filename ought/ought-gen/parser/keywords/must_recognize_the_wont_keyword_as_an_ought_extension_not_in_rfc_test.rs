/// MUST recognize the WONT keyword as an ought extension (not in RFC 2119)
#[test]
fn test_parser__keywords__must_recognize_the_wont_keyword_as_an_ought_extension_not_in_rfc() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::*;

    let md = r#"# Svc

## Scope

- **WONT** support OAuth 1.0
- **WONT** implement SOAP endpoints
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec.sections[0].clauses;
    assert_eq!(clauses.len(), 2);
    assert_eq!(clauses[0].keyword, Keyword::Wont);
    assert!(clauses[0].text.contains("OAuth 1.0"));
    assert_eq!(clauses[1].keyword, Keyword::Wont);
    assert!(clauses[1].text.contains("SOAP"));
}