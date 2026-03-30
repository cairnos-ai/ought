/// SHOULD ship with a JavaScript/TypeScript runner
///
/// The runner should include a TypeScript/JavaScript runner that delegates to
/// `npx jest --verbose`.  Both "typescript" and the "ts" alias must be recognised.
#[test]
fn test_runner__language_runners__should_ship_with_a_javascript_typescript_runner() {
    // Primary name must be recognised.
    let runner = ought_run::runners::from_name("typescript")
        .expect("from_name(\"typescript\") must succeed — TS/JS runner should be shipped");

    assert_eq!(
        runner.name(),
        "typescript",
        "TypeScript runner must report name \"typescript\""
    );

    // "ts" is the documented short alias — it must resolve to the same runner.
    let runner_alias = ought_run::runners::from_name("ts")
        .expect("from_name(\"ts\") must succeed — \"ts\" is the short alias for the TS runner");

    assert_eq!(
        runner_alias.name(),
        "typescript",
        "\"ts\" alias must resolve to a runner named \"typescript\""
    );

    // Availability depends on whether npx is installed; just confirm no panic.
    let _ = runner.is_available();
}