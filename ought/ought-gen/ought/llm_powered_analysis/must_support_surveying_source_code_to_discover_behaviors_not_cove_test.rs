/// MUST support surveying source code to discover behaviors not covered by any spec (`ought survey`)
#[test]
fn test_ought__llm_powered_analysis__must_support_surveying_source_code_to_discover_behaviors_not_cove() {
    use std::fs;

    struct StubSurveyGenerator;
    impl Generator for StubSurveyGenerator {
        fn generate(&self, _: &Clause, _: &GenerationContext) -> anyhow::Result<GeneratedTest> {
            Ok(GeneratedTest {
                clause_id: ClauseId("survey::stub".to_string()),
                code: r#"[{"file":"src/payments.rs","line":12,"description":"process_payment validates card number format","suggested_clause":"MUST validate card number format before charging","suggested_keyword":"Must","suggested_spec":"payments.ought.md"}]"#.to_string(),
                language: Language::Rust,
                file_path: PathBuf::from("_survey.json"),
            })
        }
    }

    let base = std::env::temp_dir()
        .join(format!("ought_survey_capability_{}", std::process::id()));
    let src_dir = base.join("src");
    let spec_dir = base.join("specs");
    fs::create_dir_all(&src_dir).unwrap();
    fs::create_dir_all(&spec_dir).unwrap();

    // Source code with public behaviors not yet covered by any spec.
    fs::write(
        src_dir.join("payments.rs"),
        "pub fn process_payment(card: &str, amount: u64) -> Result<(), String> {\n    if card.len() != 16 { return Err(\"invalid card\".into()); }\n    Ok(())\n}\n",
    ).unwrap();

    // Spec exists but does NOT cover the payment validation behavior.
    fs::write(
        spec_dir.join("auth.ought.md"),
        "# Auth\n\n## Login\n\n- **MUST** return 401 for invalid credentials\n",
    ).unwrap();

    let specs = SpecGraph::from_roots(&[spec_dir.clone()]).expect("spec graph should parse");

    let res = survey(&specs, &[src_dir.clone()], &StubSurveyGenerator);
    assert!(
        res.is_ok(),
        "ought survey must be supported and return Ok; err: {:?}",
        res.err()
    );

    let result = res.unwrap();
    // The SurveyResult type must exist and carry a list of uncovered behaviors.
    let _ = result.uncovered;

    let _ = fs::remove_dir_all(&base);
}