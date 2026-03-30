/// MUST show which OTHERWISE level caught the failure
    /// GIVEN: the parent obligation fails
    #[test]
    fn test_reporter__otherwise_chain_display__must_show_which_otherwise_level_caught_the_failure() {
        // Scenario A: the first OTHERWISE catches the failure
        let caught_at_first = Obligation {
            keyword: "MUST",
            text: "respond within 200ms",
            status: Status::Failed,
            otherwise: vec![
                OtherwiseClause { keyword: "OTHERWISE", text: "return a cached response", status: Status::Passed  },
                OtherwiseClause { keyword: "OTHERWISE", text: "return 504",               status: Status::Skipped },
            ],
        };

        let out_a = render_chain(&caught_at_first, false, 0);

        let ow1_a = out_a.lines().find(|l| l.contains("return a cached response")).unwrap();
        let ow2_a = out_a.lines().find(|l| l.contains("return 504")).unwrap();

        assert!(
            ow1_a.contains('✓'),
            "first OTHERWISE (caught the failure) must show ✓, got: {:?}", ow1_a,
        );
        assert!(
            !ow1_a.contains('~'),
            "first OTHERWISE (active fallback) must NOT show ~, got: {:?}", ow1_a,
        );
        assert!(
            ow2_a.contains('~'),
            "second OTHERWISE (not reached) must show ~, got: {:?}", ow2_a,
        );

        // Scenario B: the first OTHERWISE also fails; the second level catches it
        let caught_at_second = Obligation {
            keyword: "MUST",
            text: "respond within 200ms",
            status: Status::Failed,
            otherwise: vec![
                OtherwiseClause { keyword: "OTHERWISE", text: "return a cached response", status: Status::Failed },
                OtherwiseClause { keyword: "OTHERWISE", text: "return 504",               status: Status::Passed },
            ],
        };

        let out_b = render_chain(&caught_at_second, false, 0);

        let ow1_b = out_b.lines().find(|l| l.contains("return a cached response")).unwrap();
        let ow2_b = out_b.lines().find(|l| l.contains("return 504")).unwrap();

        assert!(
            ow1_b.contains('✗'),
            "first OTHERWISE (also failed) must show ✗, got: {:?}", ow1_b,
        );
        assert!(
            ow2_b.contains('✓'),
            "second OTHERWISE (caught the failure) must show ✓, got: {:?}", ow2_b,
        );
        assert!(
            !ow2_b.contains('~'),
            "second OTHERWISE (active fallback) must NOT show ~, got: {:?}", ow2_b,
        );
    }