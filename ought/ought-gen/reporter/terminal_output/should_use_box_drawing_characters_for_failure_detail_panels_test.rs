/// SHOULD use box-drawing characters for failure detail panels
    #[test]
    fn test_reporter__terminal_output__should_use_box_drawing_characters_for_failure_detail_panels() {
        let files_fail = vec![SpecFile {
            path: "spec.md",
            sections: vec![Section {
                name: "S",
                clauses: vec![
                    Clause { keyword: Keyword::Must, text: "failing clause", status: ClauseStatus::Failed },
                ],
            }],
        }];
        let files_pass = vec![SpecFile {
            path: "spec.md",
            sections: vec![Section {
                name: "S",
                clauses: vec![
                    Clause { keyword: Keyword::Must, text: "passing clause", status: ClauseStatus::Passed },
                ],
            }],
        }];

        let out_fail = render(&files_fail, &RenderOpts { use_color: false, is_tty: false });
        let out_pass = render(&files_pass, &RenderOpts { use_color: false, is_tty: false });

        // Failure detail panel must include box-drawing characters.
        let box_chars: &[char] = &['┌', '─', '│', '└', '┐', '┘', '├', '┤', '┬', '┴'];
        let failure_has_box = box_chars.iter().any(|c| out_fail.contains(*c));
        assert!(
            failure_has_box,
            "failure detail panel must contain box-drawing characters (┌─│└…), got:\n{out_fail}"
        );

        // Check that the panel forms a recognisable top border and left border.
        assert!(out_fail.contains('┌'), "failure panel must have a top-left corner '┌'");
        assert!(out_fail.contains('│'), "failure panel must have a vertical bar '│' for content rows");
        assert!(out_fail.contains('└'), "failure panel must have a bottom-left corner '└'");

        // Passing output must NOT show detail panels at all.
        let passing_has_box = box_chars.iter().any(|c| out_pass.contains(*c));
        assert!(
            !passing_has_box,
            "passing output must not contain failure detail panels, got:\n{out_pass}"
        );
    }