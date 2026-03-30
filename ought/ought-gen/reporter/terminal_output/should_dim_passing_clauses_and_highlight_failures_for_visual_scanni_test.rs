/// SHOULD dim passing clauses and highlight failures for visual scanning
    #[test]
    fn test_reporter__terminal_output__should_dim_passing_clauses_and_highlight_failures_for_visual_scanni() {
        let files = vec![SpecFile {
            path: "spec.md",
            sections: vec![Section {
                name: "S",
                clauses: vec![
                    Clause { keyword: Keyword::Must, text: "passing clause", status: ClauseStatus::Passed },
                    Clause { keyword: Keyword::Must, text: "failing clause", status: ClauseStatus::Failed },
                ],
            }],
        }];
        let out_color   = render(&files, &RenderOpts { use_color: true,  is_tty: true  });
        let out_nocolor = render(&files, &RenderOpts { use_color: false, is_tty: false });

        // With color: passing clause line must carry the dim code.
        let passing_line = out_color.lines()
            .find(|l| l.contains("passing clause"))
            .expect("passing clause line missing in color output");
        assert!(
            passing_line.contains(DIM),
            "passing clause must be dimmed ({DIM:?}), got: {passing_line:?}"
        );

        // With color: failing MUST clause must be highlighted (red or bold).
        let failing_line = out_color.lines()
            .find(|l| l.contains("failing clause"))
            .expect("failing clause line missing in color output");
        assert!(
            failing_line.contains(RED) || failing_line.contains(BOLD),
            "failing clause must be highlighted (red or bold), got: {failing_line:?}"
        );

        // Passing and failing lines must have visually distinct styling codes.
        assert_ne!(
            passing_line.contains(RED),
            failing_line.contains(DIM) && !failing_line.contains(RED),
            "passing and failing clauses must not share identical styling"
        );

        // Without color: no ANSI escape codes should appear at all.
        assert!(
            !out_nocolor.contains("\x1b["),
            "color-disabled output must contain no ANSI escape sequences"
        );
    }