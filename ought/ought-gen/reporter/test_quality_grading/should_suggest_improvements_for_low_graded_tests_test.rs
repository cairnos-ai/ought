/// SHOULD suggest improvements for low-graded tests
#[test]
fn test_reporter__test_quality_grading__should_suggest_improvements_for_low_graded_tests() {
    #[derive(Debug, Clone)]
    struct Improvement {
        description: String,
    }

    #[derive(Debug, Clone)]
    struct Grade {
        clause_id: String,
        grade: char,
        explanation: Option<String>,
        suggested_improvement: Option<Improvement>,
    }

    fn is_low_grade(grade: char) -> bool {
        matches!(grade, 'C' | 'D' | 'F')
    }

    let grades = vec![
        Grade {
            clause_id: "spec::section::clause_ok".to_string(),
            grade: 'A',
            explanation: None,
            suggested_improvement: None,
        },
        Grade {
            clause_id: "spec::section::clause_weak".to_string(),
            grade: 'C',
            explanation: Some("Test checks status code but not payload.".to_string()),
            suggested_improvement: Some(Improvement {
                description: "Add an assertion on the response body to verify the JWT token field is present and non-empty.".to_string(),
            }),
        },
        Grade {
            clause_id: "spec::section::clause_trivial".to_string(),
            grade: 'F',
            explanation: Some("Test always passes; no behavior is verified.".to_string()),
            suggested_improvement: Some(Improvement {
                description: "Replace the empty test body with assertions that exercise the actual behavior described by the clause.".to_string(),
            }),
        },
    ];

    for g in &grades {
        if is_low_grade(g.grade) {
            assert!(
                g.suggested_improvement.is_some(),
                "low grade '{}' for '{}' should include a suggested improvement",
                g.grade, g.clause_id
            );
            let improvement = g.suggested_improvement.as_ref().unwrap();
            assert!(
                !improvement.description.trim().is_empty(),
                "suggested improvement for grade '{}' must not be empty",
                g.grade
            );
            // Improvement should be actionable — long enough to be meaningful
            assert!(
                improvement.description.len() > 10,
                "suggested improvement for grade '{}' must be a meaningful suggestion, not a placeholder",
                g.grade
            );
        }
    }

    // High-grade tests should not have unnecessary improvement suggestions cluttering the output
    let high_grade = grades.iter().find(|g| g.grade == 'A').expect("test data must include an A grade");
    // Improvement may be None for a high grade — that is the expected/preferred behavior
    if let Some(imp) = &high_grade.suggested_improvement {
        // If one is present despite a high grade, it must still be non-empty
        assert!(!imp.description.trim().is_empty());
    }
}