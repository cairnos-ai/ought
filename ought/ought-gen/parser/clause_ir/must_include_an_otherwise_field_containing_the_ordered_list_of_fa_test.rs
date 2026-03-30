/// MUST include an `otherwise` field containing the ordered list of fallback clauses (empty if none)
#[test]
fn test_parser__clause_ir__must_include_an_otherwise_field_containing_the_ordered_list_of_fa() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::*;

    // Clause without OTHERWISE has an empty otherwise vec
    let md = "# Svc\n\n## Perf\n\n- **MUST** respond within 200ms\n";
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    assert!(spec.sections[0].clauses[0].otherwise.is_empty());

    // Clause with OTHERWISE chain carries an ordered list of fallback clauses
    let md_chain = concat!(
        "# Svc\n\n## Perf\n\n",
        "- **MUST** respond within 200ms\n",
        "  - **OTHERWISE** return a cached response\n",
        "  - **OTHERWISE** return 504 Gateway Timeout\n"
    );
    let spec_chain =
        Parser::parse_string(md_chain, Path::new("test.ought.md")).expect("parse failed");
    let clause = &spec_chain.sections[0].clauses[0];

    assert_eq!(clause.keyword, Keyword::Must);

    // Fallbacks are ordered by appearance
    assert_eq!(clause.otherwise.len(), 2);
    assert_eq!(clause.otherwise[0].keyword, Keyword::Otherwise);
    assert!(
        clause.otherwise[0].text.contains("cached response"),
        "first fallback should be cached response"
    );
    assert_eq!(clause.otherwise[1].keyword, Keyword::Otherwise);
    assert!(
        clause.otherwise[1].text.contains("504"),
        "second fallback should be 504"
    );

    // OTHERWISE clauses are not surfaced as top-level section clauses
    assert_eq!(spec_chain.sections[0].clauses.len(), 1);

    // OTHERWISE inherits the severity of its parent
    assert_eq!(clause.otherwise[0].severity, Severity::Required);
}