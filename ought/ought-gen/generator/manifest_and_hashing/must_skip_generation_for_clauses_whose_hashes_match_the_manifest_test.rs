/// MUST skip generation for clauses whose hashes match the manifest (unless `--force`)
#[test]
fn test_generator__manifest_and_hashing__must_skip_generation_for_clauses_whose_hashes_match_the_manifest() {
    use ought_gen::manifest::{Manifest, ManifestEntry};
    use ought_spec::{ClauseId, Parser};
    use chrono::Utc;
    use std::path::PathBuf;

    // ── Unit: Manifest::is_stale() drives the skip decision ─────────────────

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

    // Both hashes match → not stale → generation is skipped.
    assert!(
        !manifest.is_stale(&id, clause_hash, source_hash),
        "is_stale() must return false (skip generation) when both hashes match the manifest"
    );

    // Clause text changed → stale → regenerate.
    assert!(
        manifest.is_stale(&id, "different_hash_00", source_hash),
        "is_stale() must return true when the clause_hash differs from the stored value"
    );

    // Source file modified → stale → regenerate.
    assert!(
        manifest.is_stale(&id, clause_hash, "new_source_hash_0"),
        "is_stale() must return true when the source_hash differs from the stored value"
    );

    // No manifest entry at all → stale (first-time generation).
    let new_id = ClauseId("spec::section::must_new".to_string());
    assert!(
        manifest.is_stale(&new_id, clause_hash, source_hash),
        "is_stale() must return true for a clause with no manifest entry"
    );

    // ── Integration: --check respects matching hashes; --force overrides ────

    let dir = std::env::temp_dir()
        .join(format!("ought_skip_gen_{}", std::process::id()));
    std::fs::create_dir_all(dir.join("ought")).unwrap();

    std::fs::write(
        dir.join("ought.toml"),
        "[project]\nname = \"test\"\nversion = \"0.1.0\"\n\n\
         [specs]\nroots = [\"ought/\"]\n\n\
         [context]\nsearch_paths = []\nexclude = []\n\n\
         [generator]\nprovider = \"claude\"\n\n\
         [runner.rust]\ncommand = \"cargo test\"\ntest_dir = \"ought/ought-gen/\"\n",
    )
    .unwrap();

    let spec_text = "# Spec\n\n## Section\n\n- **MUST** validate the request\n";
    std::fs::write(dir.join("ought/spec.ought.md"), spec_text).unwrap();

    // Parse to obtain the actual clause hash and ID produced by the same algorithm
    // the CLI will use — avoids hard-coding a value that could drift.
    let parsed = Parser::parse_string(spec_text, &PathBuf::from("ought/spec.ought.md"))
        .expect("spec must parse");
    let actual_hash = parsed.sections[0].clauses[0].content_hash.clone();
    let actual_id   = parsed.sections[0].clauses[0].id.0.clone();

    // Write a manifest that exactly matches the parsed clause hash.
    std::fs::create_dir_all(dir.join("ought/ought-gen")).unwrap();
    std::fs::write(
        dir.join("ought/ought-gen/manifest.toml"),
        format!(
            "[\"{actual_id}\"]\n\
             clause_hash = \"{actual_hash}\"\n\
             source_hash = \"\"\n\
             generated_at = \"2026-01-01T00:00:00Z\"\n\
             model = \"test\"\n"
        ),
    )
    .unwrap();

    let bin = option_env!("CARGO_BIN_EXE_ought")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("ought"));

    // --check with matching manifest must exit 0 (all clauses up-to-date, none regenerated).
    let check_out = std::process::Command::new(&bin)
        .args(["generate", "--check"])
        .current_dir(&dir)
        .output()
        .expect("ought generate --check must execute");
    assert_eq!(
        check_out.status.code(),
        Some(0),
        "ought generate --check must exit 0 when all hashes match (generation skipped); \
         stderr: {}",
        String::from_utf8_lossy(&check_out.stderr)
    );

    // --check --force must exit 1: --force marks every clause stale, bypassing the manifest.
    let force_out = std::process::Command::new(&bin)
        .args(["generate", "--check", "--force"])
        .current_dir(&dir)
        .output()
        .expect("ought generate --check --force must execute");
    assert_eq!(
        force_out.status.code(),
        Some(1),
        "ought generate --check --force must exit 1 because --force ignores the manifest; \
         stderr: {}",
        String::from_utf8_lossy(&force_out.stderr)
    );

    let _ = std::fs::remove_dir_all(&dir);
}