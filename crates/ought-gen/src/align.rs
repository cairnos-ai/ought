//! Source/spec alignment data structures.
//!
//! Alignment compares source code with existing `.ought.md` specs and
//! produces a structured reconciliation report. Applying that report is
//! explicit and writes full spec files; unsupported specs are marked pending
//! rather than deleted.

use serde::{Deserialize, Serialize};

/// A unit of work assigned to a single alignment agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignAssignment {
    pub id: String,
    pub mode: AlignMode,
    pub project_root: String,
    pub config_path: String,
    /// Absolute path to the directory where `.ought.md` files live.
    pub specs_root: String,
    /// Optional user focus for discovery runs.
    pub focus: Option<String>,
    /// When true, proposed changes may be applied to disk.
    pub apply: bool,
    /// Optional kind restriction requested by the user.
    pub only: Option<AlignChangeKind>,
    pub candidates: Vec<AlignCandidate>,
}

/// Agent workflow mode for source/spec reconciliation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AlignMode {
    /// Inspect existing mapped specs for drift against code and tests.
    Align,
    /// Discover uncovered source behavior and optionally draft new specs.
    Discover,
}

impl AlignMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Align => "align",
            Self::Discover => "discover",
        }
    }
}

/// One likely source/spec reconciliation target.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AlignCandidate {
    pub kind: AlignChangeKind,
    pub title: String,
    /// Path to the target `.ought.md` file, relative to `specs_root`.
    pub target_spec_path: String,
    /// Source files or directories, relative to project root where possible.
    pub source_files: Vec<String>,
}

/// Alignment change class.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AlignChangeKind {
    Add,
    Update,
    Remove,
}

impl AlignChangeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Update => "update",
            Self::Remove => "remove",
        }
    }
}

impl std::fmt::Display for AlignChangeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for AlignChangeKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "add" => Ok(Self::Add),
            "update" => Ok(Self::Update),
            "remove" => Ok(Self::Remove),
            other => Err(format!("unknown alignment change kind: {}", other)),
        }
    }
}

/// Full structured alignment report.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AlignReport {
    pub summary: AlignSummary,
    pub changes: Vec<AlignChange>,
    pub applied: bool,
    pub errors: Vec<String>,
}

impl AlignReport {
    pub fn from_parts(applied: bool, changes: Vec<AlignChange>, errors: Vec<String>) -> Self {
        let summary = AlignSummary::from_changes(&changes, errors.len());
        Self {
            summary,
            changes,
            applied,
            errors,
        }
    }

    pub fn merge(applied: bool, reports: Vec<Self>) -> Self {
        let mut changes = Vec::new();
        let mut errors = Vec::new();
        for report in reports {
            changes.extend(report.changes);
            errors.extend(report.errors);
        }
        changes.sort_by(|a, b| {
            a.kind
                .cmp(&b.kind)
                .then_with(|| a.target_spec.cmp(&b.target_spec))
        });
        Self::from_parts(applied, changes, errors)
    }
}

/// Counts for human and JSON summaries.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AlignSummary {
    pub total: usize,
    pub add: usize,
    pub update: usize,
    pub remove: usize,
    pub applied: usize,
    pub errors: usize,
}

impl AlignSummary {
    fn from_changes(changes: &[AlignChange], errors: usize) -> Self {
        let mut summary = Self {
            total: changes.len(),
            errors,
            ..Self::default()
        };
        for change in changes {
            match change.kind {
                AlignChangeKind::Add => summary.add += 1,
                AlignChangeKind::Update => summary.update += 1,
                AlignChangeKind::Remove => summary.remove += 1,
            }
            if change.applied_status.is_applied() {
                summary.applied += 1;
            }
            if change.applied_status.is_error() {
                summary.errors += 1;
            }
        }
        summary
    }
}

/// One proposed or applied alignment change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignChange {
    pub kind: AlignChangeKind,
    pub target_spec: String,
    pub source_files: Vec<String>,
    pub summary: String,
    pub rationale: String,
    pub confidence: Option<f64>,
    pub proposed_content: Option<String>,
    pub applied_status: AlignAppliedStatus,
}

/// Status of a proposed change.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AlignAppliedStatus {
    #[default]
    NotApplied,
    Written {
        path: String,
    },
    MarkedStale {
        path: String,
    },
    Rejected {
        errors: Vec<String>,
    },
    Skipped {
        reason: String,
    },
    Errored {
        error: String,
    },
}

impl AlignAppliedStatus {
    pub fn is_applied(&self) -> bool {
        matches!(self, Self::Written { .. } | Self::MarkedStale { .. })
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Self::Rejected { .. } | Self::Errored { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_report_includes_change_kinds_and_applied_statuses() {
        let report = AlignReport::from_parts(
            true,
            vec![
                AlignChange {
                    kind: AlignChangeKind::Add,
                    target_spec: "added.ought.md".into(),
                    source_files: vec!["src/add.rs".into()],
                    summary: "add".into(),
                    rationale: "missing".into(),
                    confidence: Some(0.9),
                    proposed_content: None,
                    applied_status: AlignAppliedStatus::Written {
                        path: "/tmp/ought/added.ought.md".into(),
                    },
                },
                AlignChange {
                    kind: AlignChangeKind::Update,
                    target_spec: "changed.ought.md".into(),
                    source_files: vec!["src/change.rs".into()],
                    summary: "update".into(),
                    rationale: "drifted".into(),
                    confidence: None,
                    proposed_content: None,
                    applied_status: AlignAppliedStatus::NotApplied,
                },
                AlignChange {
                    kind: AlignChangeKind::Remove,
                    target_spec: "removed.ought.md".into(),
                    source_files: vec!["src/removed.rs".into()],
                    summary: "remove".into(),
                    rationale: "unsupported".into(),
                    confidence: None,
                    proposed_content: None,
                    applied_status: AlignAppliedStatus::MarkedStale {
                        path: "/tmp/ought/removed.ought.md".into(),
                    },
                },
            ],
            vec![],
        );

        let value = serde_json::to_value(report).unwrap();
        assert_eq!(value["summary"]["add"], 1);
        assert_eq!(value["summary"]["update"], 1);
        assert_eq!(value["summary"]["remove"], 1);
        assert_eq!(value["summary"]["applied"], 2);
        assert_eq!(value["changes"][0]["kind"], "add");
        assert_eq!(value["changes"][0]["applied_status"]["status"], "written");
        assert_eq!(value["changes"][2]["kind"], "remove");
        assert_eq!(
            value["changes"][2]["applied_status"]["status"],
            "marked_stale"
        );
    }
}
