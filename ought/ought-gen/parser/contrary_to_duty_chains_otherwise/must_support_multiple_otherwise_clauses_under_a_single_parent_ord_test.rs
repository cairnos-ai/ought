/// MUST support multiple OTHERWISE clauses under a single parent (ordered fallback chain)
#[test]
fn test_parser__contrary_to_duty_chains_otherwise__must_support_multiple_otherwise_clauses_under_a_single_parent_ord() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::*;

    let md = r#"# Svc

## Payments

- **MUST** charge the primary card
  - **OTHERWISE** charge the backup card
  - **OTHERWISE** add to pending queue
  - **OTHERWISE** reject with insufficient funds error
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec.sections[0].clauses;

    assert_eq!(clauses.len(), 1, "only one top-level clause should exist");

    let otherwise = &clauses[0].otherwise;
    assert_eq!(otherwise.len(), 3, "all three fallbacks must be collected under the single parent");
    assert!(otherwise.iter().all(|c| c.keyword == Keyword::Otherwise));

    assert!(otherwise[0].text.contains("backup card"));
    assert!(otherwise[1].text.contains("pending queue"));
    assert!(otherwise[2].text.contains("insufficient funds"));
}