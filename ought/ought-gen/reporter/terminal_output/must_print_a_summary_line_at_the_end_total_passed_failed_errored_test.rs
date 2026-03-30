/// MUST print a summary line at the end: total passed, failed, errored, by severity
    #[test]
    fn test_reporter__terminal_output__must_print_a_summary_line_at_the_end_total_passed_failed_errored() {
        let files = vec![SpecFile {
            path: "spec.md",
            sections: vec![Section {
                name: "S",
                clauses: vec![
                    Clause { keyword: Keyword::Must,   text: "p1", status: ClauseStatus::Passed  },
                    Clause { keyword: Keyword::Must,   text: "p2", status: ClauseStatus::Passed  },
                    Clause { keyword: Keyword::Must,   text: "f1", status: ClauseStatus::Failed  },
                    Clause { keyword: Keyword::Should, text: "e1", status: ClauseStatus::Errored },
                ],
            }],
        }];
        let out = render(&files, &RenderOpts { use_color: false, is_tty: false });

        // Summary must be the last line (after all clause lines).
        let last_line = out.lines().last().expect("output must not be empty");
        assert!(last_line.to_lowercase().contains("summary"),
            "last line must be a summary line, got: {last_line:?}");

        // Summary must contain correct aggregate counts.
        assert!(last_line.contains("2 passed"),
            "summary must state '2 passed', got: {last_line:?}");
        assert!(last_line.contains("1 failed"),
            "summary must state '1 failed', got: {last_line:?}");
        assert!(last_line.contains("1 errored") || last_line.contains("1 error"),
            "summary must state errored count, got: {last_line:?}");

        // Clause content must appear *before* the summary.
        let pos_clause  = out.find("p1").expect("clause text not found");
        let pos_summary = out.rfind(last_line).expect("summary line not found");
        assert!(pos_clause < pos_summary, "clause content must appear before the summary line");
    }