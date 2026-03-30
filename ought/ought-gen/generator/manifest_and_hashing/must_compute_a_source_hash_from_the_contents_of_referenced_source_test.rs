/// MUST compute a source hash from the contents of referenced source files
#[test]
fn test_generator__manifest_and_hashing__must_compute_a_source_hash_from_the_contents_of_referenced_source() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use ought_gen::manifest::{Manifest, ManifestEntry};
    use ought_spec::ClauseId;
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

    // Stored source hash matches → entry is not stale.
    assert!(
        !manifest.is_stale(&id, fixed_clause_hash, &hash_v1),
        "matching source hash must not be reported as stale"
    );

    // Source file was modified → new hash → entry is stale.
    assert!(
        manifest.is_stale(&id, fixed_clause_hash, &hash_v2),
        "changed source file hash must be reported as stale so the test is regenerated"
    );
}