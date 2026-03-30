/// MUST assign severity levels: MUST/MUST NOT/MUST ALWAYS/MUST BY = required,
/// SHOULD/SHOULD NOT = recommended, MAY = optional, WONT = negative-confirmation
#[test]
fn test_parser__keywords__must_assign_severity_levels_must_must_not_must_always_must_by_req() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::*;

    let md = r#"# Svc

## All Keywords

- **MUST** do something required
- **MUST NOT** skip something required
- **MUST ALWAYS** hold an invariant
- **MUST BY 1s** finish within deadline
- **SHOULD** follow recommendation
- **SHOULD NOT** violate recommendation
- **MAY** use optional feature
- **WONT** implement out-of-scope thing
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec.sections[0].clauses;
    assert_eq!(clauses.len(), 8);

    // Required group
    assert_eq!(clauses[0].severity, Severity::Required,   "MUST → Required");
    assert_eq!(clauses[1].severity, Severity::Required,   "MUST NOT → Required");
    assert_eq!(clauses[2].severity, Severity::Required,   "MUST ALWAYS → Required");
    assert_eq!(clauses[3].severity, Severity::Required,   "MUST BY → Required");

    // Recommended group
    assert_eq!(clauses[4].severity, Severity::Recommended, "SHOULD → Recommended");
    assert_eq!(clauses[5].severity, Severity::Recommended, "SHOULD NOT → Recommended");

    // Optional
    assert_eq!(clauses[6].severity, Severity::Optional,   "MAY → Optional");

    // Negative confirmation
    assert_eq!(clauses[7].severity, Severity::NegativeConfirmation, "WONT → NegativeConfirmation");
}