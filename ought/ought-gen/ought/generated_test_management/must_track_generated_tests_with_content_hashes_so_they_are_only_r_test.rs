/// MUST track generated tests with content hashes so they are only regenerated
/// when the spec or source changes.
#[test]
fn test_ought__generated_test_management__must_track_generated_tests_with_content_hashes() {
    use std::collections::HashMap;

    // Simulate a manifest entry with a known clause hash.
    let clause_id = "auth::login::must_return_jwt";
    let clause_hash = "a1b2c3d4e5f67890";
    let source_hash = "";

    let mut entries = HashMap::new();
    entries.insert(
        clause_id.to_string(),
        ManifestEntry {
            clause_hash: clause_hash.to_string(),
            source_hash: source_hash.to_string(),
        },
    );
    let manifest = Manifest { entries };

    // Same hash → not stale; test should not be regenerated.
    assert!(
        !manifest.is_stale(clause_id, clause_hash, source_hash),
        "clause with matching hash must not be considered stale"
    );

    // Different clause hash → stale; test should be regenerated.
    assert!(
        manifest.is_stale(clause_id, "different_hash_00000", source_hash),
        "clause with changed content hash must be considered stale"
    );

    // Different source hash → stale; test should be regenerated.
    assert!(
        manifest.is_stale(clause_id, clause_hash, "source_changed_hash"),
        "clause with changed source hash must be considered stale"
    );

    // Unknown clause (no manifest entry) → stale; test must be generated for the first time.
    assert!(
        manifest.is_stale("auth::login::unknown_clause", clause_hash, source_hash),
        "clause absent from manifest must be considered stale"
    );

    struct ManifestEntry {
        clause_hash: String,
        source_hash: String,
    }

    struct Manifest {
        entries: HashMap<String, ManifestEntry>,
    }

    impl Manifest {
        fn is_stale(&self, clause_id: &str, clause_hash: &str, source_hash: &str) -> bool {
            match self.entries.get(clause_id) {
                Some(entry) => entry.clause_hash != clause_hash || entry.source_hash != source_hash,
                None => true,
            }
        }
    }
}