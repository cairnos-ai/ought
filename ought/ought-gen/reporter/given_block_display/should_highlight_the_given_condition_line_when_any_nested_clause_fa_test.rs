/// SHOULD highlight the GIVEN condition line when any nested clause fails (the condition is relevant context)
    #[test]
    fn test_reporter__given_block_display__should_highlight_the_given_condition_line_when_any_nested_clause_fa() {
        // Scenario from spec hint: one child fails → condition line is highlighted
        let has_failure = GivenBlock {
            condition: "the user is authenticated",
            children: vec![
                Child { keyword: "MUST", text: "return their profile data",            status: Status::Passed },
                Child { keyword: "MUST", text: "NOT return other users' private data", status: Status::Failed },
            ],
        };

        let out_with_failure = render_given(&has_failure, 1, true);

        // GIVEN header must be bolded to draw attention to the relevant condition
        let given_line = out_with_failure.lines()
            .find(|l| l.contains("GIVEN the user is authenticated:"))
            .expect("GIVEN header line must be present");
        assert!(
            given_line.contains(BOLD),
            "GIVEN header must be highlighted (bold) when any nested clause fails, got: {:?}",
            given_line
        );
        assert!(
            !given_line.contains(DIM),
            "GIVEN header must NOT be dimmed when a nested clause fails, got: {:?}",
            given_line
        );

        // The two blocks together: highlight only the failing one, dim the passing one
        let all_pass = GivenBlock {
            condition: "the token is expired",
            children: vec![
                Child { keyword: "MUST",   text: "return 401",                      status: Status::Passed },
                Child { keyword: "SHOULD", text: "include WWW-Authenticate header", status: Status::Passed },
            ],
        };

        let combined = render_given_blocks(&[has_failure, all_pass], 1, true);

        let failing_given_line = combined.lines()
            .find(|l| l.contains("GIVEN the user is authenticated:"))
            .expect("first GIVEN header must be in combined output");
        let passing_given_line = combined.lines()
            .find(|l| l.contains("GIVEN the token is expired:"))
            .expect("second GIVEN header must be in combined output");

        assert!(
            failing_given_line.contains(BOLD),
            "GIVEN with failing child must be bold, got: {:?}", failing_given_line
        );
        assert!(
            passing_given_line.contains(DIM),
            "GIVEN with all-passing children must be dim, got: {:?}", passing_given_line
        );

        // Symmetry: the two headers must have visually distinct styling
        assert_ne!(
            failing_given_line.contains(BOLD),
            passing_given_line.contains(BOLD),
            "the two GIVEN headers must not share the same bold styling"
        );
    }
}