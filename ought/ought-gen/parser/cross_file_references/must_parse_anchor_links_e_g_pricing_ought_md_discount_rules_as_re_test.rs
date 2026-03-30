/// MUST parse anchor links (e.g. `pricing.ought.md#discount-rules`) as references to specific sections
#[test]
fn test_parser__cross_file_references__must_parse_anchor_links_e_g_pricing_ought_md_discount_rules_as_re() {
    let md = r#"# Checkout

requires: [Pricing](pricing.ought.md#discount-rules), [Auth](auth.ought.md#session-tokens)

## Payment

- **MUST** apply discount rules from the pricing spec
"#;
    let spec = Parser::parse_string(md, Path::new("checkout.ought.md"))
        .expect("parse failed");

    assert_eq!(
        spec.metadata.requires.len(),
        2,
        "anchor links must be parsed as cross-references"
    );

    let pricing = &spec.metadata.requires[0];
    assert_eq!(
        pricing.label, "Pricing",
        "link label must be captured from an anchor link"
    );
    assert_eq!(
        pricing.path.to_str().unwrap(),
        "pricing.ought.md",
        "file path must be extracted without the fragment"
    );
    assert_eq!(
        pricing.anchor.as_deref(),
        Some("discount-rules"),
        "URL fragment must be stored as the anchor field"
    );

    let auth = &spec.metadata.requires[1];
    assert_eq!(
        auth.path.to_str().unwrap(),
        "auth.ought.md",
        "second anchor link file path must be extracted correctly"
    );
    assert_eq!(
        auth.anchor.as_deref(),
        Some("session-tokens"),
        "second anchor must be extracted from the URL fragment"
    );
}