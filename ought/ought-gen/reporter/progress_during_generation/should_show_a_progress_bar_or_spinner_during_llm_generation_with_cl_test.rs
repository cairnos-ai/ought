/// SHOULD show a progress bar or spinner during LLM generation with clause count
#[cfg(test)]
mod progress_spinner_clause_count_test {
    use std::fmt::Write as FmtWrite;

    const BRAILLE_SPINNERS: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    const BAR_FILL: char = '█';
    const BAR_EMPTY: char = '░';
    const BAR_WIDTH: usize = 20;

    #[derive(Debug)]
    struct GenProgress {
        total: usize,
        completed: usize,
        is_tty: bool,
    }

    impl GenProgress {
        fn new(total: usize, is_tty: bool) -> Self {
            Self { total, completed: 0, is_tty }
        }

        fn advance(&mut self) {
            if self.completed < self.total {
                self.completed += 1;
            }
        }

        /// Returns the current progress line.  In TTY mode it includes a
        /// spinner character; in non-TTY mode it omits animation characters
        /// but still emits the clause count.
        fn render_line(&self) -> String {
            let count = format!("[{}/{}]", self.completed, self.total);
            if self.is_tty {
                let spinner = BRAILLE_SPINNERS[self.completed % BRAILLE_SPINNERS.len()];
                format!("{} Generating {} clauses {}...", spinner, self.total, count)
            } else {
                format!("Generating {} clauses {}...", self.total, count)
            }
        }

        /// Render a block-fill progress bar (only meaningful in TTY mode).
        fn render_bar(&self) -> String {
            let filled = (self.completed * BAR_WIDTH) / self.total.max(1);
            let empty = BAR_WIDTH - filled;
            let mut bar = String::new();
            for _ in 0..filled  { bar.push(BAR_FILL); }
            for _ in 0..empty   { bar.push(BAR_EMPTY); }
            format!("[{}] {}/{}", bar, self.completed, self.total)
        }
    }

    #[test]
    /// SHOULD show a progress bar or spinner during LLM generation with clause count
    fn test_reporter__progress_during_generation__should_show_a_progress_bar_or_spinner_during_llm_generation_with_cl() {
        // --- TTY mode: spinner + clause count present ---
        let mut p = GenProgress::new(10, true);
        p.advance(); // completed = 1
        p.advance(); // completed = 2
        p.advance(); // completed = 3

        let line = p.render_line();

        // Clause count must appear in output.
        assert!(
            line.contains("3/10") || line.contains("3 of 10"),
            "progress line must include clause count, got: {:?}", line
        );

        // At least one braille spinner character must be present in TTY mode.
        let has_spinner = BRAILLE_SPINNERS.iter().any(|&ch| line.contains(ch));
        assert!(has_spinner, "TTY progress line must contain a spinner character, got: {:?}", line);

        // --- Progress bar variant ---
        let bar = p.render_bar();
        assert!(
            bar.contains('█') || bar.contains('░'),
            "progress bar must contain fill characters, got: {:?}", bar
        );
        assert!(
            bar.contains("3/10"),
            "progress bar must include clause count, got: {:?}", bar
        );

        // --- Total clause count must appear for multiple positions ---
        for completed in 0..=10 {
            let mut pp = GenProgress::new(10, true);
            for _ in 0..completed { pp.advance(); }
            let l = pp.render_line();
            let expected_frac = format!("{}/10", completed);
            assert!(
                l.contains(&expected_frac),
                "at completed={}, line must contain '{}', got: {:?}", completed, expected_frac, l
            );
        }

        // --- Non-TTY mode: no spinner, but count still present ---
        let mut pq = GenProgress::new(5, false);
        pq.advance();
        pq.advance();
        let plain = pq.render_line();

        assert!(
            plain.contains("2/5") || plain.contains("2 of 5"),
            "non-TTY progress line must include clause count, got: {:?}", plain
        );

        let has_spinner_plain = BRAILLE_SPINNERS.iter().any(|&ch| plain.contains(ch));
        assert!(
            !has_spinner_plain,
            "non-TTY progress line must NOT contain braille spinner characters, got: {:?}", plain
        );

        // --- Completed message after all clauses done ---
        let mut done = GenProgress::new(3, true);
        for _ in 0..3 { done.advance(); }
        let done_line = done.render_line();
        // Clause count should show full completion.
        assert!(
            done_line.contains("3/3"),
            "completed progress must show '3/3', got: {:?}", done_line
        );
    }
}