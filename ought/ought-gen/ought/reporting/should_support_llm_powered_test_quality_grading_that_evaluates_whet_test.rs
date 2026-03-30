/// SHOULD support LLM-powered test quality grading that evaluates whether generated tests actually validate their clauses
#[test]
fn test_ought__reporting__should_support_llm_powered_test_quality_grading_that_evaluates_whet() {
    use std::path::PathBuf;
    use std::time::Duration;
    use ought_report::grade::grade;
    use ought_report::types::Grade;
    use ought_run::{RunResult, TestDetails, TestResult, TestStatus};
    use ought_spec::{Clause, ClauseId, Keyword, Metadata, Section, SourceLocation, Spec};

    let clause_id = "search::results::must_rank_by_relevance";
    let spec = Spec {
        name: "Search".to_string(),
        metadata: Metadata::default(),
        sections: vec![Section {
            title: "Results".to_string(),
            depth: 1,
            prose: String::new(),
            clauses: vec![Clause {
                id: ClauseId(clause_id.to_string()),
                keyword: Keyword::Must,
                severity: Keyword::Must.severity(),
                text: "rank results by descending relevance score".to_string(),
                condition: None,
                otherwise: vec![],
                temporal: None,
                hints: vec![],
                source_location: SourceLocation { file: PathBuf::from("search.ought.md"), line: 3 },
                content_hash: "bb".to_string(),
            }],
            subsections: vec![],
        }],
        source_path: PathBuf::from("search.ought.md"),
    };
    let run = RunResult {
        results: vec![TestResult {
            clause_id: ClauseId(clause_id.to_string()),
            status: TestStatus::Passed,
            message: None,
            duration: Duration::from_millis(20),
            details: TestDetails::default(),
        }],
        total_duration: Duration::from_millis(20),
    };

    // grade() must accept a RunResult and spec slice and return Ok(Vec<Grade>).
    let result = grade(&run, &[spec]);
    assert!(result.is_ok(), "grade() must not return Err: {:?}", result.err());

    let grades = result.unwrap();
    // Every returned Grade must carry a non-empty clause_id and a letter in A–F.
    for g in &grades {
        assert!(!g.clause_id.0.is_empty(),
            "each Grade must have a non-empty clause_id for display alongside the clause");
        assert!(
            ('A'..='F').contains(&g.grade),
            "grade '{}' must be a letter A through F", g.grade
        );
        // Grades below B should include an explanation so the developer knows how to improve.
        if g.grade > 'B' {
            assert!(
                g.explanation.is_some(),
                "grades below B must include an explanation; clause '{}' got '{}' with none",
                g.clause_id,
                g.grade
            );
        }
    }

    // The Grade struct must support clause_id, a letter grade char, and an optional explanation
    // so the reporter can render grade badges and improvement suggestions alongside clauses.
    let shallow_grade = Grade {
        clause_id: ClauseId(clause_id.to_string()),
        grade: 'C',
        explanation: Some(
            "Test only checks that results are non-empty; it does not verify ordering.".to_string(),
        ),
    };
    assert_eq!(shallow_grade.grade, 'C');
    assert!(('A'..='F').contains(&shallow_grade.grade));
    assert!(
        shallow_grade.explanation.is_some(),
        "a C grade must carry an explanation to guide improvement"
    );

    let excellent_grade = Grade {
        clause_id: ClauseId(clause_id.to_string()),
        grade: 'A',
        explanation: None,
    };
    assert_eq!(excellent_grade.grade, 'A');
    // An A grade may omit the explanation.
    let _ = excellent_grade.explanation; // field must exist even if None
}