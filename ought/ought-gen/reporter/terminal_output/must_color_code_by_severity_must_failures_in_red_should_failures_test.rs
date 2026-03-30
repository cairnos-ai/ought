/// MUST color-code by severity: MUST failures in red, SHOULD failures in yellow, MAY in dim/gray
    #[test]
    fn test_reporter__terminal_output__must_color_code_by_severity_must_failures_in_red_should_failures() {
        let files = vec![SpecFile {
            path: "spec.md",
            sections: vec![Section {
                name: "Severity",
                clauses: vec![
                    Clause { keyword: Keyword::Must,   text: "must-fail",   status: ClauseStatus::Failed },
                    Clause { keyword: Keyword::Should, text: "should-fail", status: ClauseStatus::Failed },
                    Clause { keyword: Keyword::May,    text: "may-fail",    status: ClauseStatus::Failed },
                ],
            }],
        }];
        let out = render(&files, &RenderOpts { use_color: true, is_tty: true });

        let must_line   = out.lines().find(|l| l.contains("must-fail")  ).expect("MUST failure line missing");
        let should_line = out.lines().find(|l| l.contains("should-fail")).expect("SHOULD failure line missing");
        let may_line    = out.lines().find(|l| l.contains("may-fail")   ).expect("MAY failure line missing");

        assert!(must_line.contains(RED),
            "MUST failures must be colored red ({RED:?}), got: {must_line:?}");
        assert!(should_line.contains(YELLOW),
            "SHOULD failures must be colored yellow ({YELLOW:?}), got: {should_line:?}");
        assert!(may_line.contains(DIM),
            "MAY failures must be dim/gray ({DIM:?}), got: {may_line:?}");

        // Lines must also contain the RESET code so color does not bleed into the next line.
        assert!(must_line.contains(RESET),   "MUST failure line must reset ANSI after the clause");
        assert!(should_line.contains(RESET), "SHOULD failure line must reset ANSI after the clause");
        assert!(may_line.contains(RESET),    "MAY failure line must reset ANSI after the clause");
    }