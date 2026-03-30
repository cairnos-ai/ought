/// MUST use a distinct indicator for OTHERWISE results: `↳` prefix to show the fallback relationship
    #[test]
    fn test_reporter__otherwise_chain_display__must_use_a_distinct_indicator_for_otherwise_results_prefix_to_sho() {
        let obligation = Obligation {
            keyword: "MUST",
            text: "respond within 200ms",
            status: Status::Failed,
            otherwise: vec![
                OtherwiseClause { keyword: "OTHERWISE", text: "return a cached response", status: Status::Passed  },
                OtherwiseClause { keyword: "OTHERWISE", text: "return 504",               status: Status::Skipped },
            ],
        };

        let out = render_chain(&obligation, false, 0);

        // Every OTHERWISE line must include the ↳ prefix
        for text in &["return a cached response", "return 504"] {
            let ow_line = out.lines()
                .find(|l| l.contains(text))
                .unwrap_or_else(|| panic!("OTHERWISE clause {:?} must be in output", text));
            assert!(
                ow_line.contains('↳'),
                "OTHERWISE line must carry the ↳ prefix, got: {:?}", ow_line,
            );
            // ↳ must be the first non-whitespace character on the OTHERWISE line
            let trimmed = ow_line.trim_start();
            assert!(
                trimmed.starts_with('↳'),
                "↳ must lead the OTHERWISE line (before icon and text), got: {:?}", trimmed,
            );
        }

        // The parent obligation line must NOT carry ↳
        let parent_line = out.lines()
            .find(|l| l.contains("respond within 200ms"))
            .expect("parent clause must be in output");
        assert!(
            !parent_line.contains('↳'),
            "parent clause line must NOT carry ↳, got: {:?}", parent_line,
        );
    }