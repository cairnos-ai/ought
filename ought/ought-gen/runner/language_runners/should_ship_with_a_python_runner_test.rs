/// SHOULD ship with a Python runner
///
/// The runner should include a Python language runner that delegates to `pytest`.
/// The runner implementation must exist and be reachable by name; availability
/// in the current environment (i.e. whether pytest is installed) is separate.
#[test]
fn test_runner__language_runners__should_ship_with_a_python_runner() {
    // The factory must recognise "python" as a valid language key.
    let runner = ought_run::runners::from_name("python")
        .expect("from_name(\"python\") must succeed — Python runner should be shipped");

    assert_eq!(
        runner.name(),
        "python",
        "Python runner must report name \"python\""
    );

    // is_available() reflects whether pytest is installed in this environment —
    // that is not required for the runner to ship.  We just verify the method
    // exists and returns a bool without panicking.
    let _ = runner.is_available();
}