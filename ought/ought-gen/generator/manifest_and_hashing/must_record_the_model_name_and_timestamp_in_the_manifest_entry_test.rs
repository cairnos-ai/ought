/// MUST record the model name and timestamp in the manifest entry
#[test]
fn test_generator__manifest_and_hashing__must_record_the_model_name_and_timestamp_in_the_manifest_entry() {
    use ought_gen::manifest::{Manifest, ManifestEntry};
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