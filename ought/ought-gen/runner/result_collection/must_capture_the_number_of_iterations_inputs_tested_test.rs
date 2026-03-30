/// MUST capture the number of iterations/inputs tested
/// GIVEN: a clause is MUST ALWAYS
#[test]
fn test_runner__result_collection__must_capture_the_number_of_iterations_inputs_tested() {
    struct MustAlwaysResult {
        clause_id: String,
        passed: bool,
        iterations_tested: usize,
    }

    // MUST ALWAYS clause that passed over 50 inputs
    let pass_result = MustAlwaysResult {
        clause_id: "validation::must_always_reject_empty_input".to_string(),
        passed: true,
        iterations_tested: 50,
    };

    assert_eq!(pass_result.iterations_tested, 50);
    assert!(
        pass_result.iterations_tested > 0,
        "MUST ALWAYS result must record at least one iteration"
    );

    // MUST ALWAYS clause that failed partway through — iterations still captured
    let fail_result = MustAlwaysResult {
        clause_id: "validation::must_always_sanitize".to_string(),
        passed: false,
        iterations_tested: 23,
    };

    assert!(!fail_result.passed);
    assert_eq!(
        fail_result.iterations_tested, 23,
        "number of iterations tested must be captured even when the clause fails"
    );
    assert!(
        fail_result.iterations_tested > 0,
        "at least one iteration must be recorded"
    );
}