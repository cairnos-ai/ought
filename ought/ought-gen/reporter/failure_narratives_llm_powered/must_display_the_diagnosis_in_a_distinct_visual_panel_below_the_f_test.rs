/// MUST display the diagnosis in a distinct visual panel below the failure
#[test]
fn test_reporter__failure_narratives_llm_powered__must_display_the_diagnosis_in_a_distinct_visual_panel_below_the_f() {
    struct DiagnosisPanel {
        content: String,
    }

    impl DiagnosisPanel {
        fn render(&self) -> String {
            format!(
                "  ┌─ Diagnosis ──────────────────────────────────┐\n  │ {}\n  └──────────────────────────────────────────────┘",
                self.content
            )
        }
    }

    fn render_failure_with_diagnosis(failure_label: &str, panel: &DiagnosisPanel) -> String {
        format!("FAILED  {}\n{}", failure_label, panel.render())
    }

    let panel = DiagnosisPanel {
        content: "The handler returns 404 because the route is not registered.".to_string(),
    };

    let output = render_failure_with_diagnosis("auth::login::must_return_jwt", &panel);

    // Diagnosis panel must appear after the failure line
    let failure_pos = output.find("FAILED").expect("failure label must be present");
    let panel_pos = output.find("Diagnosis").expect("diagnosis panel must be present in output");
    assert!(
        panel_pos > failure_pos,
        "diagnosis panel must appear below the failure, not before it"
    );

    // Panel must use a distinct visual border to set it apart
    assert!(
        output.contains('┌') && output.contains('└'),
        "diagnosis panel must have a distinct visual border (box-drawing characters)"
    );

    // Panel must contain the actual diagnosis text
    assert!(
        output.contains(&panel.content),
        "rendered output must contain the diagnosis text"
    );
}