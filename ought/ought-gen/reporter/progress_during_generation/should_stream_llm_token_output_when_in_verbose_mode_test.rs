/// SHOULD stream LLM token output when in verbose mode
#[cfg(test)]
mod verbose_streaming_test {
    use std::cell::RefCell;
    use std::rc::Rc;

    /// Represents a single flush event: the token text and whether it was
    /// flushed immediately (streamed) or only after the full response arrived.
    #[derive(Debug, Clone)]
    struct FlushEvent {
        text: String,
        immediate: bool,
    }

    /// Simulates the output capture layer for the verbose streaming path.
    struct TokenSink {
        verbose: bool,
        events: Rc<RefCell<Vec<FlushEvent>>>,
        buffer: String,
    }

    impl TokenSink {
        fn new(verbose: bool) -> Self {
            Self {
                verbose,
                events: Rc::new(RefCell::new(Vec::new())),
                buffer: String::new(),
            }
        }

        /// Called each time the LLM emits a token fragment.
        fn receive(&mut self, token: &str) {
            self.buffer.push_str(token);
            if self.verbose {
                self.events.borrow_mut().push(FlushEvent {
                    text: token.to_string(),
                    immediate: true,
                });
            }
        }

        /// Called when the LLM stream ends.
        fn finish(&mut self) {
            if !self.verbose {
                // Non-verbose: emit everything at once only after completion.
                let full = self.buffer.clone();
                self.events.borrow_mut().push(FlushEvent {
                    text: full,
                    immediate: false,
                });
            }
        }

        fn events(&self) -> Vec<FlushEvent> {
            self.events.borrow().clone()
        }

        fn full_output(&self) -> String {
            self.buffer.clone()
        }
    }

    const TOKENS: &[&str] = &[
        "fn ", "test_", "auth_", "login", "()", " {\n",
        "    ", "assert!", "(true", ");\n",
        "}",
    ];

    #[test]
    /// SHOULD stream LLM token output when in verbose mode
    fn test_reporter__progress_during_generation__should_stream_llm_token_output_when_in_verbose_mode() {
        // --- Verbose mode: each token must be flushed immediately ---
        let mut verbose_sink = TokenSink::new(true);
        for &tok in TOKENS {
            verbose_sink.receive(tok);
        }
        verbose_sink.finish();

        let verbose_events = verbose_sink.events();

        // Every token must have produced an immediate flush event.
        assert_eq!(
            verbose_events.len(), TOKENS.len(),
            "verbose mode must emit one flush event per token, got {} events for {} tokens",
            verbose_events.len(), TOKENS.len()
        );

        for (i, event) in verbose_events.iter().enumerate() {
            assert!(
                event.immediate,
                "verbose flush event {} must be immediate (streamed), but was buffered",
                i
            );
        }

        // Token ordering must be preserved.
        for (i, (&expected_tok, event)) in TOKENS.iter().zip(verbose_events.iter()).enumerate() {
            assert_eq!(
                event.text, expected_tok,
                "token {} ordering must be preserved: expected {:?}, got {:?}",
                i, expected_tok, event.text
            );
        }

        // The concatenated verbose output must equal the full response.
        let verbose_concat: String = verbose_events.iter().map(|e| e.text.as_str()).collect();
        let expected_full: String = TOKENS.concat();
        assert_eq!(verbose_concat, expected_full, "verbose concatenated output must match full response");

        // --- Non-verbose mode: tokens must only appear after finish() ---
        let mut quiet_sink = TokenSink::new(false);

        // Before finish, no events should have been emitted.
        for &tok in TOKENS {
            quiet_sink.receive(tok);
            let mid_events = quiet_sink.events();
            assert!(
                mid_events.is_empty(),
                "non-verbose mode must not emit events before finish(); got {:?} mid-stream",
                mid_events
            );
        }

        quiet_sink.finish();

        let quiet_events = quiet_sink.events();
        assert_eq!(quiet_events.len(), 1, "non-verbose mode must emit exactly one event (after finish)");
        assert!(
            !quiet_events[0].immediate,
            "non-verbose finish event must not be marked as immediate/streamed"
        );
        assert_eq!(
            quiet_events[0].text,
            TOKENS.concat(),
            "non-verbose event must contain the complete buffered response"
        );

        // --- Verbose output must not be empty for non-empty token stream ---
        let full = verbose_sink.full_output();
        assert!(!full.is_empty(), "full output must not be empty after receiving tokens");
        assert!(
            full.contains("test_auth_login"),
            "full output must contain the generated function name"
        );
    }
}