/// MUST NOT allow OTHERWISE under MAY, WONT, or GIVEN (only under obligations that can be violated)
#[test]
fn test_parser__contrary_to_duty_chains_otherwise__must_not_allow_otherwise_under_may_wont_or_given_only_under_obligatio() {
    use std::path::Path;
    use ought_spec::parser::Parser;

    // OTHERWISE under MAY is invalid — MAY cannot be violated
    let md_may = r#"# Svc

## Section

- **MAY** use optional feature
  - **OTHERWISE** do nothing
"#;
    assert!(
        Parser::parse_string(md_may, Path::new("test.ought.md")).is_err(),
        "OTHERWISE under MAY must be a parse error"
    );

    // OTHERWISE under WONT is invalid — WONT is a negative confirmation, not a violable obligation
    let md_wont = r#"# Svc

## Section

- **WONT** implement feature X
  - **OTHERWISE** implement feature Y instead
"#;
    assert!(
        Parser::parse_string(md_wont, Path::new("test.ought.md")).is_err(),
        "OTHERWISE under WONT must be a parse error"
    );

    // OTHERWISE under GIVEN is invalid — GIVEN is a grouping construct, not a violable obligation
    let md_given = r#"# Svc

## Section

- **GIVEN** some condition:
  - **OTHERWISE** fallback without an obligation
"#;
    assert!(
        Parser::parse_string(md_given, Path::new("test.ought.md")).is_err(),
        "OTHERWISE under GIVEN must be a parse error"
    );
}