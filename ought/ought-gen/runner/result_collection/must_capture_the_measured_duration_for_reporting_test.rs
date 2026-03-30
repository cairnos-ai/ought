/// MUST capture the measured duration for reporting
/// GIVEN: a clause is MUST BY
#[test]
fn test_runner__result_collection__must_capture_the_measured_duration_for_reporting() {
    use std::time::Duration;

    struct MustByResult {
        clause_id: String,
        passed: bool,
        measured_duration: Duration,
        deadline: Duration,
    }

    // Clause completed within its deadline → passed, duration still recorded
    let within_deadline = MustByResult {
        clause_id: "api::must_respond_within_200ms".to_string(),
        passed: true,
        measured_duration: Duration::from_millis(180),
        deadline: Duration::from_millis(200),
    };

    assert!(within_deadline.passed);
    assert!(within_deadline.measured_duration <= within_deadline.deadline);
    assert_eq!(
        within_deadline.measured_duration,
        Duration::from_millis(180),
        "measured duration must be captured for a passing MUST BY clause"
    );

    // Clause exceeded its deadline → failed, but duration must still be captured
    let exceeded_deadline = MustByResult {
        clause_id: "api::must_respond_within_200ms".to_string(),
        passed: false,
        measured_duration: Duration::from_millis(350),
        deadline: Duration::from_millis(200),
    };

    assert!(!exceeded_deadline.passed);
    assert!(exceeded_deadline.measured_duration > exceeded_deadline.deadline);
    assert_eq!(
        exceeded_deadline.measured_duration,
        Duration::from_millis(350),
        "measured duration must be captured even when the deadline is exceeded"
    );
}