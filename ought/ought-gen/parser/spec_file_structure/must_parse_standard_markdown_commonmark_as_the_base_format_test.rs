/// MUST parse standard Markdown (CommonMark) as the base format
#[test]
fn test_parser__spec_file_structure__must_parse_standard_markdown_commonmark_as_the_base_format() {
    // Exercises headings, paragraphs, emphasis, inline code, fenced code blocks,
    // blockquotes, plain lists, and inline links — core CommonMark elements.
    let md = r#"# My Spec

## Intro

A paragraph with *italic*, ***bold italic***, and `inline code` text.

> A blockquote providing background context.

See also [the reference docs](http://example.com) for more detail.

- A plain list item
- Another informational bullet

```json
{"example": true}
```

## Rules

- **MUST** handle all CommonMark elements in surrounding prose
"#;
    let result = Parser::parse_string(md, Path::new("commonmark_test.ought.md"));
    assert!(
        result.is_ok(),
        "Parser must not fail on standard CommonMark: {:?}",
        result.err()
    );
    let spec = result.unwrap();
    assert_eq!(spec.name, "My Spec");
    let rules = spec.sections.iter().find(|s| s.title == "Rules").unwrap();
    assert_eq!(rules.clauses.len(), 1);
    assert_eq!(rules.clauses[0].keyword, Keyword::Must);
}