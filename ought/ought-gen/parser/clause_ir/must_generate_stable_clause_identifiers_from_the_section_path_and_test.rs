/// MUST generate stable clause identifiers from the section path and clause text
/// (e.g. `auth::login::must_return_jwt`)
#[test]
fn test_parser__clause_ir__must_generate_stable_clause_identifiers_from_the_section_path_and() {
    use std::path::Path;
    use ought_spec::parser::Parser;

    let md = "# Auth\n\n## Login\n\n- **MUST** return a JWT token\n";

    // Parsing the same content twice produces identical identifiers
    let spec1 = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let spec2 = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    assert_eq!(
        spec1.sections[0].clauses[0].id.0,
        spec2.sections[0].clauses[0].id.0
    );

    // Identifier follows spec::section::keyword_slug_of_text pattern
    assert_eq!(
        spec1.sections[0].clauses[0].id.0,
        "auth::login::must_return_a_jwt_token"
    );

    // Nested section path appears in full in the identifier
    let nested_md =
        "# Auth\n\n## Login\n\n### OAuth\n\n- **MUST** validate token signature\n";
    let nested_spec =
        Parser::parse_string(nested_md, Path::new("test.ought.md")).expect("parse failed");
    let nested_id = &nested_spec.sections[0].subsections[0].clauses[0].id.0;
    assert!(
        nested_id.starts_with("auth::login::oauth::"),
        "expected nested id to start with auth::login::oauth::, got: {nested_id}"
    );

    // Different clause text within the same section produces a different identifier
    let md2 = "# Auth\n\n## Login\n\n- **MUST** reject invalid tokens\n";
    let spec3 = Parser::parse_string(md2, Path::new("test.ought.md")).expect("parse failed");
    assert_ne!(
        spec1.sections[0].clauses[0].id.0,
        spec3.sections[0].clauses[0].id.0
    );
}