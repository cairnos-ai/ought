/// MUST provide a CLI (`ought`) as the primary interface for all operations
#[test]
fn test_ought__what_ought_does__must_provide_a_cli_ought_as_the_primary_interface_for_all_operati() {
    use std::process::Command;

    let output = Command::new("ought").arg("--help").output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let combined = format!("{}{}", stdout, stderr);
            assert!(
                !combined.is_empty(),
                "The `ought` CLI must produce output when invoked with --help"
            );
        }
        Err(e) => {
            panic!(
                "The `ought` CLI binary must be available on PATH; invocation failed: {}",
                e
            );
        }
    }
}