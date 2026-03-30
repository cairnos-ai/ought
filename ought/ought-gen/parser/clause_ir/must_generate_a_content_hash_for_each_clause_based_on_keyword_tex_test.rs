/// MUST generate a content hash for each clause based on keyword + text + relevant context
#[test]
fn test_parser__clause_ir__must_generate_a_content_hash_for_each_clause_based_on_keyword_tex() {
    use std::path::Path;
    use ought_spec::parser::Parser;

    let md = "# Svc\n\n## Rules\n\n- **MUST** do something specific\n";

    // Same content parsed twice produces the same hash
    let spec1 = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let spec2 = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    assert_eq!(
        spec1.sections[0].clauses[0].content_hash,
        spec2.sections[0].clauses[0].content_hash
    );

    // Different keyword changes the hash
    let md_should = "# Svc\n\n## Rules\n\n- **SHOULD** do something specific\n";
    let spec_should =
        Parser::parse_string(md_should, Path::new("test.ought.md")).expect("parse failed");
    assert_ne!(
        spec1.sections[0].clauses[0].content_hash,
        spec_should.sections[0].clauses[0].content_hash
    );

    // Different text changes the hash
    let md_diff = "# Svc\n\n## Rules\n\n- **MUST** do something entirely different\n";
    let spec_diff =
        Parser::parse_string(md_diff, Path::new("test.ought.md")).expect("parse failed");
    assert_ne!(
        spec1.sections[0].clauses[0].content_hash,
        spec_diff.sections[0].clauses[0].content_hash
    );

    // Different condition (GIVEN context) changes the hash
    let md_cond =
        "# Svc\n\n## Rules\n\n- **GIVEN** logged in:\n  - **MUST** do something specific\n";
    let spec_cond =
        Parser::parse_string(md_cond, Path::new("test.ought.md")).expect("parse failed");
    assert_ne!(
        spec1.sections[0].clauses[0].content_hash,
        spec_cond.sections[0].clauses[0].content_hash
    );

    // Hash is a non-empty lowercase hex string
    let hash = &spec1.sections[0].clauses[0].content_hash;
    assert!(!hash.is_empty());
    assert!(
        hash.chars().all(|c| c.is_ascii_hexdigit()),
        "expected hex hash, got: {hash}"
    );
}