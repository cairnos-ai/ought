/// MUST NOT fail on standard markdown that doesn't contain ought keywords (just produce zero clauses)
#[test]
fn test_parser__spec_file_structure__must_not_fail_on_standard_markdown_that_doesn_t_contain_ought_keyword() {
    let md = r#"# Plain Spec

## Overview

This is a standard markdown document with no ought keywords whatsoever.

It has paragraphs, *italic text*, and `code spans`.

- A plain list item
- Another plain list item

## Details

More prose here. No deontic keywords appear in bold in any list items.

```python
def example():
    return True
```
"#;
    let result = Parser::parse_string(md, Path::new("test.ought.md"));
    assert!(
        result.is_ok(),
        "Parser must not fail on keyword-free markdown: {:?}",
        result.err()
    );
    let spec = result.unwrap();
    assert_eq!(spec.name, "Plain Spec");
    assert!(
        !spec.sections.is_empty(),
        "sections must still be parsed from headings"
    );
    let total_clauses: usize = spec.sections.iter().map(|s| s.clauses.len()).sum();
    assert_eq!(
        total_clauses, 0,
        "keyword-free markdown must produce zero clauses"
    );
}