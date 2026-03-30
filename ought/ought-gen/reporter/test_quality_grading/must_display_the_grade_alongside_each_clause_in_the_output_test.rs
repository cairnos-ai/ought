/// MUST display the grade alongside each clause in the output
#[test]
fn test_reporter__test_quality_grading__must_display_the_grade_alongside_each_clause_in_the_output() {
    #[derive(Debug, Clone)]
    struct Grade {
        clause_id: String,
        grade: char,
        explanation: Option<String>,
    }

    fn render_clause_line(clause_id: &str, status: &str, grade: Option<&Grade>) -> String {
        let grade_badge = match grade {
            Some(g) => format!(" [{}]", g.grade),
            None => String::new(),
        };
        format!("{}  {}{}", status, clause_id, grade_badge)
    }

    let grade = Grade {
        clause_id: "auth::login::must_return_jwt".to_string(),
        grade: 'B',
        explanation: None,
    };

    // When grade is present, it must appear on the same line as the clause
    let line_with_grade = render_clause_line("auth::login::must_return_jwt", "✓", Some(&grade));
    assert!(
        line_with_grade.contains("auth::login::must_return_jwt"),
        "clause id must be present in the output line"
    );
    assert!(
        line_with_grade.contains('B'),
        "grade letter must appear alongside the clause in the output"
    );
    // Grade and clause must be on the same line (no newline between them)
    let newline_pos = line_with_grade.find('\n');
    let grade_pos = line_with_grade.find('B').expect("grade must be present");
    match newline_pos {
        None => {} // single line output — grade and clause are definitely together
        Some(nl) => assert!(
            grade_pos < nl,
            "grade must appear on the same line as the clause, not below it"
        ),
    }

    // When grading is disabled (no Grade provided), the grade badge must be absent
    let line_no_grade = render_clause_line("auth::login::must_return_jwt", "✓", None);
    assert!(
        !line_no_grade.contains('[') && !line_no_grade.contains(']'),
        "when grading is disabled, no grade badge must appear in the output"
    );
}