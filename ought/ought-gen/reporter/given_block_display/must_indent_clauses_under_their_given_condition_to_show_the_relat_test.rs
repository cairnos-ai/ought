/// MUST indent clauses under their GIVEN condition to show the relationship
    #[test]
    fn test_reporter__given_block_display__must_indent_clauses_under_their_given_condition_to_show_the_relat() {
        let block = GivenBlock {
            condition: "the token is expired",
            children: vec![
                Child { keyword: "MUST",   text: "return 401",                      status: Status::Passed },
                Child { keyword: "SHOULD", text: "include WWW-Authenticate header", status: Status::Passed },
            ],
        };

        let out = render_given(&block, 1, false);

        let given_line = out.lines()
            .find(|l| l.contains("GIVEN the token is expired:"))
            .expect("GIVEN header line must be present");

        let child_line = out.lines()
            .find(|l| l.contains("return 401"))
            .expect("child clause line must be present");

        // Leading whitespace on the child line must be strictly greater than on the header
        let given_indent = given_line.len() - given_line.trim_start().len();
        let child_indent = child_line.len() - child_line.trim_start().len();
        assert!(
            child_indent > given_indent,
            "child clause indent ({}) must exceed GIVEN header indent ({})",
            child_indent, given_indent
        );

        // Every child clause must be indented more than the GIVEN header
        let child_texts = ["return 401", "include WWW-Authenticate header"];
        for text in &child_texts {
            let line = out.lines()
                .find(|l| l.contains(text))
                .unwrap_or_else(|| panic!("child clause '{}' not found in output", text));
            let indent = line.len() - line.trim_start().len();
            assert!(
                indent > given_indent,
                "child '{}' (indent={}) must be indented more than GIVEN header (indent={})",
                text, indent, given_indent
            );
        }
    }