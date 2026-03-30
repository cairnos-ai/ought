/// MUST include any code-block hints attached to the clause
#[test]
fn test_generator__context_assembly__must_include_any_code_block_hints_attached_to_the_clause() {
    use std::path::PathBuf;
    use ought_spec::types::{Clause, ClauseId, Keyword, Severity, SourceLocation};
    use ought_gen::context::GenerationContext;
    use ought_gen::generator::Language;
    use ought_gen::providers::build_prompt;

    let hint_code = "let token = jwt::encode(&claims, &secret, Algorithm::HS256).unwrap();";

    let clause = Clause {
        id: ClauseId("auth::must_encode_jwt".to_string()),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: "encode the JWT using HS256 algorithm".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![hint_code.to_string()],
        source_location: SourceLocation { file: PathBuf::from("auth.ought.md"), line: 5 },
        content_hash: "hint111".to_string(),
    };

    let ctx = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };

    let prompt = build_prompt(&clause, &ctx);

    assert!(
        prompt.contains("## Hints"),
        "prompt must contain a Hints section when clause has code-block hints; got:\n{prompt}"
    );
    assert!(
        prompt.contains(hint_code),
        "prompt must embed the hint code verbatim; got:\n{prompt}"
    );
}