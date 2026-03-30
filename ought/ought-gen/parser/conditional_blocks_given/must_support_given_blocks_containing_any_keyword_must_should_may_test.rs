/// MUST support GIVEN blocks containing any keyword (MUST, SHOULD, MAY, WONT, OTHERWISE, etc.)
#[test]
fn test_parser__conditional_blocks_given__must_support_given_blocks_containing_any_keyword_must_should_may() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::*;

    fn parse(md: &str) -> Spec {
        Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed")
    }

    let md = r#"# Svc

## Behaviour

- **GIVEN** the feature flag is enabled:
  - **MUST** activate the new code path
  - **MUST NOT** fall back to the legacy path
  - **SHOULD** emit a telemetry event
  - **SHOULD NOT** cache the result
  - **MAY** log additional debug info
  - **WONT** support IE11 in this mode
"#;
    let spec = parse(md);
    let clauses = &spec.sections[0].clauses;
    assert_eq!(clauses.len(), 6);

    let keywords: Vec<Keyword> = clauses.iter().map(|c| c.keyword).collect();
    assert!(keywords.contains(&Keyword::Must),      "MUST inside GIVEN");
    assert!(keywords.contains(&Keyword::MustNot),   "MUST NOT inside GIVEN");
    assert!(keywords.contains(&Keyword::Should),    "SHOULD inside GIVEN");
    assert!(keywords.contains(&Keyword::ShouldNot), "SHOULD NOT inside GIVEN");
    assert!(keywords.contains(&Keyword::May),       "MAY inside GIVEN");
    assert!(keywords.contains(&Keyword::Wont),      "WONT inside GIVEN");

    // All inherit the condition
    for clause in clauses {
        assert_eq!(
            clause.condition.as_deref(),
            Some("the feature flag is enabled:"),
            "clause '{}' missing condition", clause.text
        );
    }
}