/// MUST detect and remove orphaned generated tests (clause was deleted from spec)
#[test]
fn test_generator__manifest_and_hashing__must_detect_and_remove_orphaned_generated_tests_clause_was_delete() {
    use ought_gen::manifest::{Manifest, ManifestEntry};
    use ought_spec::ClauseId;
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
        "remove_orphans must be idempotent — calling it twice with the same set must not alter the manifest"
    );

    // Edge case: empty valid set removes all remaining entries.
    manifest.remove_orphans(&[]);
    assert!(
        manifest.entries.is_empty(),
        "remove_orphans with an empty valid_ids set must clear all manifest entries"
    );
}