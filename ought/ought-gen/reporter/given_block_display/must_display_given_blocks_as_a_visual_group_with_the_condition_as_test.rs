/// MUST display GIVEN blocks as a visual group with the condition as a header line
    #[test]
    fn test_reporter__given_block_display__must_display_given_blocks_as_a_visual_group_with_the_condition_as() {
        let block = GivenBlock {
            condition: "the user is authenticated",
            children: vec![
                Child { keyword: "MUST",   text: "return their profile data",            status: Status::Passed },
                Child { keyword: "MUST",   text: "NOT return other users' private data", status: Status::Failed },
            ],
        };

        let out = render_given(&block, 1, false);

        // The GIVEN condition must appear as the first line (the header)
        let first_line = out.lines().next().expect("output must have at least one line");
        assert!(
            first_line.contains("GIVEN the user is authenticated:"),
            "first line must be 'GIVEN <condition>:' header, got: {:?}",
            first_line
        );

        // Both child clauses must follow the GIVEN header in the output
        let header_pos  = out.find("GIVEN the user is authenticated:").unwrap();
        let clause1_pos = out.find("return their profile data").unwrap();
        let clause2_pos = out.find("NOT return other users' private data").unwrap();
        assert!(header_pos < clause1_pos, "GIVEN header must precede first child clause");
        assert!(header_pos < clause2_pos, "GIVEN header must precede second child clause");

        // The group must show status indicators for each child
        assert!(out.contains("✓"), "passed child clause must carry a pass indicator");
        assert!(out.contains("✗"), "failed child clause must carry a fail indicator");

        // The second GIVEN block is independent — verify two separate headers render correctly
        let block2 = GivenBlock {
            condition: "the token is expired",
            children: vec![
                Child { keyword: "MUST",   text: "return 401",                      status: Status::Passed },
                Child { keyword: "SHOULD", text: "include WWW-Authenticate header", status: Status::Passed },
            ],
        };
        let out2 = render_given(&block2, 1, false);
        let first_line2 = out2.lines().next().expect("second block must have output");
        assert!(
            first_line2.contains("GIVEN the token is expired:"),
            "second GIVEN block header must use its own condition, got: {:?}",
            first_line2
        );
    }