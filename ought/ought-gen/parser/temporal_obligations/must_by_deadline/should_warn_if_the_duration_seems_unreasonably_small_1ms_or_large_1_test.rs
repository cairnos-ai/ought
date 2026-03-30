/// SHOULD warn if the duration seems unreasonably small (< 1ms) or large (> 1h)
///
/// Verifies that the parser emits at least one diagnostic warning when a
/// `**MUST BY**` duration is below 1 ms (e.g. `0ms`) or exceeds 1 hour
/// (e.g. `61m`), while still successfully producing a parsed clause so that
/// downstream tools are not blocked.
#[test]
fn test_parser__temporal_obligations__must_by_deadline__should_warn_if_the_duration_seems_unreasonably_small_1ms_or_large_1() {
    use std::path::Path;
    use ought_spec::parser::Parser;
    use ought_spec::types::*;

    fn parse_with_diagnostics(md: &str) -> (Spec, Vec<Diagnostic>) {
        Parser::parse_string_with_diagnostics(md, Path::new("test.ought.md"))
            .expect("parse should succeed even for out-of-range durations")
    }

    // --- too small: 0ms ---
    let md_zero = r#"# Svc

## SLAs

- **MUST BY 0ms** process the request
"#;
    let (spec_zero, diags_zero) = parse_with_diagnostics(md_zero);
    // Clause is still emitted
    assert_eq!(spec_zero.sections[0].clauses.len(), 1, "clause should still be produced for 0ms");
    // At least one warning-level diagnostic about the suspicious duration
    let has_warn_zero = diags_zero.iter().any(|d| {
        matches!(d.level, DiagnosticLevel::Warning) && d.message.to_lowercase().contains("duration")
    });
    assert!(
        has_warn_zero,
        "expected a warning diagnostic for duration < 1ms, got: {:?}",
        diags_zero
    );

    // --- too large: 61m (> 1h) ---
    let md_large = r#"# Svc

## SLAs

- **MUST BY 61m** complete the archive migration
"#;
    let (spec_large, diags_large) = parse_with_diagnostics(md_large);
    assert_eq!(spec_large.sections[0].clauses.len(), 1, "clause should still be produced for 61m");
    let has_warn_large = diags_large.iter().any(|d| {
        matches!(d.level, DiagnosticLevel::Warning) && d.message.to_lowercase().contains("duration")
    });
    assert!(
        has_warn_large,
        "expected a warning diagnostic for duration > 1h, got: {:?}",
        diags_large
    );

    // --- in-range value should produce no such warning ---
    let md_ok = r#"# Svc

## SLAs

- **MUST BY 500ms** respond to the ping
"#;
    let (_, diags_ok) = parse_with_diagnostics(md_ok);
    let has_warn_ok = diags_ok.iter().any(|d| {
        matches!(d.level, DiagnosticLevel::Warning) && d.message.to_lowercase().contains("duration")
    });
    assert!(
        !has_warn_ok,
        "expected no duration warning for a reasonable value (500ms), got: {:?}",
        diags_ok
    );
}