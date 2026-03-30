/// MUST be easy to integrate into CI pipelines (run without LLM access, gate on staleness separately)
#[test]
fn test_ought__integration__must_be_easy_to_integrate_into_ci_pipelines_run_without_llm_acces() {
    let bin = option_env!("CARGO_BIN_EXE_ought")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("ought"));

    // `ought run --help` must succeed with no LLM credentials present.
    // The runner only executes pre-generated tests and must never require network access.
    let run_help = std::process::Command::new(&bin)
        .args(["run", "--help"])
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("ANTHROPIC_AUTH_TOKEN")
        .env_remove("OPENAI_API_KEY")
        .output();

    match run_help {
        Ok(out) => {
            let run_stderr = String::from_utf8_lossy(&out.stderr);
            assert_ne!(
                out.status.code(),
                Some(2),
                "`ought run` must be a recognised subcommand for CI use; stderr: {run_stderr}"
            );
            assert!(
                !run_stderr.contains("unrecognized subcommand"),
                "`ought run` must be a recognised subcommand; stderr: {run_stderr}"
            );
        }
        Err(e) => panic!("`ought` binary must be available in CI environments; failed: {}", e),
    }

    // `ought check` must be a distinct subcommand so CI pipelines can gate on staleness
    // separately from running tests (no LLM required for staleness detection).
    let check_help = std::process::Command::new(&bin)
        .args(["check", "--help"])
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("ANTHROPIC_AUTH_TOKEN")
        .env_remove("OPENAI_API_KEY")
        .output();

    if let Ok(out) = check_help {
        let check_stderr = String::from_utf8_lossy(&out.stderr);
        assert_ne!(
            out.status.code(),
            Some(2),
            "`ought check` must be a recognised subcommand so CI can gate on staleness separately; stderr: {check_stderr}"
        );
        assert!(
            !check_stderr.contains("unrecognized subcommand"),
            "`ought check` must exist as a distinct subcommand for CI staleness gating; stderr: {check_stderr}"
        );
    }
}