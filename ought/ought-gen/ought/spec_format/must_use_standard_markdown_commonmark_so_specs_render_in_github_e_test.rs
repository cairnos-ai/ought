/// MUST use standard markdown (CommonMark) so specs render in GitHub, editors,
/// and browsers with zero tooling.
#[test]
fn test_ought__spec_format__must_use_standard_markdown_commonmark_so_specs_render_in_github_e() {
    use std::path::Path;
    use ought_spec::parser::Parser;

    // A spec that uses a representative range of CommonMark constructs:
    // ATX headings, fenced code blocks, bold/italic inline, unordered lists,
    // blockquotes, and inline code.  The parser MUST accept all of these
    // without error and surface the H1 as the spec name plus at least the
    // clause embedded in the list.
    let md = r#"# Spec Format

> This spec is written in standard CommonMark.

## Rules

Some *introductory* prose with `inline code` and **bold** text.

- **MUST** render correctly in any CommonMark-compliant renderer

```markdown
# Example heading
- **MUST** sample clause
```
"#;

    let result = Parser::parse_string(md, Path::new("spec_format.ought.md"));
    assert!(result.is_ok(), "CommonMark spec should parse without errors: {:?}", result.err());
    let spec = result.unwrap();

    // H1 becomes the spec name
    assert_eq!(spec.name, "Spec Format");

    // Section captured with prose
    assert_eq!(spec.sections.len(), 1);
    assert_eq!(spec.sections[0].title, "Rules");
    assert!(!spec.sections[0].prose.is_empty(), "CommonMark prose should be captured");

    // The single deontic clause in the list is parsed
    assert_eq!(spec.sections[0].clauses.len(), 1);
    assert!(spec.sections[0].clauses[0].text.contains("render correctly"));

    // The fenced code block following the clause is attached as a hint
    assert_eq!(spec.sections[0].clauses[0].hints.len(), 1);
}