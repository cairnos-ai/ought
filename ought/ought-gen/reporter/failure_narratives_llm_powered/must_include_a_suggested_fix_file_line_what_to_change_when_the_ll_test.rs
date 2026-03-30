/// MUST include a suggested fix (file, line, what to change) when the LLM can determine one
#[test]
fn test_reporter__failure_narratives_llm_powered__must_include_a_suggested_fix_file_line_what_to_change_when_the_ll() {
    struct SuggestedFix {
        file: String,
        line: usize,
        description: String,
    }

    impl SuggestedFix {
        fn render(&self) -> String {
            format!("  Suggested fix → {}:{}: {}", self.file, self.line, self.description)
        }
    }

    struct DiagnosisResult {
        narrative: String,
        suggested_fix: Option<SuggestedFix>,
    }

    // When LLM can determine a fix, all three fields must be present
    let result_with_fix = DiagnosisResult {
        narrative: "Route is not registered, so the handler is never reached.".to_string(),
        suggested_fix: Some(SuggestedFix {
            file: "src/routes.rs".to_string(),
            line: 42,
            description: "Register the missing route: router.get(\"/api/users\", handle_users)".to_string(),
        }),
    };

    let fix = result_with_fix.suggested_fix.as_ref().expect("fix must be present when LLM determines one");
    assert!(!fix.file.is_empty(), "suggested fix must specify a file path");
    assert!(fix.line > 0, "suggested fix must specify a positive line number");
    assert!(!fix.description.is_empty(), "suggested fix must describe what to change");

    let rendered = fix.render();
    assert!(rendered.contains("src/routes.rs"), "rendered fix must include the file path");
    assert!(rendered.contains("42"), "rendered fix must include the line number");
    assert!(rendered.contains("Register the missing route"), "rendered fix must include the change description");

    // When LLM cannot determine a fix, the field must be absent (not a placeholder)
    let result_no_fix = DiagnosisResult {
        narrative: "The root cause could not be determined from available context.".to_string(),
        suggested_fix: None,
    };
    assert!(
        result_no_fix.suggested_fix.is_none(),
        "suggested_fix must be None when LLM cannot determine a fix — must not emit an empty/placeholder fix"
    );
}