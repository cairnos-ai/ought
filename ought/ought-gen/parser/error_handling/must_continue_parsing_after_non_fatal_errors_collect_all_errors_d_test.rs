/// MUST continue parsing after non-fatal errors (collect all errors, don't stop at the first)
#[test]
fn test_parser__error_handling__must_continue_parsing_after_non_fatal_errors_collect_all_errors_d() {
    // Several keyword typos are interspersed with valid clauses. The parser must
    // not abort at the first unrecognized item — every valid clause that appears
    // later in the document must still be returned.
    let md = "\
# Svc

## Rules

- **MUTS** first typo — not a recognised keyword
- **MUST** first valid clause after typo
- **SHOLD** second typo
- **SHOULD** second valid clause after typo
- **MUST NOT** third valid clause at end of section
";
    let result = Parser::parse_string(md, Path::new("test.ought.md"));
    assert!(
        result.is_ok(),
        "unrecognised keyword typos must not cause a hard parse failure"
    );
    let spec = result.unwrap();
    let clauses = &spec.sections[0].clauses;

    // If the parser had stopped at the first bad item, only 0 or 1 clause would be
    // present. All three must be here to demonstrate full-document traversal.
    assert_eq!(
        clauses.len(),
        3,
        "parser must collect all valid clauses across the entire document, \
         not stop at the first unrecognised item"
    );
    assert_eq!(
        clauses[0].keyword,
        Keyword::Must,
        "valid MUST after first typo must be parsed"
    );
    assert_eq!(
        clauses[1].keyword,
        Keyword::Should,
        "valid SHOULD after second typo must be parsed"
    );
    assert_eq!(
        clauses[2].keyword,
        Keyword::MustNot,
        "valid MUST NOT at end of section must be parsed"
    );

    // Errors are returned as Vec<ParseError> — the whole collection, not just the first.
    // Verify the Vec type is used (not a single-error short-circuit) for file errors too.
    let file_result = Parser::parse_file(Path::new("/no/such/file.ought.md"));
    let errors = file_result.unwrap_err();
    // The Vec itself is the contract; callers can iterate all diagnostics.
    assert!(
        !errors.is_empty(),
        "errors must be returned in a Vec so callers see all diagnostics"
    );
}