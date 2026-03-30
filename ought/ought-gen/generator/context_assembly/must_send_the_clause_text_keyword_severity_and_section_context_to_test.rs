/// MUST send the clause text, keyword, severity, and section context to the LLM
#[test]
fn test_generator__context_assembly__must_send_the_clause_text_keyword_severity_and_section_context_to() {
    use std::path::PathBuf;
    use ought_spec::types::{Clause, ClauseId, Keyword, Severity, SourceLocation};
    use ought_gen::context::GenerationContext;
    use ought_gen::generator::{ClauseGroup, Language};
    use ought_gen::providers::{build_prompt, build_batch_prompt};

    let clause = Clause {
        id: ClauseId("auth::login::must_return_jwt".to_string()),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: "return a JWT token on successful login".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("test.ought.md"), line: 1 },
        content_hash: "abc123".to_string(),
    };

    let ctx = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };

    // Single-clause prompt must include text, keyword, severity
    let prompt = build_prompt(&clause, &ctx);
    assert!(
        prompt.contains("return a JWT token on successful login"),
        "prompt must include clause text; got:\n{prompt}"
    );
    assert!(
        prompt.contains("MUST"),
        "prompt must include keyword string; got:\n{prompt}"
    );
    assert!(
        prompt.contains("Required"),
        "prompt must include severity; got:\n{prompt}"
    );

    // Batch prompt must forward section path as context
    let group = ClauseGroup {
        section_path: "Auth > Login".to_string(),
        clauses: vec![&clause],
        conditions: vec![],
    };
    let batch_prompt = build_batch_prompt(&group, &ctx);
    assert!(
        batch_prompt.contains("Auth > Login"),
        "batch prompt must include section path; got:\n{batch_prompt}"
    );
    // Clause text and keyword must also appear in batch prompt
    assert!(
        batch_prompt.contains("return a JWT token on successful login"),
        "batch prompt must include clause text; got:\n{batch_prompt}"
    );
    assert!(
        batch_prompt.contains("MUST"),
        "batch prompt must include keyword; got:\n{batch_prompt}"
    );
}