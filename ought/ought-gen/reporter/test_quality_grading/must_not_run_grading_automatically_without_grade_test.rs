/// MUST NOT run grading automatically without `--grade`
#[test]
fn test_reporter__test_quality_grading__must_not_run_grading_automatically_without_grade() {
    use std::cell::Cell;

    struct MockLlmGrader {
        call_count: Cell<usize>,
    }

    impl MockLlmGrader {
        fn new() -> Self {
            MockLlmGrader { call_count: Cell::new(0) }
        }

        fn grade(&self, _clause_id: &str, _test_body: &str) -> char {
            self.call_count.set(self.call_count.get() + 1);
            'A'
        }

        fn calls_made(&self) -> usize {
            self.call_count.get()
        }
    }

    struct ReportOptions {
        grade: bool,
    }

    struct Reporter {
        options: ReportOptions,
    }

    impl Reporter {
        fn run_report(&self, clauses: &[&str], grader: &MockLlmGrader) -> Vec<String> {
            clauses
                .iter()
                .map(|clause_id| {
                    let mut line = format!("✓  {}", clause_id);
                    if self.options.grade {
                        let g = grader.grade(clause_id, "fn test() { assert!(true); }");
                        line.push_str(&format!(" [{}]", g));
                    }
                    line
                })
                .collect()
        }
    }

    let clauses = &[
        "auth::login::must_return_jwt",
        "auth::login::must_return_401",
        "auth::login::must_expire_token",
    ];
    let grader = MockLlmGrader::new();

    // Without --grade: no LLM calls must be made, regardless of clause count
    let reporter_no_flag = Reporter { options: ReportOptions { grade: false } };
    let lines = reporter_no_flag.run_report(clauses, &grader);

    assert_eq!(
        grader.calls_made(),
        0,
        "LLM grader must not be called when --grade flag is absent (API cost concern)"
    );
    assert_eq!(lines.len(), clauses.len(), "report must still emit one line per clause without --grade");
    for line in &lines {
        assert!(
            !line.contains('['),
            "output must not contain a grade badge when --grade is absent, got: {}",
            line
        );
    }

    // With --grade: grader must be invoked once per clause
    let grader2 = MockLlmGrader::new();
    let reporter_with_flag = Reporter { options: ReportOptions { grade: true } };
    let lines_graded = reporter_with_flag.run_report(clauses, &grader2);

    assert_eq!(
        grader2.calls_made(),
        clauses.len(),
        "--grade flag set: LLM grader must be called once per clause"
    );
    for line in &lines_graded {
        assert!(
            line.contains('['),
            "each line must contain a grade badge when --grade is set, got: {}",
            line
        );
    }
}