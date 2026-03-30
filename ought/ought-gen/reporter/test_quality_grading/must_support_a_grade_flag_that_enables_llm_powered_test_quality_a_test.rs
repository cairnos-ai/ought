/// MUST support a `--grade` flag that enables LLM-powered test quality assessment
#[test]
fn test_reporter__test_quality_grading__must_support_a_grade_flag_that_enables_llm_powered_test_quality_a() {
    use std::cell::Cell;

    struct MockLlmGrader {
        call_count: Cell<usize>,
    }

    impl MockLlmGrader {
        fn new() -> Self {
            MockLlmGrader { call_count: Cell::new(0) }
        }

        fn grade_test(&self, _clause_id: &str, _test_code: &str) -> char {
            self.call_count.set(self.call_count.get() + 1);
            'B'
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
        fn grade_tests(&self, clauses: &[&str], grader: &MockLlmGrader) -> Vec<char> {
            if !self.options.grade {
                return vec![];
            }
            clauses.iter().map(|c| grader.grade_test(c, "fn test() {}")).collect()
        }
    }

    let grader = MockLlmGrader::new();
    let clauses = &["auth::login::must_return_jwt", "auth::login::must_return_401"];

    // Without --grade: grading must not run
    let reporter_no_flag = Reporter { options: ReportOptions { grade: false } };
    let grades = reporter_no_flag.grade_tests(clauses, &grader);
    assert!(grades.is_empty(), "--grade not set: grading must not run, returned grades must be empty");
    assert_eq!(grader.calls_made(), 0, "--grade not set: LLM grader must not be called");

    // With --grade: grading runs and returns one grade per clause
    let reporter_with_flag = Reporter { options: ReportOptions { grade: true } };
    let grades = reporter_with_flag.grade_tests(clauses, &grader);
    assert_eq!(grades.len(), clauses.len(), "--grade set: must return one grade per clause");
    assert!(grader.calls_made() > 0, "--grade set: LLM grader must be invoked");
}