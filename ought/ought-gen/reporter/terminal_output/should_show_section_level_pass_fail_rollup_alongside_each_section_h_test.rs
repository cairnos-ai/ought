/// SHOULD show section-level pass/fail rollup alongside each section header
    #[test]
    fn test_reporter__terminal_output__should_show_section_level_pass_fail_rollup_alongside_each_section_h() {
        let files = vec![SpecFile {
            path: "spec.md",
            sections: vec![
                Section {
                    name: "AllPass",
                    clauses: vec![
                        Clause { keyword: Keyword::Must, text: "a", status: ClauseStatus::Passed },
                        Clause { keyword: Keyword::Must, text: "b", status: ClauseStatus::Passed },
                    ],
                },
                Section {
                    name: "PartialFail",
                    clauses: vec![
                        Clause { keyword: Keyword::Must, text: "c", status: ClauseStatus::Passed },
                        Clause { keyword: Keyword::Must, text: "d", status: ClauseStatus::Failed },
                    ],
                },
            ],
        }];
        let out = render(&files, &RenderOpts { use_color: false, is_tty: false });

        let all_pass_header = out.lines()
            .find(|l| l.contains("AllPass"))
            .expect("AllPass section header missing");
        let partial_fail_header = out.lines()
            .find(|l| l.contains("PartialFail"))
            .expect("PartialFail section header missing");

        // AllPass: 2/2 pass — header should show full rollup.
        assert!(
            all_pass_header.contains("2/2") || all_pass_header.contains("✓"),
            "AllPass header must show full rollup, got: {all_pass_header:?}"
        );

        // PartialFail: 1/2 pass — header should signal at least one failure.
        assert!(
            partial_fail_header.contains("1/2") || partial_fail_header.contains("✗"),
            "PartialFail header must show partial rollup, got: {partial_fail_header:?}"
        );

        // Rollup must appear *on the section header line*, not on a separate line.
        assert!(all_pass_header.contains("##"),
            "rollup must be on the same line as the section header (##), got: {all_pass_header:?}");
        assert!(partial_fail_header.contains("##"),
            "rollup must be on the same line as the section header (##), got: {partial_fail_header:?}");
    }