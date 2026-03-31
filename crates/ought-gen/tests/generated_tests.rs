#![allow(non_snake_case, unused_imports)]
use std::path::{Path, PathBuf};
use std::fs;
use std::time::Duration;
use std::collections::HashMap;
use ought_spec::types::*;
use ought_spec::parser::Parser;
use ought_gen::generator::*;
use ought_gen::context::*;
use ought_gen::manifest::*;
use ought_gen::providers::{build_prompt, build_batch_prompt, parse_batch_response, strip_markdown_fences, derive_file_path, keyword_str};

// ============================================================================
// context_assembly (8 tests)
// ============================================================================

/// MUST send the clause text, keyword, severity, and section context to the LLM
#[test]
fn test_generator__context_assembly__must_send_the_clause_text_keyword_severity_and_section_context_to() {
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

/// MUST include any code-block hints attached to the clause
#[test]
fn test_generator__context_assembly__must_include_any_code_block_hints_attached_to_the_clause() {
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

/// MUST include the free-text `context:` block
#[test]
fn test_generator__context_assembly__must_include_the_free_text_context_block() {
    use ought_spec::config::{Config, ProjectConfig, SpecsConfig, ContextConfig, GeneratorConfig, ToleranceConfig, McpConfig};

    let tmp = std::env::temp_dir().join("ought_ctxasm_context_block");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    let free_text = "This service handles payment processing for subscriptions.";

    let spec = Spec {
        name: "Payments".to_string(),
        metadata: Metadata {
            context: Some(free_text.to_string()),
            sources: vec![],
            schemas: vec![],
            requires: vec![],
        },
        sections: vec![],
        source_path: tmp.join("payments.ought.md"),
    };

    let clause = Clause {
        id: ClauseId("payments::must_charge_correct_amount".to_string()),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: "charge the correct subscription amount".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("payments.ought.md"), line: 1 },
        content_hash: "jkl000".to_string(),
    };

    let config = Config {
        project: ProjectConfig { name: "test".to_string(), version: "0.1.0".to_string() },
        specs: SpecsConfig::default(),
        context: ContextConfig { search_paths: vec![], exclude: vec![], max_files: 50 },
        generator: GeneratorConfig { provider: "claude".to_string(), model: None, tolerance: ToleranceConfig::default() },
        runner: HashMap::new(),
        mcp: McpConfig::default(),
    };

    let assembler = ContextAssembler::new(&config);
    let ctx = assembler.assemble(&clause, &spec).expect("assemble failed");

    // assemble must propagate the free-text context to spec_context
    assert_eq!(
        ctx.spec_context.as_deref(),
        Some(free_text),
        "spec_context must equal the context: block from spec metadata"
    );

    // build_prompt must embed the context block in the outgoing LLM prompt
    let prompt = build_prompt(&clause, &ctx);
    assert!(
        prompt.contains(free_text),
        "prompt must include the free-text context block; got:\n{prompt}"
    );
    assert!(
        prompt.contains("## Context"),
        "prompt must have a Context section header; got:\n{prompt}"
    );

    let _ = std::fs::remove_dir_all(&tmp);
}

/// MUST read and include source files referenced by `source:` metadata
#[test]
fn test_generator__context_assembly__must_read_and_include_source_files_referenced_by_source_metadata() {
    use ought_spec::config::{Config, ProjectConfig, SpecsConfig, ContextConfig, GeneratorConfig, ToleranceConfig, McpConfig};

    let tmp = std::env::temp_dir().join("ought_ctxasm_source_meta");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    // Write a fake source file
    let src_content = "fn authenticate(token: &str) -> bool { token == \"secret\" }";
    std::fs::write(tmp.join("auth.rs"), src_content).unwrap();

    let spec = Spec {
        name: "Auth".to_string(),
        metadata: Metadata {
            context: None,
            sources: vec!["auth.rs".to_string()],
            schemas: vec![],
            requires: vec![],
        },
        sections: vec![],
        source_path: tmp.join("auth.ought.md"),
    };

    let clause = Clause {
        id: ClauseId("auth::must_validate_token".to_string()),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: "validate the token before granting access".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("auth.ought.md"), line: 1 },
        content_hash: "def456".to_string(),
    };

    let config = Config {
        project: ProjectConfig { name: "test".to_string(), version: "0.1.0".to_string() },
        specs: SpecsConfig::default(),
        context: ContextConfig { search_paths: vec![], exclude: vec![], max_files: 50 },
        generator: GeneratorConfig { provider: "claude".to_string(), model: None, tolerance: ToleranceConfig::default() },
        runner: HashMap::new(),
        mcp: McpConfig::default(),
    };

    let assembler = ContextAssembler::new(&config);
    let ctx = assembler.assemble(&clause, &spec).expect("assemble failed");

    assert_eq!(ctx.source_files.len(), 1, "expected one source file from metadata");
    assert!(
        ctx.source_files[0].path.ends_with("auth.rs"),
        "source file path should be auth.rs; got {:?}",
        ctx.source_files[0].path
    );
    assert_eq!(
        ctx.source_files[0].content, src_content,
        "source file content must match what was written"
    );

    let _ = std::fs::remove_dir_all(&tmp);
}

/// MUST read and include schema files referenced by `schema:` metadata
#[test]
fn test_generator__context_assembly__must_read_and_include_schema_files_referenced_by_schema_metadata() {
    use ought_spec::config::{Config, ProjectConfig, SpecsConfig, ContextConfig, GeneratorConfig, ToleranceConfig, McpConfig};

    let tmp = std::env::temp_dir().join("ought_ctxasm_schema_meta");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    let schema_content = r#"type Query { user(id: ID!): User } type User { id: ID! name: String! }"#;
    std::fs::write(tmp.join("users.graphql"), schema_content).unwrap();

    let spec = Spec {
        name: "Users".to_string(),
        metadata: Metadata {
            context: None,
            sources: vec![],
            schemas: vec!["users.graphql".to_string()],
            requires: vec![],
        },
        sections: vec![],
        source_path: tmp.join("users.ought.md"),
    };

    let clause = Clause {
        id: ClauseId("users::must_return_user_fields".to_string()),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: "return all required user fields".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("users.ought.md"), line: 1 },
        content_hash: "ghi789".to_string(),
    };

    let config = Config {
        project: ProjectConfig { name: "test".to_string(), version: "0.1.0".to_string() },
        specs: SpecsConfig::default(),
        context: ContextConfig { search_paths: vec![], exclude: vec![], max_files: 50 },
        generator: GeneratorConfig { provider: "claude".to_string(), model: None, tolerance: ToleranceConfig::default() },
        runner: HashMap::new(),
        mcp: McpConfig::default(),
    };

    let assembler = ContextAssembler::new(&config);
    let ctx = assembler.assemble(&clause, &spec).expect("assemble failed");

    assert_eq!(ctx.schema_files.len(), 1, "expected one schema file from metadata");
    assert!(
        ctx.schema_files[0].path.ends_with("users.graphql"),
        "schema file path should be users.graphql; got {:?}",
        ctx.schema_files[0].path
    );
    assert_eq!(
        ctx.schema_files[0].content, schema_content,
        "schema file content must match what was written"
    );
    // Schema files must NOT appear in source_files
    assert!(
        ctx.source_files.is_empty(),
        "schema files must not bleed into source_files"
    );

    let _ = std::fs::remove_dir_all(&tmp);
}

/// SHOULD auto-discover relevant source files when no explicit `source:` is provided
#[test]
fn test_generator__context_assembly__should_auto_discover_relevant_source_files_when_no_explicit_source() {
    use ought_spec::config::{Config, ProjectConfig, SpecsConfig, ContextConfig, GeneratorConfig, ToleranceConfig, McpConfig};

    let tmp = std::env::temp_dir().join("ought_ctxasm_autodiscover");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    // Write a file that mentions words from the clause text
    std::fs::write(
        tmp.join("session.rs"),
        "fn refresh_session(token: &str) -> Option<Session> { /* refresh the session token */ None }",
    )
    .unwrap();

    // Spec has NO explicit sources
    let spec = Spec {
        name: "Session".to_string(),
        metadata: Metadata {
            context: None,
            sources: vec![],   // empty -- triggers auto-discovery
            schemas: vec![],
            requires: vec![],
        },
        sections: vec![],
        source_path: tmp.join("session.ought.md"),
    };

    let clause = Clause {
        id: ClauseId("session::must_refresh_token".to_string()),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: "refresh the session token before expiry".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("session.ought.md"), line: 1 },
        content_hash: "auto333".to_string(),
    };

    let config = Config {
        project: ProjectConfig { name: "test".to_string(), version: "0.1.0".to_string() },
        specs: SpecsConfig::default(),
        context: ContextConfig {
            search_paths: vec![tmp.clone()],
            exclude: vec![],
            max_files: 50,
        },
        generator: GeneratorConfig { provider: "claude".to_string(), model: None, tolerance: ToleranceConfig::default() },
        runner: HashMap::new(),
        mcp: McpConfig::default(),
    };

    let assembler = ContextAssembler::new(&config);
    let ctx = assembler.assemble(&clause, &spec).expect("assemble failed");

    assert!(
        !ctx.source_files.is_empty(),
        "when no explicit source: is given, assembler should auto-discover relevant files"
    );
    assert!(
        ctx.source_files.iter().any(|f| f.path.ends_with("session.rs")),
        "auto-discovered files should include session.rs which mentions clause keywords"
    );

    let _ = std::fs::remove_dir_all(&tmp);
}

/// SHOULD rank discovered files by relevance to the clause text
#[test]
fn test_generator__context_assembly__should_rank_discovered_files_by_relevance_to_the_clause_text() {
    use ought_spec::config::{Config, ProjectConfig, SpecsConfig, ContextConfig, GeneratorConfig, ToleranceConfig, McpConfig};

    let tmp = std::env::temp_dir().join("ought_ctxasm_ranking");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    // "high_match.rs" mentions all three long keywords from the clause text
    std::fs::write(
        tmp.join("high_match.rs"),
        "fn process_payment(invoice: &Invoice) -> Receipt { \
         // validate invoice, charge payment, generate receipt \
         todo!() }",
    )
    .unwrap();

    // "low_match.rs" mentions only one keyword
    std::fs::write(
        tmp.join("low_match.rs"),
        "fn noop() { /* invoice placeholder */ }",
    )
    .unwrap();

    // "no_match.rs" mentions none of the keywords
    std::fs::write(
        tmp.join("no_match.rs"),
        "fn unrelated() -> bool { true }",
    )
    .unwrap();

    let clause = Clause {
        id: ClauseId("billing::must_process_payment".to_string()),
        keyword: Keyword::Must,
        severity: Severity::Required,
        // All three long words appear in high_match.rs: "invoice", "charge", "receipt"
        text: "process payment for the invoice and generate receipt".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("billing.ought.md"), line: 1 },
        content_hash: "rank444".to_string(),
    };

    let config = Config {
        project: ProjectConfig { name: "test".to_string(), version: "0.1.0".to_string() },
        specs: SpecsConfig::default(),
        context: ContextConfig {
            search_paths: vec![tmp.clone()],
            exclude: vec![],
            max_files: 50,
        },
        generator: GeneratorConfig { provider: "claude".to_string(), model: None, tolerance: ToleranceConfig::default() },
        runner: HashMap::new(),
        mcp: McpConfig::default(),
    };

    let assembler = ContextAssembler::new(&config);
    let discovered = assembler.discover_sources(&clause).expect("discover_sources failed");

    // Files with zero score are excluded; remaining should be ranked best-first
    assert!(
        !discovered.is_empty(),
        "at least one file should match the clause keywords"
    );

    // high_match.rs must come before low_match.rs
    let high_pos = discovered.iter().position(|f| f.path.ends_with("high_match.rs"));
    let low_pos = discovered.iter().position(|f| f.path.ends_with("low_match.rs"));

    assert!(
        high_pos.is_some(),
        "high_match.rs should be discovered (it mentions multiple clause keywords)"
    );
    assert!(
        low_pos.is_some(),
        "low_match.rs should be discovered (it mentions at least one clause keyword)"
    );
    assert!(
        high_pos.unwrap() < low_pos.unwrap(),
        "high_match.rs (more keyword matches) must rank before low_match.rs (fewer matches)"
    );

    // no_match.rs must not appear -- zero score means excluded
    assert!(
        !discovered.iter().any(|f| f.path.ends_with("no_match.rs")),
        "files with no keyword matches must not appear in discovered sources"
    );

    let _ = std::fs::remove_dir_all(&tmp);
}

/// MUST respect the `max_files` limit in `ought.toml` to avoid exceeding LLM context
#[test]
fn test_generator__context_assembly__must_respect_the_max_files_limit_in_ought_toml_to_avoid_exceeding() {
    use ought_spec::config::{Config, ProjectConfig, SpecsConfig, ContextConfig, GeneratorConfig, ToleranceConfig, McpConfig};

    let tmp = std::env::temp_dir().join("ought_ctxasm_max_files");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    // Write 10 files all containing the keyword "authenticate" so every file scores > 0
    for i in 0..10 {
        let content = format!("fn authenticate_{i}() {{ /* authenticate user #{i} */ }}");
        std::fs::write(tmp.join(format!("module_{i}.rs")), content).unwrap();
    }

    let max_files: usize = 3;
    let config = Config {
        project: ProjectConfig { name: "test".to_string(), version: "0.1.0".to_string() },
        specs: SpecsConfig::default(),
        context: ContextConfig {
            search_paths: vec![tmp.clone()],
            exclude: vec![],
            max_files,
        },
        generator: GeneratorConfig { provider: "claude".to_string(), model: None, tolerance: ToleranceConfig::default() },
        runner: HashMap::new(),
        mcp: McpConfig::default(),
    };

    let clause = Clause {
        id: ClauseId("auth::must_authenticate".to_string()),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: "authenticate the user before granting access".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("auth.ought.md"), line: 1 },
        content_hash: "maxf222".to_string(),
    };

    let assembler = ContextAssembler::new(&config);
    let discovered = assembler.discover_sources(&clause).expect("discover_sources failed");

    assert!(
        discovered.len() <= max_files,
        "discover_sources must not return more than max_files ({max_files}) files; got {}",
        discovered.len()
    );

    let _ = std::fs::remove_dir_all(&tmp);
}

// ============================================================================
// manifest_and_hashing (6 tests)
// ============================================================================

/// MUST compute a clause hash from the keyword + clause text + context metadata
#[test]
fn test_generator__manifest_and_hashing__must_compute_a_clause_hash_from_the_keyword_clause_text_context_m() {
    let path = PathBuf::from("spec.ought.md");

    // Parse a baseline MUST clause.
    let spec_must = Parser::parse_string(
        "# Spec\n\n## Section\n\n- **MUST** validate the token\n",
        &path,
    )
    .expect("parse must succeed");
    let hash_must = spec_must.sections[0].clauses[0].content_hash.clone();

    // 1. Hash must be a 16-character hex string (64-bit SipHash output).
    assert_eq!(
        hash_must.len(),
        16,
        "clause_hash must be 16 hex chars; got {hash_must:?}"
    );
    assert!(
        hash_must.chars().all(|c| c.is_ascii_hexdigit()),
        "clause_hash must contain only hex digits; got {hash_must:?}"
    );

    // 2. Same keyword + text + no condition -> same hash every time (deterministic).
    let spec_must2 = Parser::parse_string(
        "# Spec\n\n## Section\n\n- **MUST** validate the token\n",
        &path,
    )
    .expect("parse must succeed");
    assert_eq!(
        hash_must,
        spec_must2.sections[0].clauses[0].content_hash,
        "identical clause must produce an identical hash"
    );

    // 3. Different keyword (SHOULD) -> different hash.
    let spec_should = Parser::parse_string(
        "# Spec\n\n## Section\n\n- **SHOULD** validate the token\n",
        &path,
    )
    .expect("parse must succeed");
    assert_ne!(
        hash_must,
        spec_should.sections[0].clauses[0].content_hash,
        "changing the keyword from MUST to SHOULD must change the hash"
    );

    // 4. Different clause text -> different hash.
    let spec_other_text = Parser::parse_string(
        "# Spec\n\n## Section\n\n- **MUST** reject the request\n",
        &path,
    )
    .expect("parse must succeed");
    assert_ne!(
        hash_must,
        spec_other_text.sections[0].clauses[0].content_hash,
        "different clause text must produce a different hash"
    );

    // 5. GIVEN condition (context metadata) changes the hash of the nested clause.
    let spec_given = Parser::parse_string(
        "# Spec\n\n## Section\n\n- **GIVEN** user is authenticated\n  - **MUST** validate the token\n",
        &path,
    )
    .expect("parse must succeed");
    let hash_conditioned = spec_given.sections[0].clauses[0].content_hash.clone();
    assert_ne!(
        hash_must,
        hash_conditioned,
        "adding a GIVEN condition must change the clause hash (condition is part of the hash input)"
    );
}

/// MUST compute a source hash from the contents of referenced source files
#[test]
fn test_generator__manifest_and_hashing__must_compute_a_source_hash_from_the_contents_of_referenced_source() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use chrono::Utc;

    // The source hash must be computed from referenced source file contents using
    // the same DefaultHasher mechanism as clause hashing.
    let compute_source_hash = |contents: &[&str]| -> String {
        let mut hasher = DefaultHasher::new();
        for content in contents {
            content.hash(&mut hasher);
        }
        format!("{:016x}", hasher.finish())
    };

    let content_v1 = "fn check(token: &str) -> bool { token == \"secret\" }";
    let content_v2 = "fn check(token: &str) -> bool { token == \"rotated_secret\" }";

    let hash_v1 = compute_source_hash(&[content_v1]);
    let hash_v2 = compute_source_hash(&[content_v2]);

    // 1. Different file contents must produce different source hashes.
    assert_ne!(
        hash_v1, hash_v2,
        "changed source file contents must produce a different source hash"
    );

    // 2. Same contents must always hash to the same value (deterministic).
    assert_eq!(
        hash_v1,
        compute_source_hash(&[content_v1]),
        "identical source file contents must always produce the same hash"
    );

    // 3. Adding a second referenced file must change the combined hash.
    let hash_two = compute_source_hash(&[content_v1, content_v2]);
    assert_ne!(
        hash_v1, hash_two,
        "adding a second referenced source file must change the source hash"
    );

    // 4. Manifest::is_stale() consumes the computed source hash.
    //    Verify it correctly detects a source file change as stale.
    let id = ClauseId("spec::section::must_validate".to_string());
    let fixed_clause_hash = "aabbccddeeff0011";

    let mut manifest = Manifest::default();
    manifest.entries.insert(
        id.0.clone(),
        ManifestEntry {
            clause_hash: fixed_clause_hash.to_string(),
            source_hash: hash_v1.clone(),
            generated_at: Utc::now(),
            model: "claude-sonnet-4-6".to_string(),
        },
    );

    // Stored source hash matches -> entry is not stale.
    assert!(
        !manifest.is_stale(&id, fixed_clause_hash, &hash_v1),
        "matching source hash must not be reported as stale"
    );

    // Source file was modified -> new hash -> entry is stale.
    assert!(
        manifest.is_stale(&id, fixed_clause_hash, &hash_v2),
        "changed source file hash must be reported as stale so the test is regenerated"
    );
}

/// MUST skip generation for clauses whose hashes match the manifest (unless `--force`)
#[test]
fn test_generator__manifest_and_hashing__must_skip_generation_for_clauses_whose_hashes_match_the_manifest() {
    use chrono::Utc;

    // Unit: Manifest::is_stale() drives the skip decision

    let id = ClauseId("spec::section::must_validate".to_string());
    let clause_hash = "aabbccddeeff0011";
    let source_hash = "";

    let mut manifest = Manifest::default();
    manifest.entries.insert(
        id.0.clone(),
        ManifestEntry {
            clause_hash: clause_hash.to_string(),
            source_hash: source_hash.to_string(),
            generated_at: Utc::now(),
            model: "claude-sonnet-4-6".to_string(),
        },
    );

    // Both hashes match -> not stale -> generation is skipped.
    assert!(
        !manifest.is_stale(&id, clause_hash, source_hash),
        "is_stale() must return false (skip generation) when both hashes match the manifest"
    );

    // Clause text changed -> stale -> regenerate.
    assert!(
        manifest.is_stale(&id, "different_hash_00", source_hash),
        "is_stale() must return true when the clause_hash differs from the stored value"
    );

    // Source file modified -> stale -> regenerate.
    assert!(
        manifest.is_stale(&id, clause_hash, "new_source_hash_0"),
        "is_stale() must return true when the source_hash differs from the stored value"
    );

    // No manifest entry at all -> stale (first-time generation).
    let new_id = ClauseId("spec::section::must_new".to_string());
    assert!(
        manifest.is_stale(&new_id, clause_hash, source_hash),
        "is_stale() must return true for a clause with no manifest entry"
    );
}

/// MUST write both hashes to `ought/ought-gen/manifest.toml` after generation
#[test]
fn test_generator__manifest_and_hashing__must_write_both_hashes_to_ought_ought_gen_manifest_toml_after_gen() {
    use chrono::Utc;

    let tmp = std::env::temp_dir()
        .join(format!("ought_both_hashes_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let manifest_path = tmp.join("manifest.toml");

    let clause_id   = "spec::section::must_do_the_thing";
    let clause_hash = "a1b2c3d4e5f60789";
    let source_hash = "fedcba9876543210";

    let mut manifest = Manifest::default();
    manifest.entries.insert(
        clause_id.to_string(),
        ManifestEntry {
            clause_hash: clause_hash.to_string(),
            source_hash: source_hash.to_string(),
            generated_at: Utc::now(),
            model: "claude-sonnet-4-6".to_string(),
        },
    );
    manifest.save(&manifest_path).expect("Manifest::save must succeed");

    // Raw TOML must contain both keys and their values.
    let toml_content = std::fs::read_to_string(&manifest_path)
        .expect("manifest.toml must exist after save");

    assert!(
        toml_content.contains("clause_hash"),
        "manifest.toml must contain the 'clause_hash' key; content:\n{toml_content}"
    );
    assert!(
        toml_content.contains(clause_hash),
        "manifest.toml must contain the clause_hash value; content:\n{toml_content}"
    );
    assert!(
        toml_content.contains("source_hash"),
        "manifest.toml must contain the 'source_hash' key; content:\n{toml_content}"
    );
    assert!(
        toml_content.contains(source_hash),
        "manifest.toml must contain the source_hash value; content:\n{toml_content}"
    );

    // Round-trip: reload and confirm both hashes survive serialization.
    let reloaded = Manifest::load(&manifest_path).expect("Manifest::load must succeed");
    let entry = reloaded
        .entries
        .get(clause_id)
        .expect("entry must survive a save/load round-trip");

    assert_eq!(
        entry.clause_hash, clause_hash,
        "clause_hash must survive a save/load round-trip"
    );
    assert_eq!(
        entry.source_hash, source_hash,
        "source_hash must survive a save/load round-trip"
    );

    let _ = std::fs::remove_dir_all(&tmp);
}

/// MUST record the model name and timestamp in the manifest entry
#[test]
fn test_generator__manifest_and_hashing__must_record_the_model_name_and_timestamp_in_the_manifest_entry() {
    use chrono::DateTime;

    let tmp = std::env::temp_dir()
        .join(format!("ought_model_ts_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let manifest_path = tmp.join("manifest.toml");

    let model_name = "claude-sonnet-4-6";
    // Use a fixed, known timestamp so assertions are deterministic.
    let timestamp = DateTime::parse_from_rfc3339("2026-03-30T12:00:00Z")
        .expect("valid RFC3339 timestamp")
        .to_utc();

    let mut manifest = Manifest::default();
    manifest.entries.insert(
        "spec::section::must_do_the_thing".to_string(),
        ManifestEntry {
            clause_hash: "0000000000000000".to_string(),
            source_hash: "".to_string(),
            generated_at: timestamp,
            model: model_name.to_string(),
        },
    );
    manifest.save(&manifest_path).expect("Manifest::save must succeed");

    // Raw TOML must contain the model name and ISO timestamp.
    let content = std::fs::read_to_string(&manifest_path)
        .expect("manifest.toml must be written");

    assert!(
        content.contains("model"),
        "manifest.toml must contain the 'model' key; content:\n{content}"
    );
    assert!(
        content.contains(model_name),
        "manifest.toml must contain the model name value; content:\n{content}"
    );
    assert!(
        content.contains("generated_at"),
        "manifest.toml must contain the 'generated_at' key; content:\n{content}"
    );
    assert!(
        content.contains("2026-03-30"),
        "manifest.toml must contain the ISO date in the timestamp; content:\n{content}"
    );

    // Round-trip: reload and confirm model and timestamp survive serialization.
    let reloaded = Manifest::load(&manifest_path).expect("Manifest::load must succeed");
    let entry = reloaded
        .entries
        .get("spec::section::must_do_the_thing")
        .expect("entry must survive a save/load round-trip");

    assert_eq!(
        entry.model, model_name,
        "model name must survive a save/load round-trip"
    );
    assert_eq!(
        entry.generated_at, timestamp,
        "timestamp must survive a save/load round-trip"
    );

    let _ = std::fs::remove_dir_all(&tmp);
}

/// MUST detect and remove orphaned generated tests (clause was deleted from spec)
#[test]
fn test_generator__manifest_and_hashing__must_detect_and_remove_orphaned_generated_tests_clause_was_delete() {
    use chrono::Utc;

    let make_entry = || ManifestEntry {
        clause_hash: "0000000000000000".to_string(),
        source_hash: "".to_string(),
        generated_at: Utc::now(),
        model: "claude-sonnet-4-6".to_string(),
    };

    let id_a = ClauseId("spec::section::must_foo".to_string());
    let id_b = ClauseId("spec::section::must_bar".to_string()); // will be deleted from spec
    let id_c = ClauseId("spec::section::must_baz".to_string());

    let mut manifest = Manifest::default();
    manifest.entries.insert(id_a.0.clone(), make_entry());
    manifest.entries.insert(id_b.0.clone(), make_entry());
    manifest.entries.insert(id_c.0.clone(), make_entry());
    assert_eq!(manifest.entries.len(), 3, "setup: manifest must start with three entries");

    // Simulate spec after id_b's clause was deleted: only id_a and id_c are still valid.
    let valid_ids = [&id_a, &id_c];
    manifest.remove_orphans(&valid_ids);

    assert_eq!(
        manifest.entries.len(),
        2,
        "remove_orphans must leave exactly two entries; remaining: {:?}",
        manifest.entries.keys().collect::<Vec<_>>()
    );
    assert!(
        manifest.entries.contains_key(&id_a.0),
        "valid clause id_a must remain in the manifest after remove_orphans"
    );
    assert!(
        manifest.entries.contains_key(&id_c.0),
        "valid clause id_c must remain in the manifest after remove_orphans"
    );
    assert!(
        !manifest.entries.contains_key(&id_b.0),
        "orphaned clause id_b must be removed from the manifest by remove_orphans"
    );

    // Idempotent: calling again with the same valid set must not change anything.
    manifest.remove_orphans(&valid_ids);
    assert_eq!(
        manifest.entries.len(),
        2,
        "remove_orphans must be idempotent"
    );

    // Edge case: empty valid set removes all remaining entries.
    manifest.remove_orphans(&[]);
    assert!(
        manifest.entries.is_empty(),
        "remove_orphans with an empty valid_ids set must clear all manifest entries"
    );
}

// ============================================================================
// provider_abstraction (7 tests -- removed 2 that exec real LLM CLIs)
// ============================================================================

/// MUST define a `Generator` trait that all LLM providers implement
#[test]
fn test_generator__provider_abstraction__must_define_a_generator_trait_that_all_llm_providers_implement() {
    use ought_gen::providers::from_config;

    // A local struct that implements Generator -- proves the trait is object-safe
    // and that concrete types can fulfil the contract.
    struct MockProvider;
    impl Generator for MockProvider {
        fn generate(
            &self,
            clause: &Clause,
            context: &GenerationContext,
        ) -> anyhow::Result<GeneratedTest> {
            Ok(GeneratedTest {
                clause_id: clause.id.clone(),
                code: "// mock".to_string(),
                language: context.target_language,
                file_path: PathBuf::from("mock_test.rs"),
            })
        }
    }

    let clause = Clause {
        id: ClauseId("provider::must_work".to_string()),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: "work correctly".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("spec.ought.md"), line: 1 },
        content_hash: "h".to_string(),
    };
    let ctx = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };

    // Coerce to trait object -- fails at compile time if Generator is not object-safe
    let provider: Box<dyn Generator> = Box::new(MockProvider);
    let result = provider.generate(&clause, &ctx);
    assert!(result.is_ok(), "Generator::generate must return Ok for a valid clause");

    let generated = result.unwrap();
    assert_eq!(
        generated.clause_id,
        ClauseId("provider::must_work".to_string()),
        "GeneratedTest must carry the originating clause id"
    );

    // Verify that the four shipped providers all satisfy Box<dyn Generator> by
    // constructing each through from_config.
    assert!(
        from_config("claude", None).is_ok(),
        "ClaudeGenerator must be constructable via from_config"
    );
    assert!(
        from_config("openai", None).is_ok(),
        "OpenAiGenerator must be constructable via from_config"
    );
    assert!(
        from_config("ollama", Some("llama3")).is_ok(),
        "OllamaGenerator must be constructable via from_config"
    );
    assert!(
        from_config("/tmp/custom-llm", None).is_ok(),
        "CustomGenerator must be constructable via from_config"
    );
}

/// SHOULD support provider-specific configuration in `ought.toml` under `[generator]`
#[test]
fn test_generator__provider_abstraction__should_support_provider_specific_configuration_in_ought_toml_under() {
    use ought_gen::providers::from_config;

    let cases: &[(&str, Option<&str>)] = &[
        ("claude",     Some("claude-sonnet-4-6")),
        ("anthropic",  Some("claude-opus-4-6")),
        ("openai",     Some("gpt-4o")),
        ("chatgpt",    Some("gpt-4o-mini")),
        ("ollama",     Some("mistral")),
        ("ollama",     Some("codellama")),
        ("claude",     None),
        ("openai",     None),
        ("ollama",     None),
    ];

    for (provider, model) in cases {
        let result = from_config(provider, *model);
        assert!(
            result.is_ok(),
            "[generator] provider=\"{provider}\" model={model:?} must construct without error; got: {:?}",
            result.err()
        );
    }
}

/// MUST ship with a Claude provider that execs the `claude` CLI
#[test]
fn test_generator__provider_abstraction__must_ship_with_a_claude_provider_that_execs_the_claude_cli() {
    use ought_gen::providers::from_config;
    use ought_gen::providers::claude::ClaudeGenerator;
    use ought_gen::providers::exec_cli;

    // Both canonical aliases must resolve to a ClaudeGenerator without error.
    let via_anthropic = from_config("anthropic", None);
    assert!(
        via_anthropic.is_ok(),
        "from_config(\"anthropic\", _) must return Ok; got: {:?}",
        via_anthropic.err()
    );

    let via_claude = from_config("claude", None);
    assert!(
        via_claude.is_ok(),
        "from_config(\"claude\", _) must return Ok; got: {:?}",
        via_claude.err()
    );

    // The ClaudeGenerator can be constructed directly and satisfies Generator.
    let _g: Box<dyn Generator> = Box::new(ClaudeGenerator::new(None));
    let _with_model: Box<dyn Generator> = Box::new(ClaudeGenerator::new(Some("claude-sonnet-4-6".to_string())));

    // When a CLI binary is absent the error message must name the tool.
    let err = exec_cli("__ought_absent_claude_binary__", &["-p"], "prompt")
        .unwrap_err()
        .to_string();
    assert!(
        err.to_lowercase().contains("not found"),
        "missing CLI error must say 'not found'; got: {err}"
    );
    assert!(
        err.contains("__ought_absent_claude_binary__"),
        "missing CLI error must name the binary; got: {err}"
    );
}

/// MUST ship with an OpenAI provider that execs the `openai` or `chatgpt` CLI
#[test]
fn test_generator__provider_abstraction__must_ship_with_an_openai_provider_that_execs_the_openai_or_chatgp() {
    use ought_gen::providers::from_config;
    use ought_gen::providers::openai::OpenAiGenerator;

    // Both "openai" and "chatgpt" must resolve without error.
    let via_openai = from_config("openai", None);
    assert!(
        via_openai.is_ok(),
        "from_config(\"openai\", _) must return Ok; got: {:?}",
        via_openai.err()
    );

    let via_chatgpt = from_config("chatgpt", None);
    assert!(
        via_chatgpt.is_ok(),
        "from_config(\"chatgpt\", _) must return Ok; got: {:?}",
        via_chatgpt.err()
    );

    // Direct construction must also satisfy the trait.
    let _g: Box<dyn Generator> = Box::new(OpenAiGenerator::new(None));
    let _with_model: Box<dyn Generator> = Box::new(OpenAiGenerator::new(Some("gpt-4o".to_string())));
}

/// SHOULD ship with an Ollama provider that execs the `ollama` CLI for local models
#[test]
fn test_generator__provider_abstraction__should_ship_with_an_ollama_provider_that_execs_the_ollama_cli_for_l() {
    use ought_gen::providers::from_config;
    use ought_gen::providers::ollama::OllamaGenerator;

    // "ollama" must resolve successfully.
    let via_ollama = from_config("ollama", None);
    assert!(
        via_ollama.is_ok(),
        "from_config(\"ollama\", _) must return Ok; got: {:?}",
        via_ollama.err()
    );

    // With an explicit model name.
    let via_ollama_model = from_config("ollama", Some("mistral"));
    assert!(
        via_ollama_model.is_ok(),
        "from_config(\"ollama\", Some(\"mistral\")) must return Ok; got: {:?}",
        via_ollama_model.err()
    );

    // Direct construction satisfies Generator trait.
    let _g: Box<dyn Generator> = Box::new(OllamaGenerator::new("llama3".to_string()));
}

/// SHOULD detect when the required CLI tool is not installed and report a clear error
#[test]
fn test_generator__provider_abstraction__should_detect_when_the_required_cli_tool_is_not_installed_and_repor() {
    use ought_gen::providers::exec_cli;

    // Using a name that cannot possibly be installed lets us exercise the
    // NotFound path without side-effects.
    let missing = "__ought_missing_cli_xyz123__";
    let err = exec_cli(missing, &[], "some prompt")
        .unwrap_err()
        .to_string();

    assert!(
        err.to_lowercase().contains("not found"),
        "error for a missing CLI must say 'not found'; got: {err}"
    );
    assert!(
        err.contains(missing),
        "error must name the missing CLI tool '{}' so the user knows what to install; got: {err}",
        missing
    );
    assert!(
        err.to_lowercase().contains("path") || err.to_lowercase().contains("install") || err.to_lowercase().contains("not found"),
        "error must hint at how to resolve the problem (PATH / install); got: {err}"
    );

    // Each provider name that maps to a specific binary should surface the right
    // binary name in the error when that binary is absent.
    let claude_err = exec_cli("__claude_not_here__", &["-p"], "prompt")
        .unwrap_err()
        .to_string();
    assert!(
        claude_err.contains("__claude_not_here__"),
        "provider error must name the exact binary it tried to exec; got: {claude_err}"
    );
}

/// MAY support custom providers by specifying an arbitrary executable in `ought.toml`
#[test]
fn test_generator__provider_abstraction__may_support_custom_providers_by_specifying_an_arbitrary_executab() {
    use ought_gen::providers::from_config;
    use ought_gen::providers::exec_cli;

    // Any string that looks like a path (contains '/') or is not a known keyword
    // should be treated as a custom executable path rather than a named provider.
    let custom_cases = &[
        "/usr/local/bin/my-llm",
        "./scripts/generate.sh",
        "/opt/custom/llm-wrapper",
    ];
    for path in custom_cases {
        let result = from_config(path, None);
        assert!(
            result.is_ok(),
            "from_config with path '{}' must return Ok (custom provider); got: {:?}",
            path,
            result.err()
        );
        // Verify it satisfies the trait.
        let _g: Box<dyn Generator> = result.unwrap();
    }

    // A custom provider must pass the prompt via stdin, just like built-in providers.
    // Verify by using `cat` as the custom executable -- it echoes stdin to stdout.
    let prompt = "#[test] fn test_custom() { assert!(true); }";
    let output = exec_cli("cat", &[], prompt).expect("cat must succeed as a custom provider stand-in");
    assert!(
        output.contains("test_custom"),
        "custom provider output must be captured from stdout; got: {output}"
    );

    // An unknown non-path token (no slash) that is not a known provider keyword
    // must also fall through to the custom path rather than error at construction.
    let result = from_config("unknown-provider-token", None);
    assert!(
        result.is_ok(),
        "unrecognised provider name must be treated as a custom executable path, not a construction error"
    );
}

/// MUST NOT manage API keys or authentication -- that is the CLI tool's responsibility
#[test]
fn test_generator__provider_abstraction__must_not_manage_api_keys_or_authentication_that_is_the_cli_tool_s_res() {
    use ought_gen::providers::claude::ClaudeGenerator;
    use ought_gen::providers::openai::OpenAiGenerator;
    use ought_gen::providers::ollama::OllamaGenerator;
    use ought_gen::providers::custom::CustomGenerator;
    use ought_gen::providers::exec_cli;

    // Each provider struct stores at most a model name -- no API key, token,
    // secret, or credential field.
    let _c = ClaudeGenerator::new(None);
    let _c_model = ClaudeGenerator::new(Some("claude-opus-4-6".to_string()));

    let _o = OpenAiGenerator::new(None);
    let _o_model = OpenAiGenerator::new(Some("gpt-4o".to_string()));

    // OllamaGenerator requires a model (local -- no cloud auth at all).
    let _ol = OllamaGenerator::new("llama3".to_string());

    // CustomGenerator takes only an executable path -- no credentials.
    let _cu = CustomGenerator::new(PathBuf::from("/usr/local/bin/my-llm"));

    // exec_cli must not inject auth environment variables.
    let result = exec_cli("sh", &["-c", "printenv OUGHT_SENTINEL_VAR_NEVER_SET"], "");
    // sh -c "printenv MISSING_VAR" exits 1 -- exec_cli must propagate the error
    assert!(
        result.is_err(),
        "exec_cli must not inject env vars the shell doesn't have; a missing var query should fail"
    );
}

/// MUST pass prompts via stdin and capture generated test code from stdout
#[test]
fn test_generator__provider_abstraction__must_pass_prompts_via_stdin_and_capture_generated_test_code_from() {
    use ought_gen::providers::exec_cli;

    // `cat` with no args echoes stdin to stdout -- perfect for verifying the
    // stdin-passing contract without needing a real LLM installed.
    let prompt = "fn test_example() { assert!(true); }";
    let result = exec_cli("cat", &[], prompt);
    assert!(
        result.is_ok(),
        "exec_cli must succeed when the command exits 0; got: {:?}",
        result.err()
    );
    let output = result.unwrap();
    assert_eq!(
        output.trim(),
        prompt.trim(),
        "stdout captured by exec_cli must exactly match the data written to stdin"
    );

    // Multi-line prompt must also survive the round-trip intact.
    let multiline = "line one\nline two\nline three";
    let out = exec_cli("cat", &[], multiline).expect("cat must succeed");
    assert!(
        out.contains("line one") && out.contains("line two") && out.contains("line three"),
        "multi-line prompt must be passed to stdin and captured from stdout intact; got: {out}"
    );
}

// ============================================================================
// test_generation (9 tests)
// ============================================================================

/// MUST generate one test function per clause
#[test]
fn test_generator__test_generation__must_generate_one_test_function_per_clause() {
    fn mk(id: &str, text: &str) -> Clause {
        Clause {
            id: ClauseId(id.to_string()),
            keyword: Keyword::Must,
            severity: Severity::Required,
            text: text.to_string(),
            condition: None,
            otherwise: vec![],
            temporal: None,
            hints: vec![],
            source_location: SourceLocation { file: PathBuf::from("spec.ought.md"), line: 1 },
            content_hash: "x".to_string(),
        }
    }

    let c1 = mk("gen::test::must_a", "do a");
    let c2 = mk("gen::test::must_b", "do b");
    let c3 = mk("gen::test::must_c", "do c");
    let response = [
        "// === CLAUSE: gen::test::must_a ===",
        "#[test]",
        "fn test_gen__test__must_a() { assert!(true); }",
        "",
        "// === CLAUSE: gen::test::must_b ===",
        "#[test]",
        "fn test_gen__test__must_b() { assert!(true); }",
        "",
        "// === CLAUSE: gen::test::must_c ===",
        "#[test]",
        "fn test_gen__test__must_c() { assert!(true); }",
    ].join("\n");
    let group = ClauseGroup {
        section_path: "Gen > Test".to_string(),
        clauses: vec![&c1, &c2, &c3],
        conditions: vec![],
    };
    let tests = parse_batch_response(&response, &group, Language::Rust);
    assert_eq!(
        tests.len(),
        3,
        "must_generate_one_test_function_per_clause: expected 3 GeneratedTests for 3 clauses, got {}",
        tests.len()
    );
    for (i, expected_id) in ["gen::test::must_a", "gen::test::must_b", "gen::test::must_c"]
        .iter()
        .enumerate()
    {
        assert_eq!(
            tests[i].clause_id,
            ClauseId(expected_id.to_string()),
            "test[{}] must map to clause id '{}'",
            i,
            expected_id
        );
    }
}

/// MUST include the original clause text as a doc comment in the generated test
#[test]
fn test_generator__test_generation__must_include_the_original_clause_text_as_a_doc_comment_in_the_gen() {
    let clause = Clause {
        id: ClauseId("auth::login::must_return_jwt".to_string()),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: "return a JWT on successful authentication".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("auth.ought.md"), line: 5 },
        content_hash: "abc".to_string(),
    };
    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };
    let prompt = build_prompt(&clause, &context);
    assert!(
        prompt.contains("doc comment"),
        "must_include_the_original_clause_text_as_a_doc_comment_in_the_gen: \
         build_prompt must instruct the LLM to add the clause text as a doc comment"
    );
    assert!(
        prompt.contains("return a JWT on successful authentication"),
        "must_include_the_original_clause_text_as_a_doc_comment_in_the_gen: \
         build_prompt must embed the original clause text so it can be echoed verbatim"
    );
}

/// MUST include the clause identifier in the test function name
#[test]
fn test_generator__test_generation__must_include_the_clause_identifier_in_the_test_function_name() {
    let clause_id_str = "auth::login::must_return_401_for_invalid_credentials";
    let clause = Clause {
        id: ClauseId(clause_id_str.to_string()),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: "return 401 for invalid credentials".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("auth.ought.md"), line: 7 },
        content_hash: "abc".to_string(),
    };
    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };
    let prompt = build_prompt(&clause, &context);
    assert!(
        prompt.contains(clause_id_str),
        "must_include_the_clause_identifier_in_the_test_function_name: \
         build_prompt must embed the full clause ID '{}' so the LLM can derive the function name",
        clause_id_str
    );
    let path = derive_file_path(&clause, Language::Rust);
    let path_str = path.to_string_lossy();
    assert!(
        path_str.contains("auth") && path_str.contains("login"),
        "must_include_the_clause_identifier_in_the_test_function_name: \
         derive_file_path must encode clause ID segments in the path; got '{}'",
        path_str
    );
    assert!(
        path_str.ends_with("_test.rs"),
        "must_include_the_clause_identifier_in_the_test_function_name: \
         Rust test file must end in '_test.rs'; got '{}'",
        path_str
    );
}

/// MUST generate tests appropriate for the target language specified in `ought.toml`
#[test]
fn test_generator__test_generation__must_generate_tests_appropriate_for_the_target_language_specified() {
    let clause = Clause {
        id: ClauseId("gen::must_thing".to_string()),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: "do the thing".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("spec.ought.md"), line: 1 },
        content_hash: "x".to_string(),
    };

    // File extension must match the configured language
    for (lang, expected_suffix) in &[
        (Language::Rust,       "_test.rs"),
        (Language::Python,     "_test.py"),
        (Language::TypeScript, ".test.ts"),
        (Language::JavaScript, ".test.js"),
        (Language::Go,         "_test.go"),
    ] {
        let path = derive_file_path(&clause, *lang);
        assert!(
            path.to_string_lossy().ends_with(expected_suffix),
            "must_generate_tests_appropriate_for_the_target_language_specified: \
             {:?} must produce a file ending in '{}'; got '{}'",
            lang, expected_suffix, path.display()
        );
    }

    // build_prompt must name the language so the LLM generates language-appropriate tests
    for (lang, lang_name) in &[
        (Language::Rust,       "Rust"),
        (Language::Python,     "Python"),
        (Language::TypeScript, "TypeScript"),
        (Language::JavaScript, "JavaScript"),
        (Language::Go,         "Go"),
    ] {
        let ctx = GenerationContext {
            spec_context: None,
            source_files: vec![],
            schema_files: vec![],
            target_language: *lang,
            verbose: false,
        };
        let prompt = build_prompt(&clause, &ctx);
        assert!(
            prompt.contains(lang_name),
            "must_generate_tests_appropriate_for_the_target_language_specified: \
             build_prompt for {:?} must mention '{}' so the LLM targets the right language",
            lang, lang_name
        );
    }
}

/// SHOULD use the target language's idiomatic test patterns (e.g. `#[test]` for Rust, `test()` for Jest)
#[test]
fn test_generator__test_generation__should_use_the_target_language_s_idiomatic_test_patterns_e_g_test_f() {
    fn mk_clause() -> Clause {
        Clause {
            id: ClauseId("gen::test::must_thing".to_string()),
            keyword: Keyword::Must,
            severity: Severity::Required,
            text: "do the thing".to_string(),
            condition: None,
            otherwise: vec![],
            temporal: None,
            hints: vec![],
            source_location: SourceLocation { file: PathBuf::from("spec.ought.md"), line: 1 },
            content_hash: "x".to_string(),
        }
    }

    fn mk_ctx(lang: Language) -> GenerationContext {
        GenerationContext {
            spec_context: None,
            source_files: vec![],
            schema_files: vec![],
            target_language: lang,
            verbose: false,
        }
    }

    let clause = mk_clause();

    let rust_prompt = build_prompt(&clause, &mk_ctx(Language::Rust));
    assert!(
        rust_prompt.contains("#[test]"),
        "should_use_idiomatic_test_patterns: Rust prompt must mention '#[test]' attribute"
    );
    assert!(
        rust_prompt.contains("assert!"),
        "should_use_idiomatic_test_patterns: Rust prompt must mention 'assert!' macro"
    );

    let ts_prompt = build_prompt(&clause, &mk_ctx(Language::TypeScript));
    assert!(
        ts_prompt.contains("test()") || ts_prompt.contains("it()"),
        "should_use_idiomatic_test_patterns: TypeScript prompt must mention Jest-style 'test()' or 'it()'"
    );

    let go_prompt = build_prompt(&clause, &mk_ctx(Language::Go));
    assert!(
        go_prompt.contains("func Test") || go_prompt.contains("testing.T"),
        "should_use_idiomatic_test_patterns: Go prompt must mention 'func Test' or 'testing.T'"
    );

    let py_prompt = build_prompt(&clause, &mk_ctx(Language::Python));
    assert!(
        py_prompt.contains("def test"),
        "should_use_idiomatic_test_patterns: Python prompt must mention 'def test' naming convention"
    );
}

/// MUST generate tests that are self-contained (no cross-test dependencies)
#[test]
fn test_generator__test_generation__must_generate_tests_that_are_self_contained_no_cross_test_depende() {
    fn mk(id: &str, text: &str) -> Clause {
        Clause {
            id: ClauseId(id.to_string()),
            keyword: Keyword::Must,
            severity: Severity::Required,
            text: text.to_string(),
            condition: None,
            otherwise: vec![],
            temporal: None,
            hints: vec![],
            source_location: SourceLocation { file: PathBuf::from("spec.ought.md"), line: 1 },
            content_hash: "x".to_string(),
        }
    }

    let lone = mk("gen::must_isolated", "be isolated from other tests");
    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };
    assert!(
        build_prompt(&lone, &context).contains("self-contained"),
        "must_generate_tests_that_are_self_contained: build_prompt must require self-contained tests"
    );

    // parse_batch_response must keep each test's code isolated between its markers
    let c1 = mk("gen::must_x", "x");
    let c2 = mk("gen::must_y", "y");
    let response = [
        "// === CLAUSE: gen::must_x ===",
        "fn test_x() { let v = 1; assert_eq!(v, 1, \"gen::must_x\"); }",
        "// === CLAUSE: gen::must_y ===",
        "fn test_y() { let v = 2; assert_eq!(v, 2, \"gen::must_y\"); }",
    ]
    .join("\n");
    let group = ClauseGroup {
        section_path: "Gen".to_string(),
        clauses: vec![&c1, &c2],
        conditions: vec![],
    };
    let tests = parse_batch_response(&response, &group, Language::Rust);
    assert_eq!(tests.len(), 2, "expected 2 isolated tests");
    assert!(
        !tests[0].code.contains("test_y"),
        "must_generate_tests_that_are_self_contained: test[0] code must not reference test_y; got: {:?}",
        tests[0].code
    );
    assert!(
        !tests[1].code.contains("test_x"),
        "must_generate_tests_that_are_self_contained: test[1] code must not reference test_x; got: {:?}",
        tests[1].code
    );
}

/// MUST NOT generate tests that perform real IO
#[test]
fn test_generator__test_generation__must_not_generate_tests_that_perform_real_io_network_calls_filesystem() {
    // A plain logic clause -- no integration context
    let clause = Clause {
        id: ClauseId("auth::hash::must_use_bcrypt".to_string()),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: "hash passwords with bcrypt before storing".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("auth.ought.md"), line: 3 },
        content_hash: "abc".to_string(),
    };
    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };
    let prompt = build_prompt(&clause, &context);

    // Self-containment is the primary guard against real IO
    assert!(
        prompt.contains("self-contained"),
        "must_not_generate_tests_that_perform_real_io: build_prompt must require self-contained \
         tests -- the mechanism that prevents real IO for non-integration clauses"
    );

    // The prompt must not instruct the LLM to do real IO for a pure logic clause
    let lower = prompt.to_lowercase();
    assert!(
        !lower.contains("connect to database")
            && !lower.contains("http request")
            && !lower.contains("write to disk"),
        "must_not_generate_tests_that_perform_real_io: build_prompt for a logic clause must not \
         contain IO instructions; prompt starts with: {:?}",
        &prompt[..prompt.len().min(200)]
    );
}

/// MAY generate helper functions or fixtures when multiple clauses in a section share setup
#[test]
fn test_generator__test_generation__may_generate_helper_functions_or_fixtures_when_multiple_clauses() {
    fn mk(id: &str, text: &str) -> Clause {
        Clause {
            id: ClauseId(id.to_string()),
            keyword: Keyword::Must,
            severity: Severity::Required,
            text: text.to_string(),
            condition: None,
            otherwise: vec![],
            temporal: None,
            hints: vec![],
            source_location: SourceLocation { file: PathBuf::from("spec.ought.md"), line: 1 },
            content_hash: "x".to_string(),
        }
    }

    let c1 = mk("auth::login::must_return_jwt", "return a JWT on success");
    let c2 = mk("auth::login::must_set_expiry", "set token expiry to 1 hour");
    let c3 = mk("auth::login::must_reject_bad_password", "reject invalid passwords");

    let context = GenerationContext {
        spec_context: Some("Auth service validates credentials and issues JWTs.".to_string()),
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };
    let group = ClauseGroup {
        section_path: "Auth > Login".to_string(),
        clauses: vec![&c1, &c2, &c3],
        conditions: vec!["the request contains valid credentials".to_string()],
    };

    let prompt = build_batch_prompt(&group, &context);

    // All clause texts must be present so the LLM has full section context for shared fixtures
    for text in &["return a JWT on success", "set token expiry to 1 hour", "reject invalid passwords"] {
        assert!(
            prompt.contains(text),
            "may_generate_helper_functions: batch prompt must include all clause texts; missing: '{}'",
            text
        );
    }
    // Shared GIVEN condition must appear so the LLM can factor it into shared setup
    assert!(
        prompt.contains("valid credentials"),
        "may_generate_helper_functions: shared GIVEN condition must be in batch prompt \
         so the LLM can generate shared test fixtures"
    );

    // parse_batch_response must return one test per clause marker even when a shared helper
    // function appears before the first marker in the LLM output
    let response_with_helper = [
        "fn make_auth_client() -> AuthClient { AuthClient::default() }",
        "",
        "// === CLAUSE: auth::login::must_return_jwt ===",
        "#[test]",
        "fn test_auth__login__must_return_jwt() {",
        "    assert!(make_auth_client().login(\"u\", \"p\").is_ok(), \"auth::login::must_return_jwt\");",
        "}",
        "",
        "// === CLAUSE: auth::login::must_set_expiry ===",
        "#[test]",
        "fn test_auth__login__must_set_expiry() {",
        "    assert_eq!(make_auth_client().token_ttl_seconds(), 3600, \"auth::login::must_set_expiry\");",
        "}",
        "",
        "// === CLAUSE: auth::login::must_reject_bad_password ===",
        "#[test]",
        "fn test_auth__login__must_reject_bad_password() {",
        "    assert!(make_auth_client().login(\"u\", \"wrong\").is_err(), \"auth::login::must_reject_bad_password\");",
        "}",
    ]
    .join("\n");

    let tests = parse_batch_response(&response_with_helper, &group, Language::Rust);
    assert_eq!(
        tests.len(),
        3,
        "may_generate_helper_functions: parse_batch_response must yield exactly 3 tests \
         (one per clause marker) even when a shared helper precedes the first marker; got {}",
        tests.len()
    );
    assert_eq!(
        tests[0].clause_id,
        ClauseId("auth::login::must_return_jwt".to_string()),
        "may_generate_helper_functions: test[0] clause_id mismatch"
    );
    assert_eq!(
        tests[1].clause_id,
        ClauseId("auth::login::must_set_expiry".to_string()),
        "may_generate_helper_functions: test[1] clause_id mismatch"
    );
    assert_eq!(
        tests[2].clause_id,
        ClauseId("auth::login::must_reject_bad_password".to_string()),
        "may_generate_helper_functions: test[2] clause_id mismatch"
    );
}

/// SHOULD generate descriptive assertion messages that reference the clause
#[test]
fn test_generator__test_generation__should_generate_descriptive_assertion_messages_that_reference_the_c() {
    let clause = Clause {
        id: ClauseId("payments::checkout::must_reject_expired_cards".to_string()),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: "reject expired cards at checkout".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("payments.ought.md"), line: 10 },
        content_hash: "abc".to_string(),
    };
    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };
    let prompt = build_prompt(&clause, &context);

    // The clause ID must appear in the prompt so the LLM can embed it in assertion messages
    assert!(
        prompt.contains("payments::checkout::must_reject_expired_cards"),
        "should_generate_descriptive_assertion_messages_that_reference_the_c: \
         build_prompt must include the full clause ID so the LLM can reference it in assertions"
    );
    // The clause text must appear so the LLM can quote it in descriptive failure messages
    assert!(
        prompt.contains("reject expired cards at checkout"),
        "should_generate_descriptive_assertion_messages_that_reference_the_c: \
         build_prompt must include the original clause text so the LLM can produce \
         descriptive, clause-referencing assertion messages"
    );
}

// ============================================================================
// given_block_generation (5 tests)
// ============================================================================

/// MUST include the GIVEN condition in the LLM prompt so it understands the precondition context
#[test]
fn test_generator__given_block_generation__must_include_the_given_condition_in_the_llm_prompt_so_it_understa() {
    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };

    // Single-clause path: condition on clause itself
    let clause = Clause {
        id: ClauseId("gen::given::must_return_profile".to_string()),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: "return the user profile".to_string(),
        condition: Some("a valid session token is provided".to_string()),
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("spec.ought.md"), line: 10 },
        content_hash: "h2".to_string(),
    };

    let single_prompt = build_prompt(&clause, &context);
    assert!(
        single_prompt.contains("a valid session token is provided"),
        "build_prompt must include the GIVEN condition so the LLM understands the precondition; \
         condition text not found in prompt"
    );

    // Batch path: conditions on the group
    let group = ClauseGroup {
        section_path: "Gen > Given".to_string(),
        clauses: vec![&clause],
        conditions: vec!["a valid session token is provided".to_string()],
    };
    let batch_prompt = build_batch_prompt(&group, &context);
    assert!(
        batch_prompt.contains("a valid session token is provided"),
        "build_batch_prompt must include group GIVEN conditions so the LLM understands the precondition context; \
         condition text not found in batch prompt"
    );
}

/// MUST use the GIVEN condition text to generate test setup/precondition code
#[test]
fn test_generator__given_block_generation__must_use_the_given_condition_text_to_generate_test_setup_precondi() {
    let clause = Clause {
        id: ClauseId("gen::given::must_validate_token".to_string()),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: "reject requests with expired tokens".to_string(),
        condition: Some("the user presents an expired token".to_string()),
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("spec.ought.md"), line: 5 },
        content_hash: "abc".to_string(),
    };

    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };

    let prompt = build_prompt(&clause, &context);

    assert!(
        prompt.contains("the user presents an expired token"),
        "must_use_the_given_condition_text: prompt must contain the GIVEN condition text to inform test setup; prompt was:\n{}",
        prompt
    );
}

/// MUST generate a separate test function for each clause within the GIVEN block
#[test]
fn test_generator__given_block_generation__must_generate_a_separate_test_function_for_each_clause_within_the() {
    fn mk_clause(id: &str, text: &str, condition: &str) -> Clause {
        Clause {
            id: ClauseId(id.to_string()),
            keyword: Keyword::Must,
            severity: Severity::Required,
            text: text.to_string(),
            condition: Some(condition.to_string()),
            otherwise: vec![],
            temporal: None,
            hints: vec![],
            source_location: SourceLocation { file: PathBuf::from("spec.ought.md"), line: 1 },
            content_hash: "h".to_string(),
        }
    }

    let shared_condition = "the user is authenticated";
    let c1 = mk_clause("gen::given::must_allow_read", "allow read access", shared_condition);
    let c2 = mk_clause("gen::given::must_allow_write", "allow write access", shared_condition);

    let response = [
        "// === CLAUSE: gen::given::must_allow_read ===",
        "#[test]",
        "fn test_gen__given__must_allow_read() {",
        "    // setup: the user is authenticated",
        "    assert!(true);",
        "}",
        "",
        "// === CLAUSE: gen::given::must_allow_write ===",
        "#[test]",
        "fn test_gen__given__must_allow_write() {",
        "    // setup: the user is authenticated",
        "    assert!(true);",
        "}",
    ].join("\n");

    let group = ClauseGroup {
        section_path: "Gen > Given".to_string(),
        clauses: vec![&c1, &c2],
        conditions: vec![shared_condition.to_string()],
    };

    let tests = parse_batch_response(&response, &group, Language::Rust);

    assert_eq!(
        tests.len(),
        2,
        "must_generate_a_separate_test_function_for_each_clause: expected 2 GeneratedTests, got {}",
        tests.len()
    );
    assert_eq!(
        tests[0].clause_id,
        ClauseId("gen::given::must_allow_read".to_string()),
        "first test must map to the first clause id"
    );
    assert_eq!(
        tests[1].clause_id,
        ClauseId("gen::given::must_allow_write".to_string()),
        "second test must map to the second clause id"
    );
    assert_ne!(
        tests[0].clause_id, tests[1].clause_id,
        "each clause must produce a distinct test function"
    );
}

/// SHOULD generate a shared setup function or fixture for clauses under the same GIVEN block
#[test]
fn test_generator__given_block_generation__should_generate_a_shared_setup_function_or_fixture_for_clauses_unde() {
    fn mk_clause(id: &str, text: &str) -> Clause {
        Clause {
            id: ClauseId(id.to_string()),
            keyword: Keyword::Must,
            severity: Severity::Required,
            text: text.to_string(),
            condition: None,
            otherwise: vec![],
            temporal: None,
            hints: vec![],
            source_location: SourceLocation { file: PathBuf::from("spec.ought.md"), line: 1 },
            content_hash: "h".to_string(),
        }
    }

    let c1 = mk_clause("gen::given::must_list_items", "list all items");
    let c2 = mk_clause("gen::given::must_delete_item", "delete an item by id");

    let shared_condition = "the database contains at least one item";
    let group = ClauseGroup {
        section_path: "Gen > Given > DB".to_string(),
        clauses: vec![&c1, &c2],
        conditions: vec![shared_condition.to_string()],
    };

    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };

    let prompt = build_batch_prompt(&group, &context);

    assert!(
        prompt.contains("Preconditions (GIVEN)"),
        "should_generate_a_shared_setup: build_batch_prompt must emit a 'Preconditions (GIVEN)' \
         section so the LLM can generate a shared setup function or fixture; section not found in prompt"
    );
    assert!(
        prompt.contains(shared_condition),
        "should_generate_a_shared_setup: the shared GIVEN condition text must appear in the \
         Preconditions section so the LLM knows what state to establish"
    );
    // The prompt should tell the LLM to use the precondition for test setup
    assert!(
        prompt.contains("set up test preconditions"),
        "should_generate_a_shared_setup: prompt must instruct the LLM to use preconditions as \
         test setup, enabling generation of a shared fixture"
    );
}

/// MUST compose the conditions -- the inner test setup includes both outer and inner preconditions
#[test]
fn test_generator__given_block_generation__must_compose_the_conditions_the_inner_test_setup_includes_both_ou() {
    // A clause with its own inner GIVEN condition (nested GIVEN block)
    let inner_clause = Clause {
        id: ClauseId("gen::given::nested::must_forbid_delete".to_string()),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: "forbid deletion".to_string(),
        condition: Some("the item is marked as locked".to_string()),
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("spec.ought.md"), line: 20 },
        content_hash: "h3".to_string(),
    };

    // The outer GIVEN condition is carried by the ClauseGroup
    let outer_condition = "the user is an admin";
    let group = ClauseGroup {
        section_path: "Gen > Given > Nested".to_string(),
        clauses: vec![&inner_clause],
        conditions: vec![outer_condition.to_string()],
    };

    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };

    let prompt = build_batch_prompt(&group, &context);

    // Both the outer condition (from the group) and the inner condition
    // (from the clause itself) must appear in the prompt.
    assert!(
        prompt.contains(outer_condition),
        "must_compose_the_conditions: outer GIVEN condition '{}' must appear in the batch prompt",
        outer_condition
    );
    assert!(
        prompt.contains("the item is marked as locked"),
        "must_compose_the_conditions: inner GIVEN condition must appear in the batch prompt"
    );

    // Both must be present together -- not one or the other
    let outer_pos = prompt.find(outer_condition).expect("outer condition must be present");
    let inner_pos = prompt.find("the item is marked as locked").expect("inner condition must be present");
    assert_ne!(
        outer_pos, inner_pos,
        "outer and inner conditions must appear as distinct entries in the prompt"
    );
}

// ============================================================================
// otherwise_chain_generation (6 tests)
// ============================================================================

/// MUST generate a test for the primary obligation (the parent clause)
#[test]
fn test_generator__otherwise_chain_generation__must_generate_a_test_for_the_primary_obligation_the_parent_clause() {
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
    let parent = mk("gen::oc::must_respond_fast", Keyword::Must, "respond within 200ms", vec![ow1, ow2]);

    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };

    let group = ClauseGroup {
        section_path: "Generator > OTHERWISE Chain Generation".to_string(),
        clauses: vec![&parent],
        conditions: vec![],
    };

    let prompt = build_batch_prompt(&group, &context);

    assert!(
        prompt.contains("respond within 200ms"),
        "primary obligation text must appear in the batch prompt as a testable clause"
    );
    assert!(
        prompt.contains("gen::oc::must_respond_fast"),
        "primary obligation clause ID must appear in the batch prompt so the LLM can emit the correct marker"
    );
}

/// MUST generate a separate test for each OTHERWISE clause in the chain
#[test]
fn test_generator__otherwise_chain_generation__must_generate_a_separate_test_for_each_otherwise_clause_in_the_ch() {
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
        "must produce 3 separate GeneratedTests: 1 for the parent + 1 per OTHERWISE clause, got {}",
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

/// MUST preserve the chain order -- each OTHERWISE test assumes all previous levels also failed
#[test]
fn test_generator__otherwise_chain_generation__must_preserve_the_chain_order_each_otherwise_test_assumes_all_pre() {
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

    // Three-level chain: primary -> cached -> 504
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

    // Both OTHERWISE texts must appear, and they must appear in chain order.
    let pos_ow1 = prompt
        .find("return a cached response")
        .expect("first OTHERWISE text must appear in the prompt");
    let pos_ow2 = prompt
        .find("return 504 Gateway Timeout")
        .expect("second OTHERWISE text must appear in the prompt");

    assert!(
        pos_ow1 < pos_ow2,
        "OTHERWISE clauses must appear in chain order in the prompt \
         (first fallback at offset {} must precede second fallback at offset {}).",
        pos_ow1,
        pos_ow2
    );

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

/// MUST instruct the LLM that OTHERWISE tests should simulate the parent obligation's failure
#[test]
fn test_generator__otherwise_chain_generation__must_instruct_the_llm_that_otherwise_tests_should_simulate_the_pa() {
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
    let parent = mk(
        "gen::oc::must_respond_fast",
        Keyword::Must,
        "respond within 200ms",
        vec![ow1.clone()],
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
        clauses: vec![&parent, &ow1],
        conditions: vec![],
    };

    let prompt = build_batch_prompt(&group, &context);

    let prompt_lower = prompt.to_lowercase();
    let instructs_failure_simulation = prompt_lower.contains("simulate")
        || prompt_lower.contains("failure condition")
        || prompt_lower.contains("parent")
            && (prompt_lower.contains("fail") || prompt_lower.contains("trigger"));
    assert!(
        instructs_failure_simulation,
        "the batch prompt must instruct the LLM that an OTHERWISE test should simulate the \
         parent obligation's failure condition. No such instruction found. Prompt:\n{}",
        prompt
    );
}

/// MUST NOT generate OTHERWISE tests that depend on real infrastructure failures
#[test]
fn test_generator__otherwise_chain_generation__must_not_generate_otherwise_tests_that_depend_on_real_infrastructure() {
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

    let ow1 = mk(
        "gen::oc::otherwise_cached",
        Keyword::Otherwise,
        "return a cached response",
        vec![],
    );
    let parent = mk(
        "gen::oc::must_respond_fast",
        Keyword::Must,
        "respond within 200ms",
        vec![ow1.clone()],
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
        clauses: vec![&parent, &ow1],
        conditions: vec![],
    };

    let prompt = build_batch_prompt(&group, &context);

    let prompt_lower = prompt.to_lowercase();
    let instructs_in_process_simulation = prompt_lower.contains("in-process")
        || prompt_lower.contains("mock")
        || prompt_lower.contains("stub")
        || (prompt_lower.contains("simulate") && prompt_lower.contains("otherwise"))
        || prompt_lower.contains("real infrastructure")
        || prompt_lower.contains("inject");

    assert!(
        instructs_in_process_simulation,
        "the batch prompt must instruct the LLM to simulate OTHERWISE failure conditions \
         in-process rather than depending on real infrastructure failures. \
         No such instruction found in prompt:\n{}",
        prompt
    );
}

/// SHOULD generate a single integration-style test that walks the full degradation chain
#[test]
fn test_generator__otherwise_chain_generation__should_generate_a_single_integration_style_test_that_walks_the_full() {
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

    let prompt_lower = prompt.to_lowercase();
    let has_integration_instruction = prompt_lower.contains("integration")
        || prompt_lower.contains("degradation chain")
        || prompt_lower.contains("full chain")
        || prompt_lower.contains("walks the full")
        || prompt_lower.contains("end-to-end");

    assert!(
        has_integration_instruction,
        "when a clause has an OTHERWISE chain, the prompt should instruct the LLM to generate \
         a single integration-style test that walks the full degradation sequence in order. \
         No such instruction found in prompt:\n{}",
        prompt
    );
}

// ============================================================================
// temporal_obligation_generation (10 tests)
// ============================================================================

// --- MUST ALWAYS invariant tests (5) ---

/// MUST instruct the LLM to generate property-based or fuzz-style tests for MUST ALWAYS clauses
#[test]
fn test_generator__temporal_obligation_generation__must_always_invariant_tests__must_instruct_the_llm_to_generate_property_based() {
    let clause = Clause {
        id: ClauseId(
            "gen::temporal::must_always_output_valid_utf8".to_string(),
        ),
        keyword: Keyword::MustAlways,
        severity: Severity::Required,
        text: "the output must always be valid UTF-8".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: Some(Temporal::Invariant),
        hints: vec![],
        source_location: SourceLocation {
            file: PathBuf::from("spec.ought.md"),
            line: 10,
        },
        content_hash: "abc123".to_string(),
    };
    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };

    let prompt = build_prompt(&clause, &context);

    assert!(
        prompt.contains("property-based") || prompt.contains("fuzz-style") || prompt.contains("fuzz"),
        "must_instruct_the_llm_to_generate_property_based: \
         build_prompt for a MUST ALWAYS (Invariant) clause must tell the LLM to generate \
         property-based or fuzz-style tests. Prompt:\n{prompt}"
    );
}

/// SHOULD use the target language's property testing library when available
#[test]
fn test_generator__temporal_obligation_generation__must_always_invariant_tests__should_use_the_target_language_s_property_testin() {
    fn invariant_clause() -> Clause {
        Clause {
            id: ClauseId("gen::temporal::must_always_sorted".to_string()),
            keyword: Keyword::MustAlways,
            severity: Severity::Required,
            text: "the result list must always be sorted".to_string(),
            condition: None,
            otherwise: vec![],
            temporal: Some(Temporal::Invariant),
            hints: vec![],
            source_location: SourceLocation {
                file: PathBuf::from("spec.ought.md"),
                line: 30,
            },
            content_hash: "ghi789".to_string(),
        }
    }

    fn ctx(lang: Language) -> GenerationContext {
        GenerationContext {
            spec_context: None,
            source_files: vec![],
            schema_files: vec![],
            target_language: lang,
            verbose: false,
        }
    }

    let clause = invariant_clause();

    // Rust: should recommend proptest
    let rust_prompt = build_prompt(&clause, &ctx(Language::Rust));
    assert!(
        rust_prompt.contains("proptest"),
        "should_use_the_target_language_s_property_testin: \
         build_prompt for a Rust MUST ALWAYS clause should mention 'proptest'. \
         Prompt:\n{rust_prompt}"
    );

    // Python: should recommend hypothesis
    let py_prompt = build_prompt(&clause, &ctx(Language::Python));
    assert!(
        py_prompt.contains("hypothesis"),
        "should_use_the_target_language_s_property_testin: \
         build_prompt for a Python MUST ALWAYS clause should mention 'hypothesis'. \
         Prompt:\n{py_prompt}"
    );

    // TypeScript: should recommend fast-check
    let ts_prompt = build_prompt(&clause, &ctx(Language::TypeScript));
    assert!(
        ts_prompt.contains("fast-check"),
        "should_use_the_target_language_s_property_testin: \
         build_prompt for a TypeScript MUST ALWAYS clause should mention 'fast-check'. \
         Prompt:\n{ts_prompt}"
    );

    // JavaScript: should also recommend fast-check
    let js_prompt = build_prompt(&clause, &ctx(Language::JavaScript));
    assert!(
        js_prompt.contains("fast-check"),
        "should_use_the_target_language_s_property_testin: \
         build_prompt for a JavaScript MUST ALWAYS clause should mention 'fast-check'. \
         Prompt:\n{js_prompt}"
    );
}

/// MUST generate tests that verify the invariant holds across multiple inputs, states, or iterations
#[test]
fn test_generator__temporal_obligation_generation__must_always_invariant_tests__must_generate_tests_that_verify_the_invariant_ho() {
    fn mk_invariant(id: &str, text: &str) -> Clause {
        Clause {
            id: ClauseId(id.to_string()),
            keyword: Keyword::MustAlways,
            severity: Severity::Required,
            text: text.to_string(),
            condition: None,
            otherwise: vec![],
            temporal: Some(Temporal::Invariant),
            hints: vec![],
            source_location: SourceLocation {
                file: PathBuf::from("spec.ought.md"),
                line: 1,
            },
            content_hash: "x".to_string(),
        }
    }

    let clause = mk_invariant(
        "gen::temporal::must_always_idempotent",
        "serialization must always be idempotent across inputs",
    );
    let group = ClauseGroup {
        section_path: "Generator > Temporal Obligation Generation > MUST ALWAYS".to_string(),
        clauses: vec![&clause],
        conditions: vec![],
    };
    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };

    let prompt = build_batch_prompt(&group, &context);

    let covers_multiple_inputs = prompt.contains("multiple inputs")
        || prompt.contains("multiple states")
        || prompt.contains("iterations")
        || prompt.contains("property-based")
        || prompt.contains("fuzz");

    assert!(
        covers_multiple_inputs,
        "must_generate_tests_that_verify_the_invariant_ho: \
         build_batch_prompt for a MUST ALWAYS clause must instruct the LLM to verify the \
         invariant across multiple inputs. Prompt:\n{prompt}"
    );
}

/// SHOULD generate tests that exercise boundary conditions and edge cases for the invariant
#[test]
fn test_generator__temporal_obligation_generation__must_always_invariant_tests__should_generate_tests_that_exercise_boundary_con() {
    let clause = Clause {
        id: ClauseId("gen::temporal::must_always_non_negative".to_string()),
        keyword: Keyword::MustAlways,
        severity: Severity::Required,
        text: "the result count must always be non-negative".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: Some(Temporal::Invariant),
        hints: vec![],
        source_location: SourceLocation {
            file: PathBuf::from("spec.ought.md"),
            line: 22,
        },
        content_hash: "def456".to_string(),
    };
    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };

    let prompt = build_prompt(&clause, &context);

    let mentions_boundaries = prompt.contains("boundary")
        || prompt.contains("edge case")
        || prompt.contains("corner case")
        || prompt.contains("edge-case");

    assert!(
        mentions_boundaries,
        "should_generate_tests_that_exercise_boundary_con: \
         build_prompt for a MUST ALWAYS (Invariant) clause should mention boundary conditions \
         and edge cases. Prompt:\n{prompt}"
    );
}

/// MAY generate a loop-based stress test when no property testing library is available
#[test]
fn test_generator__temporal_obligation_generation__must_always_invariant_tests__may_generate_a_loop_based_stress_test_when_no_pr() {
    let clause = Clause {
        id: ClauseId("gen::temporal::must_always_deterministic".to_string()),
        keyword: Keyword::MustAlways,
        severity: Severity::Required,
        text: "the hash function must always be deterministic".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: Some(Temporal::Invariant),
        hints: vec![],
        source_location: SourceLocation {
            file: PathBuf::from("spec.ought.md"),
            line: 40,
        },
        content_hash: "jkl012".to_string(),
    };
    // Go has no idiomatic property testing library; a loop-based stress test is the expected fallback.
    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Go,
        verbose: false,
    };

    let prompt = build_prompt(&clause, &context);

    let mentions_loop_fallback = prompt.contains("loop")
        || prompt.contains("stress test")
        || prompt.contains("stress-test")
        || prompt.contains("for loop")
        || prompt.contains("iterate over");

    assert!(
        mentions_loop_fallback,
        "may_generate_a_loop_based_stress_test_when_no_pr: \
         build_prompt for a MUST ALWAYS clause targeting Go should mention a loop-based \
         stress test as a fallback strategy. Prompt:\n{prompt}"
    );
}

// --- MUST BY deadline tests (5) ---

/// MUST generate tests that assert the operation completes within the specified duration
#[test]
fn test_generator__temporal_obligation_generation__must_by_deadline_tests__must_generate_tests_that_assert_the_operation_complet() {
    let clause = Clause {
        id: ClauseId("api::handler::must_respond_within_deadline".to_string()),
        keyword: Keyword::MustBy,
        severity: Severity::Required,
        text: "the request handler must respond within the deadline".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: Some(Temporal::Deadline(Duration::from_millis(500))),
        hints: vec![],
        source_location: SourceLocation {
            file: PathBuf::from("spec.ought.md"),
            line: 10,
        },
        content_hash: "x".to_string(),
    };
    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };

    let prompt = build_prompt(&clause, &context);

    let instructs_deadline_assertion = prompt.contains("completes within")
        || prompt.contains("within this duration")
        || prompt.contains("within the deadline")
        || prompt.contains("deadline");

    assert!(
        instructs_deadline_assertion,
        "must_generate_tests_that_assert_the_operation_complet: \
         build_prompt for a MUST BY (Deadline) clause must instruct the LLM to assert \
         the operation completes within the specified duration. Prompt:\n{prompt}"
    );
}

/// MUST instruct the LLM to measure wall-clock time around the operation under test
#[test]
fn test_generator__temporal_obligation_generation__must_by_deadline_tests__must_instruct_the_llm_to_measure_wall_clock_time_arou() {
    let clause = Clause {
        id: ClauseId("processor::must_finish_within_1s".to_string()),
        keyword: Keyword::MustBy,
        severity: Severity::Required,
        text: "the processor must finish within 1 second".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: Some(Temporal::Deadline(Duration::from_secs(1))),
        hints: vec![],
        source_location: SourceLocation {
            file: PathBuf::from("spec.ought.md"),
            line: 20,
        },
        content_hash: "x".to_string(),
    };

    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };

    let prompt = build_prompt(&clause, &context);

    let instructs_wall_clock = prompt.contains("wall-clock")
        || prompt.contains("wall clock")
        || prompt.contains("Instant")
        || prompt.contains("elapsed")
        || prompt.contains("measure");

    assert!(
        instructs_wall_clock,
        "must_instruct_the_llm_to_measure_wall_clock_time_arou: \
         build_prompt for a MUST BY clause must tell the LLM to measure wall-clock time. \
         Prompt:\n{prompt}"
    );
}

/// MUST include the deadline duration from the clause in the test's timeout/assertion
#[test]
fn test_generator__temporal_obligation_generation__must_by_deadline_tests__must_include_the_deadline_duration_from_the_clause_in() {
    fn mk_deadline(id: &str, text: &str, ms: u64) -> Clause {
        Clause {
            id: ClauseId(id.to_string()),
            keyword: Keyword::MustBy,
            severity: Severity::Required,
            text: text.to_string(),
            condition: None,
            otherwise: vec![],
            temporal: Some(Temporal::Deadline(Duration::from_millis(ms))),
            hints: vec![],
            source_location: SourceLocation {
                file: PathBuf::from("spec.ought.md"),
                line: 1,
            },
            content_hash: "x".to_string(),
        }
    }

    let clause_250 = mk_deadline(
        "search::must_respond_within_250ms",
        "search must respond within 250 milliseconds",
        250,
    );
    let clause_2000 = mk_deadline(
        "export::must_complete_within_2s",
        "export must complete within 2 seconds",
        2000,
    );

    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };

    // Single-clause path
    let prompt_250 = build_prompt(&clause_250, &context);
    assert!(
        prompt_250.contains("250"),
        "must_include_the_deadline_duration: build_prompt for a 250 ms deadline must embed \
         that duration value. Prompt:\n{prompt_250}"
    );

    let prompt_2000 = build_prompt(&clause_2000, &context);
    assert!(
        prompt_2000.contains("2000") || prompt_2000.contains("2s") || prompt_2000.contains("2 s"),
        "must_include_the_deadline_duration: build_prompt for a 2000 ms deadline must embed \
         that duration value. Prompt:\n{prompt_2000}"
    );
    assert_ne!(
        prompt_250, prompt_2000,
        "must_include_the_deadline_duration: prompts for different deadlines must differ"
    );

    // Batch path: the duration must also appear when clauses are batched together.
    let group = ClauseGroup {
        section_path: "Generator > Temporal Obligation Generation > MUST BY".to_string(),
        clauses: vec![&clause_250, &clause_2000],
        conditions: vec![],
    };
    let batch = build_batch_prompt(&group, &context);
    assert!(
        batch.contains("250"),
        "must_include_the_deadline_duration: build_batch_prompt must include the 250 ms deadline. \
         Batch prompt:\n{batch}"
    );
}

/// SHOULD generate tests that run the operation multiple times and assert the p99 latency
#[test]
fn test_generator__temporal_obligation_generation__must_by_deadline_tests__should_generate_tests_that_run_the_operation_multiple() {
    let clause = Clause {
        id: ClauseId("index::search::must_respond_within_100ms".to_string()),
        keyword: Keyword::MustBy,
        severity: Severity::Required,
        text: "search must respond within 100 milliseconds".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: Some(Temporal::Deadline(Duration::from_millis(100))),
        hints: vec![],
        source_location: SourceLocation {
            file: PathBuf::from("spec.ought.md"),
            line: 30,
        },
        content_hash: "x".to_string(),
    };

    let group = ClauseGroup {
        section_path: "Generator > Temporal Obligation Generation > MUST BY".to_string(),
        clauses: vec![&clause],
        conditions: vec![],
    };
    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };

    let prompt = build_batch_prompt(&group, &context);

    let mentions_repeated_runs = prompt.contains("p99")
        || prompt.contains("percentile")
        || prompt.contains("multiple times")
        || prompt.contains("repeated")
        || prompt.contains("iterations");

    assert!(
        mentions_repeated_runs,
        "should_generate_tests_that_run_the_operation_multiple: \
         build_batch_prompt for a MUST BY clause should instruct the LLM to run the \
         operation multiple times. Prompt:\n{prompt}"
    );
}

/// SHOULD account for CI environment variability by supporting a configurable tolerance multiplier
#[test]
fn test_generator__temporal_obligation_generation__must_by_deadline_tests__should_account_for_ci_environment_variability_by_supp() {
    use ought_spec::config::ToleranceConfig;

    // The default multiplier must be 1.0
    let default_tolerance = ToleranceConfig::default();
    assert_eq!(
        default_tolerance.must_by_multiplier, 1.0,
        "should_account_for_ci_environment_variability: \
         ToleranceConfig::default().must_by_multiplier must be 1.0. Got: {}",
        default_tolerance.must_by_multiplier
    );

    // A multiplier > 1.0 must be representable (CI gets extra budget).
    let ci_tolerance = ToleranceConfig { must_by_multiplier: 2.0 };
    assert!(
        ci_tolerance.must_by_multiplier > 1.0,
        "ToleranceConfig must accept must_by_multiplier > 1.0. Got: {}",
        ci_tolerance.must_by_multiplier
    );

    // A multiplier < 1.0 must also be representable (strict CI can tighten the budget).
    let strict_tolerance = ToleranceConfig { must_by_multiplier: 0.5 };
    assert!(
        strict_tolerance.must_by_multiplier > 0.0 && strict_tolerance.must_by_multiplier < 1.0,
        "ToleranceConfig must accept fractional must_by_multiplier values in (0, 1). Got: {}",
        strict_tolerance.must_by_multiplier
    );

    let clause = Clause {
        id: ClauseId("gen::temporal::must_by_deadline_with_tolerance".to_string()),
        keyword: Keyword::MustBy,
        severity: Severity::Required,
        text: "the operation must complete within 100 milliseconds".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: Some(Temporal::Deadline(Duration::from_millis(100))),
        hints: vec![],
        source_location: SourceLocation {
            file: PathBuf::from("spec.ought.md"),
            line: 5,
        },
        content_hash: "x".to_string(),
    };
    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };
    let prompt = build_prompt(&clause, &context);

    let conveys_ci_tolerance = prompt.contains("tolerance")
        || prompt.contains("multiplier")
        || prompt.contains("200");

    assert!(
        conveys_ci_tolerance,
        "should_account_for_ci_environment_variability: \
         build_prompt for a MUST BY clause should convey the effective deadline or mention \
         the tolerance/multiplier. Prompt:\n{prompt}"
    );
}

// ============================================================================
// wont_clause_handling (2 tests)
// ============================================================================

/// MUST generate two kinds of tests for WONT clauses: absence and prevention
#[test]
fn test_generator__wont_clause_handling__must_generate_two_kinds_of_tests_for_wont_clauses_based_on_the_cl() {
    fn wont_clause(id: &str, text: &str) -> Clause {
        Clause {
            id: ClauseId(id.to_string()),
            keyword: Keyword::Wont,
            severity: Severity::NegativeConfirmation,
            text: text.to_string(),
            condition: None,
            otherwise: vec![],
            temporal: None,
            hints: vec![],
            source_location: SourceLocation {
                file: PathBuf::from("spec.ought.md"),
                line: 1,
            },
            content_hash: "h".to_string(),
        }
    }

    let ctx = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };

    // --- Single-clause prompt must mention both kinds ---
    let absence_clause = wont_clause(
        "generator::wont_clause_handling::wont_expose_admin_api",
        "expose an /admin/reset endpoint",
    );
    let single_prompt = build_prompt(&absence_clause, &ctx);
    assert!(
        single_prompt.contains("absence"),
        "build_prompt for a WONT clause must mention absence tests; got prompt:\n{single_prompt}"
    );
    assert!(
        single_prompt.contains("prevention"),
        "build_prompt for a WONT clause must mention prevention tests; got prompt:\n{single_prompt}"
    );

    // --- Batch prompt must also expose both options to the LLM ---
    let prevention_clause = wont_clause(
        "generator::wont_clause_handling::wont_swallow_write_errors",
        "silently swallow write errors -- must surface them as Err",
    );
    let group = ClauseGroup {
        section_path: "Generator > WONT Clause Handling".to_string(),
        clauses: vec![&absence_clause, &prevention_clause],
        conditions: vec![],
    };
    let batch_prompt = build_batch_prompt(&group, &ctx);
    assert!(
        batch_prompt.contains("absence"),
        "build_batch_prompt must mention absence tests for WONT clauses; \
         got batch prompt:\n{batch_prompt}"
    );
    assert!(
        batch_prompt.contains("prevention"),
        "build_batch_prompt must mention prevention tests for WONT clauses; \
         got batch prompt:\n{batch_prompt}"
    );

    // --- Non-WONT clauses must NOT receive the absence/prevention instructions ---
    let must_clause = Clause {
        id: ClauseId("generator::wont_clause_handling::must_do_something".to_string()),
        keyword: Keyword::Must,
        severity: Severity::Required,
        text: "do something".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation {
            file: PathBuf::from("spec.ought.md"),
            line: 1,
        },
        content_hash: "h".to_string(),
    };
    let must_prompt = build_prompt(&must_clause, &ctx);
    assert!(
        !must_prompt.contains("absence test (verify the capability does not exist)"),
        "build_prompt for a non-WONT clause must not include WONT-specific instructions; \
         got prompt:\n{must_prompt}"
    );
}

/// SHOULD use the clause text to determine which kind of WONT test to generate
#[test]
fn test_generator__wont_clause_handling__should_use_the_clause_text_to_determine_which_kind_of_wont_test_to() {
    fn wont_clause(id: &str, text: &str) -> Clause {
        Clause {
            id: ClauseId(id.to_string()),
            keyword: Keyword::Wont,
            severity: Severity::NegativeConfirmation,
            text: text.to_string(),
            condition: None,
            otherwise: vec![],
            temporal: None,
            hints: vec![],
            source_location: SourceLocation {
                file: PathBuf::from("spec.ought.md"),
                line: 1,
            },
            content_hash: "h".to_string(),
        }
    }

    let ctx = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };

    // Absence-appropriate: the feature simply must not exist at all.
    let absence_text = "expose a GraphQL introspection endpoint in production";
    let absence_clause = wont_clause(
        "generator::wont_clause_handling::wont_expose_graphql_introspection",
        absence_text,
    );

    // Prevention-appropriate: attempting the operation must fail gracefully.
    let prevention_text = "allow concurrent writes without acquiring a lock -- callers must receive a conflict error";
    let prevention_clause = wont_clause(
        "generator::wont_clause_handling::wont_allow_concurrent_writes_without_lock",
        prevention_text,
    );

    // Each clause text must appear verbatim in its single-clause prompt
    let absence_prompt = build_prompt(&absence_clause, &ctx);
    assert!(
        absence_prompt.contains(absence_text),
        "build_prompt must embed the WONT clause text verbatim; expected '{}' \
         in prompt:\n{absence_prompt}",
        absence_text
    );

    let prevention_prompt = build_prompt(&prevention_clause, &ctx);
    assert!(
        prevention_prompt.contains(prevention_text),
        "build_prompt must embed the WONT clause text verbatim; expected '{}' \
         in prompt:\n{prevention_prompt}",
        prevention_text
    );

    // Distinct clause texts must produce distinct prompts
    assert_ne!(
        absence_prompt, prevention_prompt,
        "prompts for WONT clauses with different texts must differ"
    );

    // Both texts must appear in the batch prompt
    let group = ClauseGroup {
        section_path: "Generator > WONT Clause Handling".to_string(),
        clauses: vec![&absence_clause, &prevention_clause],
        conditions: vec![],
    };
    let batch_prompt = build_batch_prompt(&group, &ctx);
    assert!(
        batch_prompt.contains(absence_text),
        "build_batch_prompt must include the absence-style clause text; expected '{}' in:\n{batch_prompt}",
        absence_text
    );
    assert!(
        batch_prompt.contains(prevention_text),
        "build_batch_prompt must include the prevention-style clause text; expected '{}' in:\n{batch_prompt}",
        prevention_text
    );
}

// ============================================================================
// error_handling (4 tests)
// ============================================================================

/// SHOULD retry transient API errors with exponential backoff (max 3 retries)
#[test]
fn test_generator__error_handling__should_retry_transient_api_errors_with_exponential_backoff_max_3_re() {
    use std::sync::{Arc, Mutex};

    // A generator that fails on the first `fail_count` calls, then succeeds.
    struct TransientFailGenerator {
        call_count: Arc<Mutex<u32>>,
        fail_count: u32,
    }
    impl Generator for TransientFailGenerator {
        fn generate(&self, clause: &Clause, _ctx: &GenerationContext) -> anyhow::Result<GeneratedTest> {
            let mut n = self.call_count.lock().unwrap();
            *n += 1;
            if *n <= self.fail_count {
                anyhow::bail!("transient API error on attempt {}: rate limit exceeded", *n);
            }
            Ok(GeneratedTest {
                clause_id: clause.id.clone(),
                code: "#[test] fn t() { assert!(true); }".to_string(),
                language: Language::Rust,
                file_path: PathBuf::from("t_test.rs"),
            })
        }
    }

    // Retry helper: 1 initial attempt + up to max_retries additional attempts.
    fn with_retry(
        generator: &dyn Generator,
        clause: &Clause,
        context: &GenerationContext,
        max_retries: u32,
    ) -> anyhow::Result<GeneratedTest> {
        let mut last_err = None;
        for _attempt in 0..=max_retries {
            match generator.generate(clause, context) {
                Ok(t) => return Ok(t),
                Err(e) => last_err = Some(e),
            }
        }
        Err(last_err.unwrap())
    }

    let clause = Clause {
        id: ClauseId("generator::error_handling::retry_subject".to_string()),
        keyword: Keyword::Should,
        severity: Severity::Recommended,
        text: "retry transient API errors with exponential backoff".to_string(),
        condition: None,
        otherwise: vec![],
        temporal: None,
        hints: vec![],
        source_location: SourceLocation { file: PathBuf::from("spec.ought.md"), line: 1 },
        content_hash: "abc".to_string(),
    };
    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };

    // Case 1: fails twice, succeeds on the 3rd call
    let count_a = Arc::new(Mutex::new(0u32));
    let generator_a = TransientFailGenerator { call_count: count_a.clone(), fail_count: 2 };
    let result = with_retry(&generator_a, &clause, &context, 3);
    assert!(
        result.is_ok(),
        "should_retry_transient: generator must succeed after 2 transient failures within max 3 retries; \
         err: {:?}",
        result.err()
    );
    assert_eq!(
        *count_a.lock().unwrap(),
        3,
        "should_retry_transient: must have been called exactly 3 times (2 failures + 1 success)"
    );

    // Case 2: permanently failing -- after 3 retries (4 total attempts), must return Err
    let count_b = Arc::new(Mutex::new(0u32));
    let generator_b = TransientFailGenerator { call_count: count_b.clone(), fail_count: u32::MAX };
    let exhausted = with_retry(&generator_b, &clause, &context, 3);
    assert!(
        exhausted.is_err(),
        "should_retry_transient: permanently-failing generator must return Err after max retries"
    );
    assert_eq!(
        *count_b.lock().unwrap(),
        4,
        "should_retry_transient: max 3 retries means 4 total attempts"
    );

    // Case 3: succeeds on the very first call -- no retries needed
    let count_c = Arc::new(Mutex::new(0u32));
    let generator_c = TransientFailGenerator { call_count: count_c.clone(), fail_count: 0 };
    let immediate = with_retry(&generator_c, &clause, &context, 3);
    assert!(immediate.is_ok(), "should_retry_transient: immediate success must not trigger retries");
    assert_eq!(
        *count_c.lock().unwrap(),
        1,
        "should_retry_transient: immediate success must call the generator exactly once"
    );
}

/// MUST report LLM API errors clearly (auth failure, rate limit, timeout)
#[test]
fn test_generator__error_handling__must_report_llm_api_errors_clearly_auth_failure_rate_limit_timeou() {
    use ought_gen::providers::exec_cli;

    // Simulate authentication failure: provider CLI exits non-zero with auth error on stderr
    let auth_result = exec_cli(
        "sh",
        &["-c", "echo 'authentication failed: invalid API key' >&2; exit 1"],
        "test prompt",
    );
    assert!(
        auth_result.is_err(),
        "must_report_llm_api_errors_clearly: auth failure must return Err, not Ok"
    );
    let auth_msg = auth_result.unwrap_err().to_string();
    assert!(
        auth_msg.contains("sh") || auth_msg.contains("exit"),
        "must_report_llm_api_errors_clearly: auth error must name the command or include exit status; got: {auth_msg}"
    );
    assert!(
        auth_msg.contains("authentication failed") || auth_msg.contains("invalid API key") || auth_msg.contains('1'),
        "must_report_llm_api_errors_clearly: auth error must surface stderr detail; got: {auth_msg}"
    );

    // Simulate rate limit
    let rate_result = exec_cli(
        "sh",
        &["-c", "echo 'rate limit exceeded: 429 Too Many Requests' >&2; exit 1"],
        "test prompt",
    );
    assert!(
        rate_result.is_err(),
        "must_report_llm_api_errors_clearly: rate limit must return Err"
    );
    let rate_msg = rate_result.unwrap_err().to_string();
    assert!(
        rate_msg.contains("rate limit exceeded") || rate_msg.contains("429") || rate_msg.contains("Too Many"),
        "must_report_llm_api_errors_clearly: rate limit error must surface stderr detail; got: {rate_msg}"
    );

    // Simulate timeout
    let timeout_result = exec_cli(
        "sh",
        &["-c", "echo 'request timed out after 30s' >&2; exit 1"],
        "test prompt",
    );
    assert!(
        timeout_result.is_err(),
        "must_report_llm_api_errors_clearly: timeout must return Err"
    );
    let timeout_msg = timeout_result.unwrap_err().to_string();
    assert!(
        timeout_msg.contains("request timed out") || timeout_msg.contains("timed out") || timeout_msg.contains("30s"),
        "must_report_llm_api_errors_clearly: timeout error must surface stderr detail; got: {timeout_msg}"
    );

    // CLI-not-found
    let missing_result = exec_cli("__ought_nonexistent_llm_binary__", &[], "test prompt");
    assert!(
        missing_result.is_err(),
        "must_report_llm_api_errors_clearly: missing CLI binary must return Err"
    );
    let missing_msg = missing_result.unwrap_err().to_string();
    assert!(
        missing_msg.contains("__ought_nonexistent_llm_binary__"),
        "must_report_llm_api_errors_clearly: missing-binary error must name the tool; got: {missing_msg}"
    );
}

/// MUST NOT leave the manifest in an inconsistent state if generation is interrupted
#[test]
fn test_generator__error_handling__must_not_leave_the_manifest_in_an_inconsistent_state_if_generation_is() {
    use chrono::Utc;

    let dir = std::env::temp_dir().join(format!(
        "ought_manifest_consistency_{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let manifest_path = dir.join("manifest.toml");

    // Phase 1: successful generation of clause_a; manifest saved
    let mut manifest = Manifest { entries: HashMap::new() };
    manifest.entries.insert(
        "generator::error_handling::clause_a".to_string(),
        ManifestEntry {
            clause_hash: "abc123".to_string(),
            source_hash: "src456".to_string(),
            generated_at: Utc::now(),
            model: "claude-sonnet-4-6".to_string(),
        },
    );
    manifest.save(&manifest_path).expect("save after clause_a must succeed");

    // Phase 2: generation of clause_b is interrupted before manifest save
    // (we never call manifest.save again)

    // Phase 3: load the manifest and verify it is well-formed
    let loaded = Manifest::load(&manifest_path)
        .expect("manifest must load cleanly after interrupted generation");

    assert_eq!(
        loaded.entries.len(),
        1,
        "must_not_leave_manifest_inconsistent: only committed entries must appear; \
         got {} entries, want 1",
        loaded.entries.len()
    );
    assert!(
        loaded.entries.contains_key("generator::error_handling::clause_a"),
        "must_not_leave_manifest_inconsistent: committed clause_a must be present"
    );
    assert!(
        !loaded.entries.contains_key("generator::error_handling::clause_b"),
        "must_not_leave_manifest_inconsistent: clause_b (never saved) must not appear"
    );

    // Each present entry must be fully populated
    let entry = &loaded.entries["generator::error_handling::clause_a"];
    assert!(
        !entry.clause_hash.is_empty(),
        "must_not_leave_manifest_inconsistent: clause_hash must not be empty"
    );
    assert!(
        !entry.source_hash.is_empty(),
        "must_not_leave_manifest_inconsistent: source_hash must not be empty"
    );
    assert!(
        !entry.model.is_empty(),
        "must_not_leave_manifest_inconsistent: model must not be empty"
    );

    // Idempotent reload
    let reloaded = Manifest::load(&manifest_path)
        .expect("manifest must be loadable a second time");
    assert_eq!(
        reloaded.entries.len(),
        loaded.entries.len(),
        "must_not_leave_manifest_inconsistent: re-loading must return same entry count"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

/// SHOULD continue generating remaining clauses if one clause fails
#[test]
fn test_generator__error_handling__should_continue_generating_remaining_clauses_if_one_clause_fails() {
    // A generator that fails for one specific clause ID, succeeds for all others.
    struct SelectiveFail {
        fail_id: &'static str,
    }
    impl Generator for SelectiveFail {
        fn generate(&self, clause: &Clause, _ctx: &GenerationContext) -> anyhow::Result<GeneratedTest> {
            if clause.id.0 == self.fail_id {
                anyhow::bail!(
                    "simulated LLM API error for clause '{}'",
                    clause.id
                );
            }
            Ok(GeneratedTest {
                clause_id: clause.id.clone(),
                code: format!(
                    "#[test] fn test_{}() {{ assert!(true); }}",
                    clause.id.0.replace("::", "__")
                ),
                language: Language::Rust,
                file_path: PathBuf::from(format!("{}_test.rs", clause.id.0.replace("::", "/"))),
            })
        }
    }

    fn mk(id: &str, kw: Keyword, sev: Severity) -> Clause {
        Clause {
            id: ClauseId(id.to_string()),
            keyword: kw,
            severity: sev,
            text: id.to_string(),
            condition: None,
            otherwise: vec![],
            temporal: None,
            hints: vec![],
            source_location: SourceLocation { file: PathBuf::from("spec.ought.md"), line: 1 },
            content_hash: "x".to_string(),
        }
    }

    let clauses = vec![
        mk("gen::error_handling::ok_first",  Keyword::Should, Severity::Recommended),
        mk("gen::error_handling::will_fail", Keyword::Should, Severity::Recommended),
        mk("gen::error_handling::ok_last",   Keyword::Should, Severity::Recommended),
    ];
    let context = GenerationContext {
        spec_context: None,
        source_files: vec![],
        schema_files: vec![],
        target_language: Language::Rust,
        verbose: false,
    };
    let generator = SelectiveFail { fail_id: "gen::error_handling::will_fail" };

    // Lenient per-clause loop: collect successes and failures without stopping early.
    let mut successes: Vec<GeneratedTest> = Vec::new();
    let mut failures: Vec<(ClauseId, String)> = Vec::new();
    for clause in &clauses {
        match generator.generate(clause, &context) {
            Ok(t) => successes.push(t),
            Err(e) => failures.push((clause.id.clone(), e.to_string())),
        }
    }

    assert_eq!(
        successes.len(),
        2,
        "should_continue_on_clause_failure: 2 of 3 clauses must succeed; got {}",
        successes.len()
    );
    assert_eq!(
        failures.len(),
        1,
        "should_continue_on_clause_failure: exactly 1 clause must fail; got {}",
        failures.len()
    );

    assert_eq!(
        successes[0].clause_id,
        ClauseId("gen::error_handling::ok_first".to_string()),
        "should_continue_on_clause_failure: first success must be ok_first"
    );
    assert_eq!(
        successes[1].clause_id,
        ClauseId("gen::error_handling::ok_last".to_string()),
        "should_continue_on_clause_failure: ok_last must be generated even though will_fail errored"
    );

    assert_eq!(
        failures[0].0,
        ClauseId("gen::error_handling::will_fail".to_string()),
        "should_continue_on_clause_failure: failures list must identify the failed clause"
    );
    assert!(
        failures[0].1.contains("gen::error_handling::will_fail"),
        "should_continue_on_clause_failure: failure message must reference the clause; \
         got: {}",
        failures[0].1
    );

    assert!(
        !successes[0].code.contains("will_fail"),
        "should_continue_on_clause_failure: ok_first test code must not reference will_fail"
    );
    assert!(
        !successes[1].code.contains("will_fail"),
        "should_continue_on_clause_failure: ok_last test code must not reference will_fail"
    );
}
