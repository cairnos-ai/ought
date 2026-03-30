/// SHOULD be parseable by a standalone library with no LLM dependency, so
/// other tools can consume the format.
#[test]
fn test_ought__spec_format__should_be_parseable_by_a_standalone_library_with_no_llm_dependency() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::{Keyword, Severity, Temporal};

    // This test deliberately exercises the full surface of the parser in a
    // single self-contained call to Parser::parse_string — no LLM, no network,
    // no external process. If the library compiles and this test passes, the
    // format is provably parseable without an LLM dependency.
    let md = r#"# Standalone Spec

context: Verifies the parser works with no external dependencies

requires: [Other](other.ought.md)

## Obligations

- **MUST** validate all inputs
- **MUST NOT** leak secrets
- **SHOULD** emit structured logs
- **SHOULD NOT** swallow errors silently
- **MAY** support optional features
- **WONT** implement deprecated protocols

## Conditional

- **GIVEN** the cache is warm:
  - **MUST** serve from cache

## Degradation

- **MUST** return a result
  - **OTHERWISE** return a fallback value

## Temporal

- **MUST ALWAYS** preserve referential integrity
- **MUST BY 500ms** respond to health checks
"#;

    let result = Parser::parse_string(md, Path::new("standalone.ought.md"));
    assert!(result.is_ok(), "Standalone parser must succeed: {:?}", result.err());

    let spec = result.unwrap();

    // Basic structure
    assert_eq!(spec.name, "Standalone Spec");
    assert_eq!(spec.metadata.context.as_deref(), Some("Verifies the parser works with no external dependencies"));
    assert_eq!(spec.metadata.requires.len(), 1);
    assert_eq!(spec.sections.len(), 4);

    // RFC 2119 keywords present
    let obligations = &spec.sections[0].clauses;
    assert_eq!(obligations.len(), 6);
    assert!(obligations.iter().any(|c| c.keyword == Keyword::Must));
    assert!(obligations.iter().any(|c| c.keyword == Keyword::MustNot));
    assert!(obligations.iter().any(|c| c.keyword == Keyword::Should));
    assert!(obligations.iter().any(|c| c.keyword == Keyword::ShouldNot));
    assert!(obligations.iter().any(|c| c.keyword == Keyword::May));
    assert!(obligations.iter().any(|c| c.keyword == Keyword::Wont));

    // GIVEN propagates condition
    let conditional = &spec.sections[1].clauses;
    assert_eq!(conditional.len(), 1);
    assert!(conditional[0].condition.is_some());

    // OTHERWISE chain
    let degradation = &spec.sections[2].clauses;
    assert_eq!(degradation.len(), 1);
    assert_eq!(degradation[0].otherwise.len(), 1);

    // Temporal constraints
    let temporal = &spec.sections[3].clauses;
    assert_eq!(temporal.len(), 2);
    assert!(matches!(temporal[0].temporal, Some(Temporal::Invariant)));
    assert!(matches!(temporal[1].temporal, Some(Temporal::Deadline(_))));

    // All clause IDs are non-empty and unique
    let all_clauses: Vec<_> = spec.sections.iter()
        .flat_map(|s| s.clauses.iter())
        .collect();
    let ids: Vec<_> = all_clauses.iter().map(|c| &c.id.0).collect();
    let unique_ids: std::collections::HashSet<_> = ids.iter().collect();
    assert_eq!(ids.len(), unique_ids.len(), "All clause IDs must be unique");
    assert!(all_clauses.iter().all(|c| !c.id.0.is_empty()), "No clause may have an empty ID");

    // Severity is always set
    for clause in &all_clauses {
        let expected = clause.keyword.severity();
        assert_eq!(clause.severity, expected,
            "Severity mismatch for {:?}: expected {:?}", clause.keyword, expected);
    }
}