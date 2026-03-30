/// MUST parse frontmatter-style metadata at the top of the file: `context:`, `source:`, `schema:`, `requires:`
#[test]
fn test_parser__spec_file_structure__must_parse_frontmatter_style_metadata_at_the_top_of_the_file_cont() {
    let md = r#"# Payments

context: Handles payment processing and invoicing
source: src/payments/, src/billing/
schema: schema/payments.graphql, schema/billing.graphql
requires: [Auth](auth.ought.md), [Users](users.ought.md#accounts)

## Checkout

- **MUST** process payments
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");

    assert_eq!(
        spec.metadata.context.as_deref(),
        Some("Handles payment processing and invoicing"),
        "context: field must be parsed into metadata.context"
    );
    assert_eq!(
        spec.metadata.sources,
        vec!["src/payments/", "src/billing/"],
        "source: field must be parsed as a comma-separated list"
    );
    assert_eq!(
        spec.metadata.schemas,
        vec!["schema/payments.graphql", "schema/billing.graphql"],
        "schema: field must be parsed as a comma-separated list"
    );
    assert_eq!(
        spec.metadata.requires.len(),
        2,
        "requires: field must list all spec dependencies"
    );
    assert_eq!(spec.metadata.requires[0].label, "Auth");
    assert_eq!(
        spec.metadata.requires[0].path.to_str().unwrap(),
        "auth.ought.md"
    );
    assert_eq!(
        spec.metadata.requires[0].anchor,
        None,
        "link without fragment must have no anchor"
    );
    assert_eq!(spec.metadata.requires[1].label, "Users");
    assert_eq!(
        spec.metadata.requires[1].path.to_str().unwrap(),
        "users.ought.md"
    );
    assert_eq!(
        spec.metadata.requires[1].anchor.as_deref(),
        Some("accounts"),
        "link fragment must be captured as anchor"
    );
}