/// MUST parse `requires:` metadata as a list of relative paths to other .ought.md files
#[test]
fn test_parser__cross_file_references__must_parse_requires_metadata_as_a_list_of_relative_paths_to_other() {
    let md = r#"# Billing

requires: pricing.ought.md
requires: users.ought.md

## Invoices

- **MUST** calculate totals correctly
"#;
    let spec = Parser::parse_string(md, Path::new("billing.ought.md"))
        .expect("parse failed");

    assert_eq!(
        spec.metadata.requires.len(),
        2,
        "requires: metadata must list all referenced spec files"
    );

    let first = &spec.metadata.requires[0];
    assert_eq!(
        first.path.to_str().unwrap(),
        "pricing.ought.md",
        "first requires: entry must carry the correct relative path"
    );
    assert!(
        first.anchor.is_none(),
        "plain path without a fragment must have no anchor"
    );

    let second = &spec.metadata.requires[1];
    assert_eq!(
        second.path.to_str().unwrap(),
        "users.ought.md",
        "second requires: entry must carry the correct relative path"
    );
    assert!(
        second.anchor.is_none(),
        "plain path without a fragment must have no anchor"
    );
}