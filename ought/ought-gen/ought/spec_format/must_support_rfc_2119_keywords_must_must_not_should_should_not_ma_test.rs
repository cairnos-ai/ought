/// MUST support RFC 2119 keywords (**MUST**, **MUST NOT**, **SHOULD**,
/// **SHOULD NOT**, **MAY**) as deontic operators.
#[test]
fn test_ought__spec_format__must_support_rfc_2119_keywords_must_must_not_should_should_not_ma() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::{Keyword, Severity};

    let md = r#"# Spec Format

## Rules

- **MUST** always validate input
- **MUST NOT** store plaintext passwords
- **SHOULD** log all access attempts
- **SHOULD NOT** cache sensitive data
- **MAY** support optional audit trails
"#;

    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec.sections[0].clauses;

    assert_eq!(clauses.len(), 5);

    // MUST → Required
    assert_eq!(clauses[0].keyword, Keyword::Must);
    assert_eq!(clauses[0].severity, Severity::Required);
    assert!(clauses[0].text.contains("validate input"));

    // MUST NOT → Required
    assert_eq!(clauses[1].keyword, Keyword::MustNot);
    assert_eq!(clauses[1].severity, Severity::Required);
    assert!(clauses[1].text.contains("plaintext passwords"));

    // SHOULD → Recommended
    assert_eq!(clauses[2].keyword, Keyword::Should);
    assert_eq!(clauses[2].severity, Severity::Recommended);
    assert!(clauses[2].text.contains("log all access"));

    // SHOULD NOT → Recommended
    assert_eq!(clauses[3].keyword, Keyword::ShouldNot);
    assert_eq!(clauses[3].severity, Severity::Recommended);
    assert!(clauses[3].text.contains("cache sensitive"));

    // MAY → Optional
    assert_eq!(clauses[4].keyword, Keyword::May);
    assert_eq!(clauses[4].severity, Severity::Optional);
    assert!(clauses[4].text.contains("audit trails"));
}