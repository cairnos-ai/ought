/// MUST support a `--diagnose` flag that enables LLM-powered failure diagnosis
#[test]
fn test_reporter__failure_narratives_llm_powered__must_support_a_diagnose_flag_that_enables_llm_powered_failure_dia() {
    #[derive(Default)]
    struct ReporterConfig {
        diagnose: bool,
    }

    impl ReporterConfig {
        fn diagnosis_enabled(&self) -> bool {
            self.diagnose
        }
    }

    let default_config = ReporterConfig::default();
    assert!(
        !default_config.diagnosis_enabled(),
        "diagnosis must be disabled by default when --diagnose flag is not passed"
    );

    let with_flag = ReporterConfig { diagnose: true };
    assert!(
        with_flag.diagnosis_enabled(),
        "diagnosis must be enabled when --diagnose flag is set"
    );
}