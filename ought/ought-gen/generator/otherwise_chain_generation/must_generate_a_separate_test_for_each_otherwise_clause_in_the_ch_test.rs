/// MUST generate a separate test for each OTHERWISE clause in the chain
#[test]
fn test_generator__otherwise_chain_generation__must_generate_a_separate_test_for_each_otherwise_clause_in_the_ch() {
    use ought_gen::generator::{ClauseGroup, Language};
    use ought_gen::providers::parse_batch_response;
    use ought_spec::{Clause, ClauseId, Keyword, Severity, SourceLocation};
    use std::path::PathBuf;

    fn mk(id: &str, kw: Keyword, text: &str, otherwise: Vec<Clause>) -> Clause {
        Clause {
            id: ClauseId(id.to_string()),
            keyword: kw,
            severity: Severity::Required,
            text: text.to_string(),
            condition: None,
            otherwise,
            temporal: None,
            hints: vec![],
            source_location: SourceLocation { file: PathBuf::from("spec.ought.md"), line: 1 },
            content_hash: "h".to_string(),
        }
    }

    let ow1 = mk("gen::oc::otherwise_cached", Keyword::Otherwise, "return a cached response", vec![]);
    let ow2 = mk("gen::oc::otherwise_504", Keyword::Otherwise, "return 504 Gateway Timeout", vec![]);
    let parent = mk(
        "gen::oc::must_respond_fast",
        Keyword::Must,
        "respond within 200ms",
        vec![ow1.clone(), ow2.clone()],
    );

    // The group must include the parent AND each OTHERWISE clause as separate entries
    // so the LLM knows to emit a separate marker — and hence a separate test — for each.
    let group = ClauseGroup {
        section_path: "Generator > OTHERWISE Chain Generation".to_string(),
        clauses: vec![&parent, &ow1, &ow2],
        conditions: vec![],
    };

    // Simulate the LLM correctly emitting a separate marker per clause
    let response = [
        "// === CLAUSE: gen::oc::must_respond_fast ===",
        "#[test]",
        "fn test_must_respond_fast() { assert!(true); }",
        "",
        "// === CLAUSE: gen::oc::otherwise_cached ===",
        "#[test]",
        "fn test_otherwise_cached() { assert!(true); }",
        "",
        "// === CLAUSE: gen::oc::otherwise_504 ===",
        "#[test]",
        "fn test_otherwise_504() { assert!(true); }",
    ]
    .join("\n");

    let tests = parse_batch_response(&response, &group, Language::Rust);

    assert_eq!(
        tests.len(),
        3,
        "must produce 3 separate GeneratedTests: 1 for the parent obligation + 1 per OTHERWISE clause, got {}",
        tests.len()
    );
    assert_eq!(
        tests[0].clause_id,
        ClauseId("gen::oc::must_respond_fast".to_string()),
        "first test must belong to the primary obligation"
    );
    assert_eq!(
        tests[1].clause_id,
        ClauseId("gen::oc::otherwise_cached".to_string()),
        "second test must belong to the first OTHERWISE clause"
    );
    assert_eq!(
        tests[2].clause_id,
        ClauseId("gen::oc::otherwise_504".to_string()),
        "third test must belong to the second OTHERWISE clause"
    );
}