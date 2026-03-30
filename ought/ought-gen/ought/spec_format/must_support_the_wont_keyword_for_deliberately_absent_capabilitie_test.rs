/// MUST support the **WONT** keyword for deliberately absent capabilities.
#[test]
fn test_ought__spec_format__must_support_the_wont_keyword_for_deliberately_absent_capabilitie() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::{Keyword, Severity};

    let md = r#"# Spec Format

## Scope Exclusions

- **WONT** support OAuth 1.0 due to known security flaws
- **WONT** provide a SOAP API
"#;

    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec.sections[0].clauses;

    assert_eq!(clauses.len(), 2);

    // Both WONT clauses get NegativeConfirmation severity
    assert_eq!(clauses[0].keyword, Keyword::Wont);
    assert_eq!(clauses[0].severity, Severity::NegativeConfirmation);
    assert!(clauses[0].text.contains("OAuth 1.0"));

    assert_eq!(clauses[1].keyword, Keyword::Wont);
    assert_eq!(clauses[1].severity, Severity::NegativeConfirmation);
    assert!(clauses[1].text.contains("SOAP API"));
}