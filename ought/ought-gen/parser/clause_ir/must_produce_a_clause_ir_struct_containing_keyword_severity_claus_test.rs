/// MUST produce a clause IR struct containing: keyword, severity, clause text,
/// source location (file, line), parent section path, and a stable identifier
#[test]
fn test_parser__clause_ir__must_produce_a_clause_ir_struct_containing_keyword_severity_claus() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::*;

    let md = "# Auth\n\n## Login\n\n- **MUST** return a JWT token\n";
    let spec = Parser::parse_string(md, Path::new("auth.ought.md")).expect("parse failed");
    let clause = &spec.sections[0].clauses[0];

    // keyword
    assert_eq!(clause.keyword, Keyword::Must);

    // severity derived from keyword
    assert_eq!(clause.severity, Severity::Required);

    // clause text
    assert!(clause.text.contains("return a JWT token"));

    // source location: file
    assert_eq!(clause.source_location.file.to_str().unwrap(), "auth.ought.md");

    // source location: line (must be a real line in the file)
    assert!(clause.source_location.line > 0);

    // stable identifier encodes the parent section path
    assert_eq!(clause.id.0, "auth::login::must_return_a_jwt_token");

    // content_hash is populated
    assert!(!clause.content_hash.is_empty());
}