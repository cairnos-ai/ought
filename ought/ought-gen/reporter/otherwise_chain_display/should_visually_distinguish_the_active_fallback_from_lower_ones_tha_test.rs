/// SHOULD visually distinguish the "active" fallback from lower ones that weren't reached
    /// GIVEN: the parent obligation fails
    /// Hint:
    ///  ✗ MUST  respond within 200ms
    ///  ↳ ✓ OTHERWISE return a cached response
    ///  ↳ ~ OTHERWISE return 504              (not reached — caught above)
    #[test]
    fn test_reporter__otherwise_chain_display__should_visually_distinguish_the_active_fallback_from_lower_ones_tha() {
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

        let active_line      = out.lines().find(|l| l.contains("return a cached response")).unwrap();
        let not_reached_line = out.lines().find(|l| l.contains("return 504")).unwrap();

        // Active fallback: ✓ icon, no "not reached" annotation
        assert!(
            active_line.contains('✓'),
            "active fallback must show ✓ to indicate it handled the failure, got: {:?}",
            active_line,
        );
        assert!(
            !active_line.contains("not reached"),
            "active fallback must NOT carry a 'not reached' annotation, got: {:?}",
            active_line,
        );

        // Lower fallback: ~ icon, explicit "not reached" annotation
        assert!(
            not_reached_line.contains('~'),
            "not-reached fallback must show ~ to indicate it was skipped, got: {:?}",
            not_reached_line,
        );
        assert!(
            not_reached_line.contains("not reached"),
            "not-reached fallback must carry an explanatory annotation, got: {:?}",
            not_reached_line,
        );

        // The two OTHERWISE lines must be textually distinguishable — different icons
        let active_icon      = active_line.trim_start().chars().nth(1); // ↳ <icon>
        let not_reached_icon = not_reached_line.trim_start().chars().nth(1);
        assert_ne!(
            active_icon, not_reached_icon,
            "active and not-reached fallbacks must carry different status icons",
        );

        // With colour: active fallback must NOT be dimmed; not-reached MUST be dimmed
        let out_color = render_chain(&obligation, true, 0);
        let active_colored      = out_color.lines().find(|l| l.contains("return a cached response")).unwrap();
        let not_reached_colored = out_color.lines().find(|l| l.contains("return 504")).unwrap();

        assert!(
            !active_colored.contains(DIM),
            "active fallback must NOT be dimmed, got: {:?}", active_colored,
        );
        assert!(
            not_reached_colored.contains(DIM),
            "not-reached fallback must be dimmed to de-emphasise it, got: {:?}", not_reached_colored,
        );
    }
}
```