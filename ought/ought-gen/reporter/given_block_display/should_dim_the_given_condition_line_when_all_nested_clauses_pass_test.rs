/// SHOULD dim the GIVEN condition line when all nested clauses pass
    #[test]
    fn test_reporter__given_block_display__should_dim_the_given_condition_line_when_all_nested_clauses_pass() {
        let all_pass = GivenBlock {
            condition: "the token is expired",
            children: vec![
                Child { keyword: "MUST",   text: "return 401",                      status: Status::Passed },
                Child { keyword: "SHOULD", text: "include WWW-Authenticate header", status: Status::Passed },
            ],
        };

        let out_color   = render_given(&all_pass, 1, true);
        let out_nocolor = render_given(&all_pass, 1, false);

        // With color: the GIVEN header line must be wrapped in the DIM code
        let given_line = out_color.lines()
            .find(|l| l.contains("GIVEN the token is expired:"))
            .expect("GIVEN header line must be present in color output");
        assert!(
            given_line.contains(DIM),
            "GIVEN header must carry DIM code ({:?}) when all nested clauses pass, got: {:?}",
            DIM, given_line
        );

        // BOLD must not appear on the GIVEN header when all children pass
        assert!(
            !given_line.contains(BOLD),
            "GIVEN header must NOT be bold when all nested clauses pass, got: {:?}",
            given_line
        );

        // Without color: no ANSI codes should appear in the header at all
        let given_line_plain = out_nocolor.lines()
            .find(|l| l.contains("GIVEN the token is expired:"))
            .expect("GIVEN header line must be present in plain output");
        assert!(
            !given_line_plain.contains("\x1b["),
            "plain output must contain no ANSI escape sequences, got: {:?}",
            given_line_plain
        );
    }