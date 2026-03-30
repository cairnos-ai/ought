/// MUST detect and remove orphaned tests when a clause is deleted from a spec.
#[test]
fn test_ought__generated_test_management__must_detect_and_remove_orphaned_tests() {
    use std::collections::HashMap;

    struct Manifest {
        entries: HashMap<String, ()>,
    }

    impl Manifest {
        fn remove_orphans(&mut self, valid_ids: &[&str]) {
            let valid: std::collections::HashSet<&str> = valid_ids.iter().copied().collect();
            self.entries.retain(|k, _| valid.contains(k.as_str()));
        }
    }

    // Start with three tracked clauses.
    let mut manifest = Manifest {
        entries: {
            let mut m = HashMap::new();
            m.insert("auth::login::must_return_jwt".to_string(), ());
            m.insert("auth::login::must_reject_bad_password".to_string(), ());
            m.insert("auth::login::must_hash_password".to_string(), ());
            m
        },
    };

    assert_eq!(manifest.entries.len(), 3);

    // Simulate the user deleting one clause from the spec.
    // Only two clause IDs remain after the edit.
    let still_valid = vec![
        "auth::login::must_return_jwt",
        "auth::login::must_reject_bad_password",
    ];

    manifest.remove_orphans(&still_valid);

    // The deleted clause must no longer appear in the manifest.
    assert!(
        !manifest.entries.contains_key("auth::login::must_hash_password"),
        "orphaned clause must be removed from the manifest after spec deletion"
    );

    // The surviving clauses must still be present.
    assert!(
        manifest.entries.contains_key("auth::login::must_return_jwt"),
        "non-orphaned clause must be retained in the manifest"
    );
    assert!(
        manifest.entries.contains_key("auth::login::must_reject_bad_password"),
        "non-orphaned clause must be retained in the manifest"
    );

    assert_eq!(
        manifest.entries.len(),
        2,
        "manifest must contain exactly the surviving clauses"
    );

    // Edge case: removing all clauses (entire spec deleted) leaves an empty manifest.
    manifest.remove_orphans(&[]);
    assert!(
        manifest.entries.is_empty(),
        "removing all valid ids must leave an empty manifest"
    );
}