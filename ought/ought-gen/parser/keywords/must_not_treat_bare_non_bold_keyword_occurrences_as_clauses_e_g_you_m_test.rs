/// MUST NOT treat bare (non-bold) keyword occurrences as clauses (e.g. "you must restart" in prose)
#[test]
fn test_parser__keywords__must_not_treat_bare_non_bold_keyword_occurrences_as_clauses_e_g_you_m() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::*;

    let md = r#"# Svc

## Overview

This service must handle all requests. Operators should monitor memory usage.
You must not store credentials in logs. The system may cache responses.
Deployments wont need downtime. Given the above, otherwise consider alternatives.

- A plain list item that says you must restart after upgrade
- Another item: should not be mistaken for a clause
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    assert!(
        spec.sections[0].clauses.is_empty(),
        "bare keywords in paragraphs and unbolded list items must not produce any clauses"
    );

    // Confirm that a bold keyword in the same section does produce a clause,
    // proving the parser is active and the above result is not a parser failure.
    let md_mixed = r#"# Svc

## Mixed

This service must handle all requests as described above.

- **MUST** actually validate the token
"#;
    let spec2 = Parser::parse_string(md_mixed, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec2.sections[0].clauses;
    assert_eq!(clauses.len(), 1, "only the bold keyword item must become a clause");
    assert_eq!(clauses[0].keyword, Keyword::Must);
    assert!(clauses[0].text.contains("validate the token"));
}