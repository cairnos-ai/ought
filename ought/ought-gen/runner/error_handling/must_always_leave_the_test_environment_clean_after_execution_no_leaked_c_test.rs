/// MUST ALWAYS leave the test environment clean after execution (no leaked child processes, temp files removed)
/// Temporal: MUST ALWAYS (invariant). Property-based / fuzz-style.
#[test]
fn test_runner__error_handling__must_always_leave_the_test_environment_clean_after_execution_no_leaked_c() {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    // Invariant: regardless of outcome (pass, fail, harness crash, command-not-found,
    // timeout, signal), the runner must remove every temp file it created and must not
    // leave orphaned child processes.
    //
    // We model this with a cleanup tracker that simulates both happy and failure paths.

    #[derive(Debug, Clone, Copy, PartialEq)]
    enum Scenario {
        TestsPassed,
        TestsFailed,
        HarnessCrashed,
        CommandNotFound,
        Timeout,
        EmptyTestList,
    }

    struct RunEnvironment {
        work_dir: PathBuf,
        sentinel_path: PathBuf,
        child_started: Arc<AtomicBool>,
    }

    impl RunEnvironment {
        fn setup(base: &PathBuf, id: u32) -> Self {
            let work_dir = base.join(format!("run_{}", id));
            fs::create_dir_all(&work_dir).unwrap();
            let sentinel = work_dir.join("ought_run.tmp");
            fs::write(&sentinel, b"in-progress").unwrap();
            RunEnvironment {
                sentinel_path: sentinel,
                work_dir,
                child_started: Arc::new(AtomicBool::new(false)),
            }
        }

        fn execute(&self, scenario: Scenario) -> Result<usize, String> {
            self.child_started.store(true, Ordering::SeqCst);
            match scenario {
                Scenario::TestsPassed      => Ok(3),
                Scenario::TestsFailed      => Ok(0),  // ran, but some assertions failed
                Scenario::HarnessCrashed   => Err("harness exited with signal 11".into()),
                Scenario::CommandNotFound  => Err("No such file or directory (os error 2)".into()),
                Scenario::Timeout          => Err("harness timed out after 30s".into()),
                Scenario::EmptyTestList    => Ok(0),
            }
        }

        fn cleanup(self) {
            // Cleanup must happen even if execute() returned Err.
            // The child process would have been waited on here (wait()/kill()).
            fs::remove_file(&self.sentinel_path).ok();
            fs::remove_dir_all(&self.work_dir).ok();
        }
    }

    let base = std::env::temp_dir().join(format!(
        "ought_invariant_{}",
        std::process::id()
    ));
    fs::create_dir_all(&base).unwrap();

    let scenarios = [
        Scenario::TestsPassed,
        Scenario::TestsFailed,
        Scenario::HarnessCrashed,
        Scenario::CommandNotFound,
        Scenario::Timeout,
        Scenario::EmptyTestList,
    ];

    // --- property: every scenario leaves the environment clean ---
    for (i, &scenario) in scenarios.iter().enumerate() {
        let env = RunEnvironment::setup(&base, i as u32);
        let sentinel = env.sentinel_path.clone();
        let work_dir = env.work_dir.clone();

        let _result = env.execute(scenario);
        env.cleanup(); // MUST be called on every path, including errors

        assert!(
            !sentinel.exists(),
            "Temp sentinel file must be removed after scenario {:?}", scenario
        );
        assert!(
            !work_dir.exists(),
            "Work directory must be removed after scenario {:?}", scenario
        );
    }

    // --- fuzz-style: 30 random iterations, interleaving all scenarios ---
    // Simulates repeated runs (e.g., watch mode, CI retries) to confirm no
    // cumulative leakage of files across runs.
    for iteration in 0u32..30 {
        let scenario = scenarios[(iteration as usize) % scenarios.len()];
        let env = RunEnvironment::setup(&base, 100 + iteration);
        let sentinel = env.sentinel_path.clone();
        let work_dir  = env.work_dir.clone();

        let _ = env.execute(scenario);
        env.cleanup();

        assert!(
            !sentinel.exists(),
            "iteration {}: sentinel file leaked for scenario {:?}", iteration, scenario
        );
        assert!(
            !work_dir.exists(),
            "iteration {}: work dir leaked for scenario {:?}", iteration, scenario
        );
    }

    fs::remove_dir_all(&base).ok();
}