/// MUST treat H2+ (`##`, `###`, etc.) as nested test groups/sections
#[test]
fn test_parser__spec_file_structure__must_treat_h2_etc_as_nested_test_groups_sections() {
    let md = r#"# Svc

## Auth

### Login

#### Credentials

- **MUST** validate credentials

### Logout

- **SHOULD** clear session

## Billing

- **MUST** charge the correct amount
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");

    // H2 headings produce top-level sections
    assert_eq!(spec.sections.len(), 2);
    assert_eq!(spec.sections[0].title, "Auth");
    assert_eq!(spec.sections[0].depth, 2);
    assert_eq!(spec.sections[1].title, "Billing");
    assert_eq!(spec.sections[1].depth, 2);

    // H3 headings become subsections of their enclosing H2
    let auth = &spec.sections[0];
    assert_eq!(auth.subsections.len(), 2);
    assert_eq!(auth.subsections[0].title, "Login");
    assert_eq!(auth.subsections[0].depth, 3);
    assert_eq!(auth.subsections[1].title, "Logout");
    assert_eq!(auth.subsections[1].depth, 3);

    // H4 headings become subsections of their enclosing H3
    let login = &auth.subsections[0];
    assert_eq!(login.subsections.len(), 1);
    assert_eq!(login.subsections[0].title, "Credentials");
    assert_eq!(login.subsections[0].depth, 4);
    assert_eq!(login.subsections[0].clauses.len(), 1);
}