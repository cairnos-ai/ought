/// SHOULD warn on likely typos (e.g. `**MUTS**` close to a known keyword)
#[test]
fn test_parser__error_handling__should_warn_on_likely_typos_e_g_muts_close_to_a_known_keyword() {
    // **MUTS** is an edit-distance-1 typo for **MUST**; **SHOLD** for **SHOULD**.
    // The parser must not silently accept them as valid keywords, and SHOULD
    // surface a diagnostic pointing to the likely correction.
    let md = "\
# Svc

## Rules

- **MUTS** typo for MUST — must not become a clause
- **MUST** the real keyword
- **SHOLD** typo for SHOULD — must not become a clause
- **SHOUD** another SHOULD variant — must not become a clause
";
    let spec = Parser::parse_string(md, Path::new("typos.ought.md"))
        .expect("keyword typos must not crash the parser");

    let clauses = &spec.sections[0].clauses;

    // Typo keywords must NOT be silently accepted and turned into clauses.
    assert!(
        !clauses.iter().any(|c| c.text.contains("typo for MUST")),
        "**MUTS** must not produce a clause — it is not a recognised keyword"
    );
    assert!(
        !clauses.iter().any(|c| c.text.contains("typo for SHOULD")),
        "**SHOLD** must not produce a clause — it is not a recognised keyword"
    );
    assert!(
        !clauses.iter().any(|c| c.text.contains("another SHOULD variant")),
        "**SHOUD** must not produce a clause — it is not a recognised keyword"
    );

    // The one genuinely valid keyword must still be recognised.
    assert_eq!(
        clauses.len(),
        1,
        "only the single valid **MUST** item must become a clause; typos become prose"
    );
    assert_eq!(
        clauses[0].keyword,
        Keyword::Must,
        "valid **MUST** must be recognised even when surrounded by typo items"
    );

    // TODO: when a warning/lint system is added, assert diagnostics such as:
    //   ParseWarning { file: "typos.ought.md", line: 5,
    //       message: "unknown keyword **MUTS** — did you mean **MUST**?" }
    //   ParseWarning { file: "typos.ought.md", line: 7,
    //       message: "unknown keyword **SHOLD** — did you mean **SHOULD**?" }
    // These could be surfaced via a `warnings: Vec<ParseWarning>` field on a
    // `ParseResult` wrapper or alongside `ParseError` in a `ParseDiagnostic` enum.
}