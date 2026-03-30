/// SHOULD inherit the parent's severity unless the OTHERWISE clause specifies its own keyword
#[test]
fn test_parser__contrary_to_duty_chains_otherwise__should_inherit_the_parent_s_severity_unless_the_otherwise_clause_sp() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::*;

    let md = r#"# Svc

## Graceful

- **MUST** return primary data
  - **OTHERWISE** return cached copy

- **SHOULD** include metadata
  - **OTHERWISE** omit metadata field
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec.sections[0].clauses;
    assert_eq!(clauses.len(), 2);

    // OTHERWISE under MUST inherits Required severity
    assert_eq!(clauses[0].severity, Severity::Required);
    assert_eq!(
        clauses[0].otherwise[0].severity,
        Severity::Required,
        "OTHERWISE under MUST must inherit Required severity"
    );

    // OTHERWISE under SHOULD inherits Recommended severity
    assert_eq!(clauses[1].severity, Severity::Recommended);
    assert_eq!(
        clauses[1].otherwise[0].severity,
        Severity::Recommended,
        "OTHERWISE under SHOULD must inherit Recommended severity"
    );
}