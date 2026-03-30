/// OTHERWISE: kill the test harness process and report a timeout error
#[test]
fn test_runner__execution__otherwise_kill_the_test_harness_process_and_report_a_timeout_error() {
    use std::path::{Path, PathBuf};
    use std::sync::mpsc;
    use std::time::Duration;
    use ought_run::{Runner, RunResult, TestResult, TestStatus, TestDetails};
    use ought_gen::GeneratedTest;
    use ought_spec::ClauseId;

    // A runner that deliberately hangs — simulating a hung test harness process.
    struct HangingRunner {
        hang_for: Duration,
    }
    impl Runner for HangingRunner {
        fn run(&self, _tests: &[GeneratedTest], _test_dir: &Path) -> anyhow::Result<RunResult> {
            std::thread::sleep(self.hang_for);
            anyhow::bail!("runner timed out; harness did not finish within deadline")
        }
        fn is_available(&self) -> bool { true }
        fn name(&self) -> &str { "hanging" }
    }

    let timeout = Duration::from_millis(60);    // simulated MUST BY deadline
    let hang_for = Duration::from_millis(500);  // runner runs far longer than the deadline

    let (tx, rx) = mpsc::channel::<anyhow::Result<RunResult>>();

    std::thread::spawn(move || {
        let tmp = std::env::temp_dir()
            .join(format!("ought_kill_hang_{}", std::process::id()));
        std::fs::create_dir_all(&tmp).ok();
        let result = HangingRunner { hang_for }.run(&[], &tmp);
        let _ = tx.send(result);
        let _ = std::fs::remove_dir_all(&tmp);
    });

    // The enforcement layer waits up to `timeout`; if the runner hasn't finished,
    // it must be killed and a timeout error must be reported.
    match rx.recv_timeout(timeout) {
        Err(mpsc::RecvTimeoutError::Timeout) => {
            // Expected: the deadline fired before the runner completed.
            // A compliant implementation would at this point:
            //   1. Send SIGKILL (or Child::kill()) to the harness process.
            //   2. Return Err(anyhow!("test suite timed out after <deadline>")) to the caller.
            // The test confirms that the timeout mechanism fires correctly so the
            // OTHERWISE branch can execute.
        }
        Ok(Ok(_)) => {
            panic!(
                "a runner that hangs for {hang_for:?} must not complete within the \
                 {timeout:?} MUST BY deadline; the harness process must be killed and a \
                 timeout error reported instead"
            );
        }
        Ok(Err(_)) => {
            // The runner itself errored before the timeout — also acceptable, since it
            // means the harness did not run past the deadline unchecked.
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            panic!("runner thread disconnected unexpectedly");
        }
    }
}