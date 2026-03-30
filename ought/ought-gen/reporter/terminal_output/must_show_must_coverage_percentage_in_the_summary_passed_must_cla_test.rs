/// MUST show MUST coverage percentage in the summary (passed MUST clauses / total MUST clauses)
    #[test]
    fn test_reporter__terminal_output__must_show_must_coverage_percentage_in_the_summary_passed_must_cla() {
        // 2 of 3 MUST clauses pass → 66 %.
        let files_partial = vec![SpecFile {
            path: "spec.md",
            sections: vec![Section {
                name: "S",
                clauses: vec![
                    Clause { keyword: Keyword::Must,   text: "m1", status: ClauseStatus::Passed },
                    Clause { keyword: Keyword::Must,   text: "m2", status: ClauseStatus::Passed },
                    Clause { keyword: Keyword::Must,   text: "m3", status: ClauseStatus::Failed },
                    Clause { keyword: Keyword::Should, text: "s1", status: ClauseStatus::Passed }, // SHOULD must not affect MUST %
                ],
            }],
        }];
        let out_partial = render(&files_partial, &RenderOpts { use_color: false, is_tty: false });
        assert!(out_partial.contains("66%") || out_partial.contains("66 %"),
            "summary must show MUST coverage of 66%, got:\n{out_partial}");

        // All MUST pass → 100 %.
        let files_all_pass = vec![SpecFile {
            path: "spec.md",
            sections: vec![Section {
                name: "S",
                clauses: vec![
                    Clause { keyword: Keyword::Must, text: "m1", status: ClauseStatus::Passed },
                    Clause { keyword: Keyword::Must, text: "m2", status: ClauseStatus::Passed },
                ],
            }],
        }];
        let out_full = render(&files_all_pass, &RenderOpts { use_color: false, is_tty: false });
        assert!(out_full.contains("100%"),
            "summary must show 100% MUST coverage when all MUST clauses pass, got:\n{out_full}");

        // The label must make it clear the percentage refers to MUST clauses.
        let summary_partial = out_partial.lines().last().unwrap_or("");
        assert!(summary_partial.to_uppercase().contains("MUST"),
            "summary line must mention MUST in coverage label, got: {summary_partial:?}");
    }