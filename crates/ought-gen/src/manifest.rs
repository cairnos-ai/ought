use std::collections::HashMap;
use std::path::Path;

use chrono::{DateTime, Utc};
use ought_spec::ClauseId;
use serde::{Deserialize, Serialize};

/// Tracks generated tests with hashes for change detection.
/// Stored as `ought/ought-gen/manifest.toml`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Manifest {
    #[serde(flatten)]
    pub entries: HashMap<String, ManifestEntry>,
}

/// A single entry in the manifest, tracking one clause's generated test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub clause_hash: String,
    pub source_hash: String,
    pub generated_at: DateTime<Utc>,
    pub model: String,
}

impl Manifest {
    /// Load the manifest from disk.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("failed to read manifest {}: {}", path.display(), e))?;
        if content.trim().is_empty() {
            return Ok(Self::default());
        }
        let manifest: Manifest = toml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("failed to parse manifest {}: {}", path.display(), e))?;
        Ok(manifest)
    }

    /// Write the manifest to disk.
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)
            .map_err(|e| anyhow::anyhow!("failed to serialize manifest: {}", e))?;
        std::fs::write(path, content)
            .map_err(|e| anyhow::anyhow!("failed to write manifest {}: {}", path.display(), e))?;
        Ok(())
    }

    /// Check if a clause's generated test is stale (hash mismatch).
    pub fn is_stale(&self, clause_id: &ClauseId, clause_hash: &str, source_hash: &str) -> bool {
        match self.entries.get(&clause_id.0) {
            Some(entry) => entry.clause_hash != clause_hash || entry.source_hash != source_hash,
            None => true,
        }
    }

    /// Remove entries for clauses that no longer exist in any spec.
    pub fn remove_orphans(&mut self, valid_ids: &[&ClauseId]) {
        let valid_strings: std::collections::HashSet<&str> =
            valid_ids.iter().map(|id| id.0.as_str()).collect();
        self.entries
            .retain(|key, _| valid_strings.contains(key.as_str()));
    }
}
