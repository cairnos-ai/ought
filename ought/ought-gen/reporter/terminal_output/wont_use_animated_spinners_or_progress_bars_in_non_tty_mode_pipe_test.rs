/// WONT use animated spinners or progress bars in non-TTY mode (pipe-friendly)
    #[test]
    fn test_reporter__terminal_output__wont_use_animated_spinners_or_progress_bars_in_non_tty_mode_pipe() {
        let files = vec![SpecFile {
            path: "spec.md",
            sections: vec![Section {
                name: "S",
                clauses: vec![
                    Clause { keyword: Keyword::Must,   text: "clause a", status: ClauseStatus::Passed },
                    Clause { keyword: Keyword::Must,   text: "clause b", status: ClauseStatus::Failed },
                    Clause { keyword: Keyword::Should, text: "clause c", status: ClauseStatus::Errored },
                ],
            }],
        }];
        let out = render(&files, &RenderOpts { use_color: false, is_tty: false });

        // Output must not be empty — we still produce results.
        assert!(!out.trim().is_empty(), "non-TTY output must not be empty");

        // Braille spinner characters must be absent.
        let braille_spinners = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        for ch in &braille_spinners {
            assert!(!out.contains(*ch),
                "non-TTY output must not contain braille spinner char '{ch}'");
        }

        // Rotating-arc spinner characters must be absent.
        let arc_spinners = ['◐', '◓', '◑', '◒'];
        for ch in &arc_spinners {
            assert!(!out.contains(*ch),
                "non-TTY output must not contain arc spinner char '{ch}'");
        }

        // Block-fill progress-bar characters must be absent.
        let progress_bar_chars = ['█', '░', '▓', '▒', '▐', '▌'];
        for ch in &progress_bar_chars {
            assert!(!out.contains(*ch),
                "non-TTY output must not contain progress bar character '{ch}'");
        }

        // ANSI cursor-control escapes used by in-place spinner animations must be absent.
        assert!(!out.contains("\x1b[?25l"), "cursor-hide escape must not appear in non-TTY output");
        assert!(!out.contains("\x1b[1A"),   "cursor-up escape must not appear in non-TTY output");
        assert!(!out.contains("\x1b[2K"),   "erase-line escape must not appear in non-TTY output");
        assert!(!out.contains("\x1b[1G"),   "carriage-return escape must not appear in non-TTY output");
    }
}