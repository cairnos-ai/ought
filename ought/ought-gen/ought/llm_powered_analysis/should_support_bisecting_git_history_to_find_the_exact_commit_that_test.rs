/// SHOULD support bisecting git history to find the exact commit that broke a clause (`ought bisect`)
#[test]
fn test_ought__llm_powered_analysis__should_support_bisecting_git_history_to_find_the_exact_commit_that() {
    use std::fs;

    struct SentinelRunner {
        sentinel: PathBuf,
        clause_id: ClauseId,
    }
    impl Runner for SentinelRunner {
        fn run(&self, _: &[GeneratedTest], _: &std::path::Path) -> anyhow::Result<RunResult> {
            let content = fs::read_to_string(&self.sentinel).unwrap_or_default();
            let status = if content.trim() == "pass" {
                TestStatus::Passed
            } else {
                TestStatus::Failed
            };
            Ok(RunResult {
                results: vec![TestResult {
                    clause_id: self.clause_id.clone(),
                    status,
                    message: None,
                    duration: Duration::ZERO,
                    details: TestDetails {
                        failure_message: None,
                        stack_trace: None,
                        iterations: None,
                        measured_duration: None,
                    },
                }],
                total_duration: Duration::ZERO,
            })
        }
        fn is_available(&self) -> bool { true }
        fn name(&self) -> &str { "sentinel" }
    }

    let base = std::env::temp_dir()
        .join(format!("ought_bisect_capability_{}", std::process::id()));
    let src_dir = base.join("src");
    let spec_dir = base.join("specs");
    fs::create_dir_all(&src_dir).unwrap();
    fs::create_dir_all(&spec_dir).unwrap();

    fs::write(
        spec_dir.join("billing.ought.md"),
        "# Billing\n\n## Invoices\n\n- **MUST** generate invoices with correct totals\n",
    ).unwrap();

    for args in &[
        vec!["init"],
        vec!["config", "user.email", "alice@example.com"],
        vec!["config", "user.name", "Alice"],
    ] {
        std::process::Command::new("git")
            .args(args)
            .current_dir(&base)
            .output()
            .unwrap();
    }

    let sentinel = src_dir.join("status.txt");

    // Commit 1: clause passes.
    fs::write(&sentinel, "pass\n").unwrap();
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&base)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "Initial: billing totals correct"])
        .current_dir(&base)
        .output()
        .unwrap();

    // Commit 2: clause breaks.
    fs::write(&sentinel, "fail\n").unwrap();
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&base)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "bug: rounding error in invoice totals"])
        .current_dir(&base)
        .output()
        .unwrap();

    let clause_id =
        ClauseId("billing::invoices::must_generate_invoices_with_correct_totals".to_string());
    let runner = SentinelRunner {
        sentinel: sentinel.clone(),
        clause_id: clause_id.clone(),
    };
    let specs = SpecGraph::from_roots(&[spec_dir.clone()]).expect("spec graph should parse");
    let options = BisectOptions { range: None, regenerate: false };

    let res = bisect(&clause_id, &specs, &runner, &options);
    assert!(
        res.is_ok(),
        "ought bisect must be supported and return Ok; err: {:?}",
        res.err()
    );

    let result = res.unwrap();
    // bisect must identify a specific breaking commit.
    assert!(
        !result.breaking_commit.hash.is_empty(),
        "bisect must identify the breaking commit hash; got empty string"
    );
    assert!(
        !result.breaking_commit.message.is_empty(),
        "bisect must populate the breaking commit message; got empty string"
    );
    // The breaking commit should be the second commit (the bug).
    assert!(
        result.breaking_commit.message.contains("rounding")
            || result.breaking_commit.message.contains("bug")
            || result.breaking_commit.message.contains("invoice"),
        "bisect must identify the actual breaking commit; got message: {:?}",
        result.breaking_commit.message
    );
    assert_eq!(
        result.clause_id, clause_id,
        "bisect result must carry the clause_id that was passed in"
    );

    let _ = fs::remove_dir_all(&base);
}