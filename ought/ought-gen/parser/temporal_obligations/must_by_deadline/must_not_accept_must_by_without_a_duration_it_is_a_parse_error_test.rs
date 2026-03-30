/// MUST NOT accept MUST BY without a duration (it is a parse error)
///
/// Verifies that a `**MUST BY**` with no following duration token is rejected
/// by the parser and surfaced as an error, not silently swallowed or turned into
/// a bare `MUST` clause.
#[test]
fn test_parser__temporal_obligations__must_by_deadline__must_not_accept_must_by_without_a_duration_it_is_a_parse_error() {
    use std::path::Path;
    use ought_spec::parser::Parser;

    // No duration: "MUST BY" ends the bold span immediately
    let md_no_duration = r#"# Svc

## Rules

- **MUST BY** respond to every request
"#;
    let result = Parser::parse_string(md_no_duration, Path::new("test.ought.md"));
    assert!(
        result.is_err(),
        "parsing MUST BY with no duration should return Err, got Ok"
    );

    // Also check a variant where the bold span closes before the duration
    let md_early_close = r#"# Svc

## Rules

- **MUST BY** 30s respond to every request
"#;
    let result2 = Parser::parse_string(md_early_close, Path::new("test.ought.md"));
    assert!(
        result2.is_err(),
        "MUST BY with duration outside the bold span should return Err, got Ok"
    );
}