/// MUST report parse errors with the file path, line number, and a clear message
#[test]
fn test_parser__error_handling__must_report_parse_errors_with_the_file_path_line_number_and_a_cle() {
    // Directly construct a ParseError and verify all three required fields are present
    // and surfaced by the Display implementation.
    let path = PathBuf::from("spec/auth.ought.md");
    let err = ParseError {
        file: path.clone(),
        line: 23,
        message: "unexpected token after MUST".to_string(),
    };

    assert_eq!(err.file, path, "error must carry the source file path");
    assert_eq!(err.line, 23, "error must carry the 1-based line number");
    assert!(!err.message.is_empty(), "error must include a non-empty human-readable message");

    // Display must render all three components so callers can show "file:line: msg"
    let display = format!("{}", err);
    assert!(
        display.contains("spec/auth.ought.md"),
        "display must include the file path; got: {display}"
    );
    assert!(
        display.contains("23"),
        "display must include the line number; got: {display}"
    );
    assert!(
        display.contains("unexpected token"),
        "display must include the message text; got: {display}"
    );

    // parse_file errors must embed the path that was attempted so the caller can
    // report "failed to read foo.ought.md" without extra bookkeeping.
    let missing = Path::new("/nonexistent/spec.ought.md");
    let result = Parser::parse_file(missing);
    assert!(result.is_err(), "parse_file must return Err for a missing file");
    let errors = result.unwrap_err();
    assert!(!errors.is_empty(), "errors Vec must be non-empty");
    assert_eq!(
        errors[0].file, missing,
        "parse_file error must record the path that was attempted"
    );
    assert!(
        !errors[0].message.is_empty(),
        "parse_file error must include a clear message describing the failure"
    );
}