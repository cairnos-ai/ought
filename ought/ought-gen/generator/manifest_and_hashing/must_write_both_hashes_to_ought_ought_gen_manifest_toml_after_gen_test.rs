/// MUST write both hashes to `ought/ought-gen/manifest.toml` after generation
#[test]
fn test_generator__manifest_and_hashing__must_write_both_hashes_to_ought_ought_gen_manifest_toml_after_gen() {
    use ought_gen::manifest::{Manifest, ManifestEntry};
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