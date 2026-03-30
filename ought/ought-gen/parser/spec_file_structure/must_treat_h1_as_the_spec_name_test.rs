/// MUST treat H1 (`#`) as the spec name
#[test]
fn test_parser__spec_file_structure__must_treat_h1_as_the_spec_name() {
    let md = "# Payment Gateway\n\n## Rules\n\n- **MUST** work\n";
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");

    assert_eq!(
        spec.name, "Payment Gateway",
        "H1 heading text must become spec.name"
    );
    // The H1 text must not also appear as a section title
    assert!(
        spec.sections.iter().all(|s| s.title != "Payment Gateway"),
        "H1 must not be duplicated as a top-level section"
    );
}