/// MUST show the full chain status: if the parent passes, OTHERWISE clauses show as `~` (not needed)
    #[test]
    fn test_reporter__otherwise_chain_display__must_show_the_full_chain_status_if_the_parent_passes_otherwise_cl() {
        let obligation = Obligation {
            keyword: "MUST",
            text: "respond within 200ms",
            status: Status::Passed, // parent passes — OTHERWISE chain is not needed
            otherwise: vec![
                OtherwiseClause { keyword: "OTHERWISE", text: "return a cached response", status: Status::Skipped },
                OtherwiseClause { keyword: "OTHERWISE", text: "return 504",               status: Status::Skipped },
            ],
        };

        let out = render_chain(&obligation, false, 0);

        // Parent must show ✓
        let parent_line = out.lines()
            .find(|l| l.contains("respond within 200ms"))
            .expect("parent line must be present");
        assert!(
            parent_line.contains('✓'),
            "passing parent must show ✓, got: {:?}", parent_line,
        );

        // Both OTHERWISE clauses must show ~ and must NOT show ✓ or ✗
        for text in &["return a cached response", "return 504"] {
            let ow_line = out.lines()
                .find(|l| l.contains(text))
                .unwrap_or_else(|| panic!("OTHERWISE clause {:?} must be in output", text));
            assert!(
                ow_line.contains('~'),
                "OTHERWISE must show ~ when parent passes (not needed), got: {:?}", ow_line,
            );
            assert!(
                !ow_line.contains('✓'),
                "OTHERWISE must NOT show ✓ when parent passes, got: {:?}", ow_line,
            );
            assert!(
                !ow_line.contains('✗'),
                "OTHERWISE must NOT show ✗ when parent passes, got: {:?}", ow_line,
            );
            // "not reached" annotation must NOT appear — this is "not needed", a different state
            assert!(
                !ow_line.contains("not reached"),
                "OTHERWISE must not say 'not reached' when parent passed (it's not needed), got: {:?}",
                ow_line,
            );
        }
    }