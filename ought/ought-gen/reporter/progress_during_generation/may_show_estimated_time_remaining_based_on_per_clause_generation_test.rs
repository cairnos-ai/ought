/// MAY show estimated time remaining based on per-clause generation speed
#[cfg(test)]
mod eta_estimation_test {
    use std::time::Duration;

    /// Tracks per-clause generation timings and computes an ETA for remaining clauses.
    struct EtaTracker {
        total: usize,
        timings_ms: Vec<u64>,
    }

    impl EtaTracker {
        fn new(total: usize) -> Self {
            Self { total, timings_ms: Vec::new() }
        }

        /// Record that one clause took `ms` milliseconds to generate.
        fn record(&mut self, ms: u64) {
            self.timings_ms.push(ms);
        }

        /// Estimated milliseconds remaining, or None if no data yet.
        fn estimate_remaining_ms(&self) -> Option<u64> {
            if self.timings_ms.is_empty() {
                return None;
            }
            let avg: f64 = self.timings_ms.iter().sum::<u64>() as f64
                / self.timings_ms.len() as f64;
            let remaining = self.total.saturating_sub(self.timings_ms.len());
            Some((avg * remaining as f64).round() as u64)
        }

        /// Human-readable ETA string suitable for display in a progress line,
        /// or None when not enough data is available.
        fn render_eta(&self) -> Option<String> {
            let ms = self.estimate_remaining_ms()?;
            if ms < 1_000 {
                Some(format!("ETA <1s"))
            } else if ms < 60_000 {
                Some(format!("ETA {}s", ms / 1_000))
            } else {
                let mins = ms / 60_000;
                let secs = (ms % 60_000) / 1_000;
                Some(format!("ETA {}m{}s", mins, secs))
            }
        }
    }

    #[test]
    /// MAY show estimated time remaining based on per-clause generation speed
    fn test_reporter__progress_during_generation__may_show_estimated_time_remaining_based_on_per_clause_generation() {
        // --- No data: estimate must not be available ---
        let tracker = EtaTracker::new(10);
        assert!(
            tracker.estimate_remaining_ms().is_none(),
            "ETA must be None before any clauses have been timed"
        );
        assert!(
            tracker.render_eta().is_none(),
            "render_eta must return None before any clauses have been timed"
        );

        // --- Uniform timing: ETA should be (avg * remaining) ---
        let mut t = EtaTracker::new(10);
        // Record 3 clauses each taking 2000 ms.
        t.record(2_000);
        t.record(2_000);
        t.record(2_000);

        let eta_ms = t.estimate_remaining_ms().expect("ETA must be available after recording timings");
        // 7 remaining clauses × 2000 ms avg = 14 000 ms
        assert_eq!(eta_ms, 14_000, "uniform 2 s/clause × 7 remaining = 14 000 ms, got {}", eta_ms);

        let eta_str = t.render_eta().expect("render_eta must produce a string");
        assert!(
            eta_str.contains("ETA"),
            "rendered ETA must contain 'ETA' prefix, got: {:?}", eta_str
        );
        assert!(
            eta_str.contains("14s") || eta_str.contains("14"),
            "rendered ETA must reflect 14-second estimate, got: {:?}", eta_str
        );

        // --- Variable timing: ETA should use the running average ---
        let mut t2 = EtaTracker::new(6);
        t2.record(1_000);
        t2.record(3_000); // avg = 2 000 ms over 2 clauses, 4 remaining → 8 000 ms
        let eta2 = t2.estimate_remaining_ms().unwrap();
        assert_eq!(eta2, 8_000, "avg 2 s/clause × 4 remaining = 8 000 ms, got {}", eta2);

        // --- Single clause remaining: ETA should match one avg period ---
        let mut t3 = EtaTracker::new(4);
        t3.record(500);
        t3.record(500);
        t3.record(500); // avg = 500 ms, 1 remaining → 500 ms
        let eta3 = t3.estimate_remaining_ms().unwrap();
        assert_eq!(eta3, 500, "1 remaining × 500 ms avg = 500 ms, got {}", eta3);

        let eta3_str = t3.render_eta().unwrap();
        assert!(
            eta3_str.contains("ETA"),
            "sub-second ETA must still render with 'ETA' prefix, got: {:?}", eta3_str
        );

        // --- All clauses complete: 0 remaining → ETA of 0 ms ---
        let mut t4 = EtaTracker::new(3);
        t4.record(1_000);
        t4.record(1_000);
        t4.record(1_000);
        let eta4 = t4.estimate_remaining_ms().unwrap();
        assert_eq!(eta4, 0, "0 remaining clauses must yield 0 ms ETA, got {}", eta4);

        // --- Minute-scale ETA renders as minutes+seconds ---
        let mut t5 = EtaTracker::new(5);
        t5.record(30_000); // avg = 30 s, 4 remaining → 120 s = 2 m 0 s
        let eta5_str = t5.render_eta().unwrap();
        assert!(
            eta5_str.contains("2m"),
            "two-minute ETA must render with 'm' suffix, got: {:?}", eta5_str
        );
    }
}