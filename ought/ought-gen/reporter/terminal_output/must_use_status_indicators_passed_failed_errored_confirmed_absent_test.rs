/// MUST use status indicators: `✓` passed, `✗` failed, `!` errored, `⊘` confirmed absent (WONT), `~` skipped
    #[test]
    fn test_reporter__terminal_output__must_use_status_indicators_passed_failed_errored_confirmed_absent() {
        let files = vec![SpecFile {
            path: "spec.md",
            sections: vec![Section {
                name: "All statuses",
                clauses: vec![
                    Clause { keyword: Keyword::Must,   text: "clause-passed",  status: ClauseStatus::Passed  },
                    Clause { keyword: Keyword::Must,   text: "clause-failed",  status: ClauseStatus::Failed  },
                    Clause { keyword: Keyword::Must,   text: "clause-errored", status: ClauseStatus::Errored },
                    Clause { keyword: Keyword::Wont,   text: "clause-absent",  status: ClauseStatus::Absent  },
                    Clause { keyword: Keyword::Should, text: "clause-skipped", status: ClauseStatus::Skipped },
                ],
            }],
        }];
        let out = render(&files, &RenderOpts { use_color: false, is_tty: false });

        // Verify each indicator appears somewhere in the output.
        assert!(out.contains("✓"), "passed indicator '✓' must appear in output");
        assert!(out.contains("✗"), "failed indicator '✗' must appear in output");
        assert!(out.contains("!"),  "errored indicator '!' must appear in output");
        assert!(out.contains("⊘"), "absent indicator '⊘' must appear in output");
        assert!(out.contains("~"),  "skipped indicator '~' must appear in output");

        // Verify each indicator is on the *correct* clause's line.
        for line in out.lines() {
            if line.contains("clause-passed")  { assert!(line.contains("✓"), "passed clause line must show ✓, got: {line:?}"); }
            if line.contains("clause-failed")  { assert!(line.contains("✗"), "failed clause line must show ✗, got: {line:?}"); }
            if line.contains("clause-errored") { assert!(line.contains("!"),  "errored clause line must show !, got: {line:?}"); }
            if line.contains("clause-absent")  { assert!(line.contains("⊘"), "absent clause line must show ⊘, got: {line:?}"); }
            if line.contains("clause-skipped") { assert!(line.contains("~"),  "skipped clause line must show ~, got: {line:?}"); }
        }
    }