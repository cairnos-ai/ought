/// MAY ship with a Go runner
///
/// The runner may optionally include a Go language runner that delegates to
/// `go test -v ./...`.  If it ships, it must be accessible via `from_name("go")`
/// and report the correct name; this test verifies that the shipped binary
/// includes the Go runner.
#[test]
fn test_runner__language_runners__may_ship_with_a_go_runner() {
    // The Go runner is optional (MAY), but the codebase does ship one.
    // Verify that `from_name` accepts "go" without error.
    let result = ought_run::runners::from_name("go");

    assert!(
        result.is_ok(),
        "from_name(\"go\") must succeed because the Go runner is included in this build; \
         error: {:?}",
        result.err()
    );

    let runner = result.unwrap();
    assert_eq!(
        runner.name(),
        "go",
        "Go runner must report name \"go\""
    );

    // Availability reflects whether `go` is installed — not a shipping requirement.
    let _ = runner.is_available();
}