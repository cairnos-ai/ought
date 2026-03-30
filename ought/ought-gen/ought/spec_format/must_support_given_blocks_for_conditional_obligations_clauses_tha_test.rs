/// MUST support **GIVEN** blocks for conditional obligations (clauses that
/// only apply when a precondition holds).
#[test]
fn test_ought__spec_format__must_support_given_blocks_for_conditional_obligations_clauses_tha() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::Keyword;

    let md = r#"# Spec Format

## Access Control

- **GIVEN** the user is authenticated:
  - **MUST** return their profile data
  - **MUST NOT** return other users' private data
  - **MAY** return extended metadata
"#;

    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec.sections[0].clauses;

    // GIVEN is a grouping construct, not itself a clause
    assert_eq!(clauses.len(), 3, "GIVEN should not appear as its own clause");
    assert!(
        clauses.iter().all(|c| c.keyword != Keyword::Given),
        "Keyword::Given should not be emitted as a top-level clause"
    );

    // The condition text is propagated to every nested clause
    let expected_condition = "the user is authenticated:";
    for clause in clauses {
        assert_eq!(
            clause.condition.as_deref(),
            Some(expected_condition),
            "All clauses inside GIVEN must carry the condition"
        );
    }

    // Nested keywords are parsed correctly
    assert_eq!(clauses[0].keyword, Keyword::Must);
    assert_eq!(clauses[1].keyword, Keyword::MustNot);
    assert_eq!(clauses[2].keyword, Keyword::May);
}