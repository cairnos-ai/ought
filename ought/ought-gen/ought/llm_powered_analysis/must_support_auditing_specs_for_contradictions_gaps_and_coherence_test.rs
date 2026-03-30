/// MUST support auditing specs for contradictions, gaps, and coherence issues (`ought audit`)
#[test]
fn test_ought__llm_powered_analysis__must_support_auditing_specs_for_contradictions_gaps_and_coherence() {
    use std::fs;

    struct StubAuditGenerator {
        json: &'static str,
    }
    impl Generator for StubAuditGenerator {
        fn generate(&self, _: &Clause, _: &GenerationContext) -> anyhow::Result<GeneratedTest> {
            Ok(GeneratedTest {
                clause_id: ClauseId("audit::stub".to_string()),
                code: self.json.to_string(),
                language: Language::Rust,
                file_path: PathBuf::from("_audit.json"),
            })
        }
    }

    // Two contradictory specs: one says MUST cache, another says MUST NOT cache.
    let findings_json = r#"[
      {"kind":"Contradiction","description":"auth::login::must_cache contradicts auth::login::must_not_cache","clauses":["auth::login::must_cache_tokens","auth::login::must_not_cache_tokens"],"suggestion":"Clarify caching policy or add a GIVEN block to scope each clause","confidence":0.95},
      {"kind":"Gap","description":"No spec covers token expiry behavior","clauses":["auth::login::must_cache_tokens"],"suggestion":"Add a clause for token expiry","confidence":null}
    ]"#;

    let base = std::env::temp_dir()
        .join(format!("ought_audit_capability_{}", std::process::id()));
    let spec_dir = base.join("specs");
    fs::create_dir_all(&spec_dir).unwrap();

    fs::write(
        spec_dir.join("auth.ought.md"),
        "# Auth\n\n## Login\n\n- **MUST** cache tokens\n- **MUST NOT** cache tokens\n",
    ).unwrap();

    let specs = SpecGraph::from_roots(&[spec_dir.clone()]).expect("spec graph should parse");

    let res = audit(&specs, &StubAuditGenerator { json: findings_json });
    assert!(
        res.is_ok(),
        "ought audit must be supported and return Ok; err: {:?}",
        res.err()
    );

    let result = res.unwrap();
    // AuditResult must carry a structured list of findings.
    assert!(
        !result.findings.is_empty(),
        "audit must return findings for a spec set with contradictions and gaps; got empty list"
    );
    // Each finding must have a kind and a description.
    for finding in &result.findings {
        assert!(
            !finding.description.is_empty(),
            "audit finding must have a non-empty description; got empty for kind {:?}",
            finding.kind
        );
        assert!(
            !finding.clauses.is_empty(),
            "audit finding must reference at least one clause id; got none for kind {:?}",
            finding.kind
        );
    }

    let _ = fs::remove_dir_all(&base);
}