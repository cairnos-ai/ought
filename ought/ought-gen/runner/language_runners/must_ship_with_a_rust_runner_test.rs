/// MUST ship with a Rust runner
///
/// The runner must include a Rust language runner that delegates to `cargo test`.
/// Being a MUST clause, the runner must exist, be accessible by name, and since
/// these tests are themselves compiled with cargo, the harness must also be available.
#[test]
fn test_runner__language_runners__must_ship_with_a_rust_runner() {
    // The factory must recognise "rust" as a valid language key.
    let runner = ought_run::runners::from_name("rust")
        .expect("from_name(\"rust\") must succeed — Rust runner is required");

    assert_eq!(
        runner.name(),
        "rust",
        "Rust runner must report name \"rust\""
    );

    // Because these tests are compiled and run with cargo, the harness is
    // definitionally available in this environment.
    assert!(
        runner.is_available(),
        "Rust runner must report is_available() == true when cargo is on PATH \
         (this test suite is itself running under cargo)"
    );
}