/// MUST support cross-file references via standard markdown links, so specs
/// can link to each other and form a hierarchy.
#[test]
fn test_ought__spec_format__must_support_cross_file_references_via_standard_markdown_links_so() {
    use std::path::Path;
    use ought_spec::parser::Parser;

    // The `requires:` metadata field accepts a comma-separated list of
    // standard markdown links — the same syntax used everywhere in CommonMark.
    let md = r#"# Spec Format

requires: [Auth](auth.ought.md), [Billing](billing.ought.md#invoices), [Core](core/base.ought.md#rules)

## Sections

- **MUST** integrate with referenced specs
"#;

    let spec = Parser::parse_string(md, Path::new("spec_format.ought.md")).expect("parse failed");
    let refs = &spec.metadata.requires;

    assert_eq!(refs.len(), 3, "Three cross-file references expected");

    // Plain reference without anchor
    assert_eq!(refs[0].label, "Auth");
    assert_eq!(refs[0].path.to_str().unwrap(), "auth.ought.md");
    assert_eq!(refs[0].anchor, None);

    // Reference with fragment anchor
    assert_eq!(refs[1].label, "Billing");
    assert_eq!(refs[1].path.to_str().unwrap(), "billing.ought.md");
    assert_eq!(refs[1].anchor.as_deref(), Some("invoices"));

    // Reference with nested path and anchor
    assert_eq!(refs[2].label, "Core");
    assert_eq!(refs[2].path.to_str().unwrap(), "core/base.ought.md");
    assert_eq!(refs[2].anchor.as_deref(), Some("rules"));
}