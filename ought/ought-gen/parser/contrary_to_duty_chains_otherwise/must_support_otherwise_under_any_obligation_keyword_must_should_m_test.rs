/// MUST support OTHERWISE under any obligation keyword (MUST, SHOULD, MUST ALWAYS, MUST BY)
#[test]
fn test_parser__contrary_to_duty_chains_otherwise__must_support_otherwise_under_any_obligation_keyword_must_should_m() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::*;

    let md = r#"# Svc

## Obligations

- **MUST** respond within 200ms
  - **OTHERWISE** return HTTP 503

- **SHOULD** include debug headers
  - **OTHERWISE** omit debug headers silently

- **MUST ALWAYS** maintain a live connection
  - **OTHERWISE** reconnect with exponential backoff

- **MUST BY 5s** complete the handshake
  - **OTHERWISE** abort and log timeout
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec.sections[0].clauses;
    assert_eq!(clauses.len(), 4);

    assert_eq!(clauses[0].keyword, Keyword::Must);
    assert_eq!(clauses[0].otherwise.len(), 1, "MUST must support OTHERWISE");

    assert_eq!(clauses[1].keyword, Keyword::Should);
    assert_eq!(clauses[1].otherwise.len(), 1, "SHOULD must support OTHERWISE");

    assert_eq!(clauses[2].keyword, Keyword::MustAlways);
    assert_eq!(clauses[2].otherwise.len(), 1, "MUST ALWAYS must support OTHERWISE");

    assert_eq!(clauses[3].keyword, Keyword::MustBy);
    assert_eq!(clauses[3].otherwise.len(), 1, "MUST BY must support OTHERWISE");
}