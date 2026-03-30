/// MUST show each clause with its keyword, text, and pass/fail status
    #[test]
    fn test_reporter__terminal_output__must_show_each_clause_with_its_keyword_text_and_pass_fail_status() {
        let files = vec![SpecFile {
            path: "spec.md",
            sections: vec![Section {
                name: "Auth",
                clauses: vec![
                    Clause { keyword: Keyword::Must,   text: "return a token",  status: ClauseStatus::Passed  },
                    Clause { keyword: Keyword::Should, text: "log the attempt", status: ClauseStatus::Failed  },
                    Clause { keyword: Keyword::May,    text: "set a cookie",    status: ClauseStatus::Skipped },
                ],
            }],
        }];
        let out = render(&files, &RenderOpts { use_color: false, is_tty: false });

        // Each clause line must carry the keyword label, the clause text, and a status indicator.
        let must_line   = out.lines().find(|l| l.contains("return a token") ).expect("MUST clause line missing");
        let should_line = out.lines().find(|l| l.contains("log the attempt")).expect("SHOULD clause line missing");
        let may_line    = out.lines().find(|l| l.contains("set a cookie")   ).expect("MAY clause line missing");

        assert!(must_line.contains("MUST"),   "MUST keyword must appear on clause line");
        assert!(must_line.contains("✓"),      "passed indicator must appear on MUST line");

        assert!(should_line.contains("SHOULD"), "SHOULD keyword must appear on clause line");
        assert!(should_line.contains("✗"),      "failed indicator must appear on SHOULD line");

        assert!(may_line.contains("MAY"), "MAY keyword must appear on clause line");
        assert!(may_line.contains("~"),   "skipped indicator must appear on MAY line");
    }