/// MUST display OTHERWISE clauses indented under their parent obligation
    #[test]
    fn test_reporter__otherwise_chain_display__must_display_otherwise_clauses_indented_under_their_parent_obliga() {
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

        // OTHERWISE clause text must appear after the parent clause in the output
        let parent_pos = out.find("respond within 200ms").expect("parent clause must be in output");
        let ow1_pos    = out.find("return a cached response").expect("first OTHERWISE must be in output");
        let ow2_pos    = out.find("return 504").expect("second OTHERWISE must be in output");
        assert!(parent_pos < ow1_pos, "parent must precede first OTHERWISE clause");
        assert!(parent_pos < ow2_pos, "parent must precede second OTHERWISE clause");

        // OTHERWISE lines must carry strictly more leading whitespace than the parent
        let parent_line = out.lines().find(|l| l.contains("respond within 200ms")).unwrap();
        let ow1_line    = out.lines().find(|l| l.contains("return a cached response")).unwrap();
        let ow2_line    = out.lines().find(|l| l.contains("return 504")).unwrap();

        let parent_indent = parent_line.len() - parent_line.trim_start().len();
        let ow1_indent    = ow1_line.len()    - ow1_line.trim_start().len();
        let ow2_indent    = ow2_line.len()    - ow2_line.trim_start().len();

        assert!(
            ow1_indent > parent_indent,
            "first OTHERWISE (indent={}) must be indented more than parent (indent={})",
            ow1_indent, parent_indent,
        );
        assert!(
            ow2_indent > parent_indent,
            "second OTHERWISE (indent={}) must be indented more than parent (indent={})",
            ow2_indent, parent_indent,
        );

        // All OTHERWISE lines at the same depth must be uniformly indented
        assert_eq!(
            ow1_indent, ow2_indent,
            "all OTHERWISE clauses at the same depth must share the same indentation",
        );
    }