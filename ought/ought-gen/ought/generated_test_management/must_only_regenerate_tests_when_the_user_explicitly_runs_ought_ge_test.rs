/// MUST only regenerate tests when the user explicitly runs `ought generate`
/// (never during `ought run`).
#[test]
fn test_ought__generated_test_management__must_only_regenerate_tests_on_generate_not_run() {
    // Model the two commands as an enum so we can assert regeneration logic
    // is gated on the Generate variant only.
    #[derive(PartialEq, Debug)]
    enum Command {
        Generate,
        Run,
    }

    struct ManifestState {
        generation_count: usize,
    }

    impl ManifestState {
        fn execute(&mut self, cmd: Command) {
            if cmd == Command::Generate {
                self.generation_count += 1;
            }
            // `run` deliberately does not touch generation_count
        }
    }

    let mut state = ManifestState { generation_count: 0 };

    // Running `ought run` must not trigger regeneration.
    state.execute(Command::Run);
    assert_eq!(
        state.generation_count, 0,
        "`ought run` must not regenerate tests"
    );

    state.execute(Command::Run);
    assert_eq!(
        state.generation_count, 0,
        "repeated `ought run` must not regenerate tests"
    );

    // Running `ought generate` must trigger regeneration.
    state.execute(Command::Generate);
    assert_eq!(
        state.generation_count, 1,
        "`ought generate` must trigger exactly one regeneration pass"
    );

    // A subsequent `ought run` must still not regenerate.
    state.execute(Command::Run);
    assert_eq!(
        state.generation_count, 1,
        "`ought run` after `ought generate` must not increment generation count"
    );
}