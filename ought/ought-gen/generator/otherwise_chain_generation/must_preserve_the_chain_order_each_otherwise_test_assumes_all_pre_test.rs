/// MUST preserve the chain order — each OTHERWISE test assumes all previous levels also failed
#[test]
fn test_generator__otherwise_chain_generation__must_preserve_the_chain_order_each_otherwise_test_assumes_all_pre() {
    use ought_gen::context::GenerationContext;
    use ought_gen::generator::{ClauseGroup, Language};
    use ought_gen::providers::build_batch_prompt;
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

    // Three-level chain: primary → cached → 504
    let ow1 = mk("gen::oc::otherwise_cached", Keyword::Otherwise, "return a cached response", vec![]);
    let ow2 = mk("gen::oc::otherwise_504", Keyword::Otherwise, "return 504 Gateway Timeout", vec![]);
    let parent = mk(
        "gen::oc::must_respond_fast",
        Keyword::Must,
        "respond within 200ms",
        vec![ow1.clone(), ow2.clone()],
    );

    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };

    let group = ClauseGroup {
        section_path: "Generator > OTHERWISE Chain Generation".to_string(),
        clauses: vec![&parent, &ow1, &ow2],
        conditions: vec![],
    };

    let prompt = build_batch_prompt(&group, &context);

    // Both OTHERWISE texts must appear, and they must appear in chain order:
    // the first fallback before the second fallback.
    let pos_ow1 = prompt
        .find("return a cached response")
        .expect("first OTHERWISE text must appear in the prompt");
    let pos_ow2 = prompt
        .find("return 504 Gateway Timeout")
        .expect("second OTHERWISE text must appear in the prompt");

    assert!(
        pos_ow1 < pos_ow2,
        "OTHERWISE clauses must appear in chain order in the prompt \
         (first fallback at offset {} must precede second fallback at offset {}). \
         The second OTHERWISE test must assume the first also failed.",
        pos_ow1,
        pos_ow2
    );

    // The second OTHERWISE's ID must appear after the first OTHERWISE's ID,
    // preserving the assumption that each level assumes all prior levels failed.
    let pos_id_ow1 = prompt
        .find("gen::oc::otherwise_cached")
        .expect("first OTHERWISE ID must appear in prompt");
    let pos_id_ow2 = prompt
        .find("gen::oc::otherwise_504")
        .expect("second OTHERWISE ID must appear in prompt");
    assert!(
        pos_id_ow1 < pos_id_ow2,
        "OTHERWISE clause IDs must appear in chain order: first ({}) before second ({})",
        pos_id_ow1,
        pos_id_ow2
    );
}