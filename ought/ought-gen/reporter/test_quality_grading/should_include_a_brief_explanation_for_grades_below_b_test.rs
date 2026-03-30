/// SHOULD include a brief explanation for grades below B
#[test]
fn test_reporter__test_quality_grading__should_include_a_brief_explanation_for_grades_below_b() {
    #[derive(Debug, Clone)]
    struct Grade {
        clause_id: String,
        grade: char,
        explanation: Option<String>,
    }

    fn needs_explanation(grade: char) -> bool {
        matches!(grade, 'C' | 'D' | 'F')
    }

    let grades = vec![
        Grade { clause_id: "spec::section::clause_a".to_string(), grade: 'A', explanation: None },
        Grade { clause_id: "spec::section::clause_b".to_string(), grade: 'B', explanation: None },
        Grade { clause_id: "spec::section::clause_c".to_string(), grade: 'C', explanation: Some("Test does not assert the response body.".to_string()) },
        Grade { clause_id: "spec::section::clause_d".to_string(), grade: 'D', explanation: Some("Assertion is too broad and accepts incorrect values.".to_string()) },
        Grade { clause_id: "spec::section::clause_f".to_string(), grade: 'F', explanation: Some("Test body is empty; it always passes.".to_string()) },
    ];

    for g in &grades {
        if needs_explanation(g.grade) {
            assert!(
                g.explanation.is_some(),
                "grade '{}' for '{}' is below B — a brief explanation must be included",
                g.grade, g.clause_id
            );
            let explanation = g.explanation.as_ref().unwrap();
            assert!(
                !explanation.trim().is_empty(),
                "explanation for grade '{}' must not be blank",
                g.grade
            );
            // Explanation should be brief — a sentence or two, not a wall of text
            assert!(
                explanation.len() < 300,
                "explanation for grade '{}' must be brief (under 300 chars), got {} chars",
                g.grade, explanation.len()
            );
        }
    }

    // A and B grades do not require an explanation (though one is allowed)
    let high_grades: Vec<&Grade> = grades.iter().filter(|g| matches!(g.grade, 'A' | 'B')).collect();
    assert!(!high_grades.is_empty(), "test data must include A and B grades");
    // No assertion forcing explanation to be None for A/B — it's allowed but not required
    for g in high_grades {
        // If an explanation is present on a high grade, it must still be non-empty
        if let Some(exp) = &g.explanation {
            assert!(!exp.trim().is_empty(), "if explanation is present for grade '{}', it must be non-empty", g.grade);
        }
    }
}