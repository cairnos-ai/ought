/// MUST parse inline markdown links to other .ought.md files as cross-references
#[test]
fn test_parser__cross_file_references__must_parse_inline_markdown_links_to_other_ought_md_files_as_cross() {
    let md = r#"# Checkout

requires: [Pricing](pricing.ought.md), [Users](users.ought.md)

## Payment

- **MUST** apply pricing rules
"#;
    let spec = Parser::parse_string(md, Path::new("checkout.ought.md"))
        .expect("parse failed");

    assert_eq!(
        spec.metadata.requires.len(),
        2,
        "each markdown link in requires: must become a separate cross-reference"
    );

    let pricing = &spec.metadata.requires[0];
    assert_eq!(
        pricing.label, "Pricing",
        "markdown link label must be captured as the SpecRef label"
    );
    assert_eq!(
        pricing.path.to_str().unwrap(),
        "pricing.ought.md",
        "markdown link URL must become the SpecRef path"
    );
    assert!(
        pricing.anchor.is_none(),
        "link without a URL fragment must have no anchor"
    );

    let users = &spec.metadata.requires[1];
    assert_eq!(
        users.label, "Users",
        "second link label must be captured"
    );
    assert_eq!(
        users.path.to_str().unwrap(),
        "users.ought.md",
        "second link URL must become the SpecRef path"
    );
    assert!(users.anchor.is_none());
}