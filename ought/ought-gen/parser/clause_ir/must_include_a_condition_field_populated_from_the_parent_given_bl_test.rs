/// MUST include a `condition` field populated from the parent GIVEN block (null if unconditional)
#[test]
fn test_parser__clause_ir__must_include_a_condition_field_populated_from_the_parent_given_bl() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::*;

    // Unconditional clause has None condition
    let md = "# Svc\n\n## Rules\n\n- **MUST** always do this\n";
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    assert!(spec.sections[0].clauses[0].condition.is_none());

    // Clauses nested under GIVEN have the condition populated from GIVEN text
    let md_given = concat!(
        "# Svc\n\n## Rules\n\n",
        "- **GIVEN** the user is authenticated:\n",
        "  - **MUST** return profile data\n",
        "  - **MUST NOT** expose other users' data\n"
    );
    let spec_given =
        Parser::parse_string(md_given, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec_given.sections[0].clauses;

    // GIVEN itself is not emitted as a clause — nested items become clauses
    assert_eq!(clauses.len(), 2);
    assert_eq!(clauses[0].keyword, Keyword::Must);
    assert_eq!(clauses[1].keyword, Keyword::MustNot);

    // Every clause in the GIVEN block shares the condition
    assert_eq!(
        clauses[0].condition.as_deref(),
        Some("the user is authenticated:")
    );
    assert_eq!(clauses[0].condition, clauses[1].condition);
}