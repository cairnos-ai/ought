//! Tool primitives for source/spec alignment.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use ought_spec::parser::{OughtMdParser, Parser as _};

use crate::align::{AlignAppliedStatus, AlignAssignment, AlignChange, AlignChangeKind};

// ── Output/input types ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadSpecOutput {
    pub target_path: String,
    pub resolved_path: String,
    pub exists: bool,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateSpecOutput {
    pub ok: bool,
    /// When `ok` is false, one formatted error per parse failure.
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposeChangeInput {
    pub kind: AlignChangeKind,
    pub target_spec: String,
    #[serde(default)]
    pub source_files: Vec<String>,
    pub summary: String,
    pub rationale: String,
    pub confidence: Option<f64>,
    pub proposed_content: Option<String>,
}

// ── Primitives ──────────────────────────────────────────────────────────

pub fn get_assignment(assignment: &AlignAssignment) -> AlignAssignment {
    assignment.clone()
}

pub fn read_spec(assignment: &AlignAssignment, target_rel: &str) -> anyhow::Result<ReadSpecOutput> {
    let resolved = resolve_under_specs_root(&assignment.specs_root, target_rel)?;
    let exists = resolved.exists();
    let content = if exists {
        Some(
            std::fs::read_to_string(&resolved)
                .map_err(|e| anyhow::anyhow!("failed to read {}: {}", resolved.display(), e))?,
        )
    } else {
        None
    };

    Ok(ReadSpecOutput {
        target_path: target_rel.to_string(),
        resolved_path: resolved.to_string_lossy().into_owned(),
        exists,
        content,
    })
}

pub fn validate_spec(content: &str) -> ValidateSpecOutput {
    match parse_spec_content(content) {
        Ok(()) => ValidateSpecOutput {
            ok: true,
            errors: Vec::new(),
        },
        Err(errors) => ValidateSpecOutput {
            ok: false,
            errors: errors
                .into_iter()
                .map(|e| format!("line {}: {}", e.line, e.message))
                .collect(),
        },
    }
}

/// Record a proposed alignment change. When `assignment.apply` is true,
/// this also applies the change to disk.
pub fn propose_change(
    assignment: &AlignAssignment,
    input: ProposeChangeInput,
) -> anyhow::Result<AlignChange> {
    if let Some(only) = assignment.only
        && input.kind != only
    {
        return Ok(build_change(
            input,
            AlignAppliedStatus::Skipped {
                reason: format!("run is restricted to {} changes", only),
            },
        ));
    }

    let mut status = AlignAppliedStatus::NotApplied;

    if let Some(ref content) = input.proposed_content {
        let validation = validate_spec(content);
        if !validation.ok {
            status = AlignAppliedStatus::Rejected {
                errors: validation.errors,
            };
        }
    }

    if assignment.apply && matches!(status, AlignAppliedStatus::NotApplied) {
        status = match input.kind {
            AlignChangeKind::Add => {
                let Some(content) = input.proposed_content.as_deref() else {
                    return Ok(build_change(
                        input,
                        AlignAppliedStatus::Rejected {
                            errors: vec![
                                "add changes require proposed_content when applying".to_string(),
                            ],
                        },
                    ));
                };
                write_full_spec(assignment, &input.target_spec, content, false)
            }
            AlignChangeKind::Update => {
                let Some(content) = input.proposed_content.as_deref() else {
                    return Ok(build_change(
                        input,
                        AlignAppliedStatus::Rejected {
                            errors: vec![
                                "update changes require proposed_content when applying".to_string(),
                            ],
                        },
                    ));
                };
                write_full_spec(assignment, &input.target_spec, content, true)
            }
            AlignChangeKind::Remove => {
                if let Some(content) = input.proposed_content.as_deref() {
                    write_full_spec(assignment, &input.target_spec, content, true)
                } else {
                    mark_spec_stale(assignment, &input.target_spec)
                }
            }
        };
    }

    Ok(build_change(input, status))
}

pub fn mark_spec_stale(assignment: &AlignAssignment, target_rel: &str) -> AlignAppliedStatus {
    let resolved = match resolve_under_specs_root(&assignment.specs_root, target_rel) {
        Ok(path) => path,
        Err(e) => {
            return AlignAppliedStatus::Errored {
                error: e.to_string(),
            };
        }
    };

    let content = match std::fs::read_to_string(&resolved) {
        Ok(content) => content,
        Err(e) => {
            return AlignAppliedStatus::Errored {
                error: format!("failed to read {}: {}", resolved.display(), e),
            };
        }
    };
    let updated = mark_content_pending(&content);
    if updated == content {
        return AlignAppliedStatus::Skipped {
            reason: "no non-pending testable clauses found".to_string(),
        };
    }
    let validation = validate_spec(&updated);
    if !validation.ok {
        return AlignAppliedStatus::Rejected {
            errors: validation.errors,
        };
    }
    match std::fs::write(&resolved, updated) {
        Ok(()) => AlignAppliedStatus::MarkedStale {
            path: resolved.to_string_lossy().into_owned(),
        },
        Err(e) => AlignAppliedStatus::Errored {
            error: format!("failed to write {}: {}", resolved.display(), e),
        },
    }
}

pub fn mark_content_pending(content: &str) -> String {
    let mut out = String::with_capacity(content.len() + 32);
    for line in content.lines() {
        out.push_str(&mark_line_pending(line));
        out.push('\n');
    }
    if !content.ends_with('\n') {
        out.pop();
    }
    out
}

fn mark_line_pending(line: &str) -> String {
    let trimmed = line.trim_start();
    let indent_len = line.len() - trimmed.len();
    if !trimmed.starts_with("- **")
        || trimmed.starts_with("- **PENDING ")
        || trimmed.starts_with("- **GIVEN**")
    {
        return line.to_string();
    }

    let keyword_start = indent_len + "- **".len();
    let mut updated = line.to_string();
    updated.insert_str(keyword_start, "PENDING ");
    updated
}

// ── Internals ───────────────────────────────────────────────────────────

fn build_change(input: ProposeChangeInput, applied_status: AlignAppliedStatus) -> AlignChange {
    AlignChange {
        kind: input.kind,
        target_spec: input.target_spec,
        source_files: input.source_files,
        summary: input.summary,
        rationale: input.rationale,
        confidence: input.confidence,
        proposed_content: input.proposed_content,
        applied_status,
    }
}

fn write_full_spec(
    assignment: &AlignAssignment,
    target_rel: &str,
    content: &str,
    allow_existing: bool,
) -> AlignAppliedStatus {
    let validation = validate_spec(content);
    if !validation.ok {
        return AlignAppliedStatus::Rejected {
            errors: validation.errors,
        };
    }

    let resolved = match resolve_under_specs_root(&assignment.specs_root, target_rel) {
        Ok(path) => path,
        Err(e) => {
            return AlignAppliedStatus::Errored {
                error: e.to_string(),
            };
        }
    };
    if resolved.exists() && !allow_existing {
        return AlignAppliedStatus::Rejected {
            errors: vec![format!(
                "{} already exists; classify this as an update instead of add",
                target_rel
            )],
        };
    }
    if let Some(parent) = resolved.parent()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        return AlignAppliedStatus::Errored {
            error: format!("failed to create directory {}: {}", parent.display(), e),
        };
    }
    match std::fs::write(&resolved, content) {
        Ok(()) => AlignAppliedStatus::Written {
            path: resolved.to_string_lossy().into_owned(),
        },
        Err(e) => AlignAppliedStatus::Errored {
            error: format!("failed to write {}: {}", resolved.display(), e),
        },
    }
}

fn parse_spec_content(content: &str) -> Result<(), Vec<ought_spec::ParseError>> {
    let parser = OughtMdParser;
    parser
        .parse_string(content, Path::new("<align-draft>.ought.md"))
        .map(|_| ())
}

fn resolve_under_specs_root(specs_root: &str, target_rel: &str) -> anyhow::Result<PathBuf> {
    let root = PathBuf::from(specs_root);
    let resolved = root.join(target_rel);
    let canonical_root = root.canonicalize().unwrap_or_else(|_| root.clone());
    let normalized = lexical_normalize(&resolved);
    if !normalized.starts_with(&canonical_root) && !normalized.starts_with(&root) {
        anyhow::bail!(
            "target_path '{}' resolves outside specs_root '{}'",
            target_rel,
            root.display()
        );
    }
    Ok(resolved)
}

fn lexical_normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn assignment(tmp: &Path, apply: bool) -> AlignAssignment {
        AlignAssignment {
            id: "test".into(),
            mode: crate::align::AlignMode::Discover,
            project_root: tmp.to_string_lossy().into_owned(),
            config_path: tmp.join("ought.toml").to_string_lossy().into_owned(),
            specs_root: tmp.join("ought").to_string_lossy().into_owned(),
            focus: None,
            apply,
            only: None,
            candidates: vec![],
        }
    }

    const VALID_SPEC: &str = "# Demo\n\ncontext: example\n\n## Behavior\n\n- **MUST** work\n";

    #[test]
    fn report_mode_does_not_write() {
        let tmp = tempdir().unwrap();
        let asn = assignment(tmp.path(), false);
        let change = propose_change(
            &asn,
            ProposeChangeInput {
                kind: AlignChangeKind::Add,
                target_spec: "demo.ought.md".into(),
                source_files: vec!["src/demo.rs".into()],
                summary: "add demo".into(),
                rationale: "missing".into(),
                confidence: Some(0.9),
                proposed_content: Some(VALID_SPEC.into()),
            },
        )
        .unwrap();

        assert!(matches!(
            change.applied_status,
            AlignAppliedStatus::NotApplied
        ));
        assert!(!tmp.path().join("ought/demo.ought.md").exists());
    }

    #[test]
    fn apply_add_writes_new_spec() {
        let tmp = tempdir().unwrap();
        let asn = assignment(tmp.path(), true);
        let change = propose_change(
            &asn,
            ProposeChangeInput {
                kind: AlignChangeKind::Add,
                target_spec: "demo.ought.md".into(),
                source_files: vec![],
                summary: "add demo".into(),
                rationale: "missing".into(),
                confidence: None,
                proposed_content: Some(VALID_SPEC.into()),
            },
        )
        .unwrap();

        assert!(matches!(
            change.applied_status,
            AlignAppliedStatus::Written { .. }
        ));
        assert_eq!(
            std::fs::read_to_string(tmp.path().join("ought/demo.ought.md")).unwrap(),
            VALID_SPEC
        );
    }

    #[test]
    fn apply_update_writes_existing_spec() {
        let tmp = tempdir().unwrap();
        let asn = assignment(tmp.path(), true);
        let spec_path = tmp.path().join("ought/demo.ought.md");
        std::fs::create_dir_all(spec_path.parent().unwrap()).unwrap();
        std::fs::write(&spec_path, VALID_SPEC).unwrap();
        let updated = "# Demo\n\ncontext: updated\n\n## Behavior\n\n- **MUST** still work\n";

        let change = propose_change(
            &asn,
            ProposeChangeInput {
                kind: AlignChangeKind::Update,
                target_spec: "demo.ought.md".into(),
                source_files: vec!["src/demo.rs".into()],
                summary: "update demo".into(),
                rationale: "behavior drifted".into(),
                confidence: Some(0.8),
                proposed_content: Some(updated.into()),
            },
        )
        .unwrap();

        assert!(matches!(
            change.applied_status,
            AlignAppliedStatus::Written { .. }
        ));
        assert_eq!(std::fs::read_to_string(spec_path).unwrap(), updated);
    }

    #[test]
    fn apply_remove_marks_existing_spec_pending() {
        let tmp = tempdir().unwrap();
        let asn = assignment(tmp.path(), true);
        let spec_path = tmp.path().join("ought/demo.ought.md");
        std::fs::create_dir_all(spec_path.parent().unwrap()).unwrap();
        std::fs::write(
            &spec_path,
            "# Demo\n\n## Behavior\n\n- **MUST** work\n  - **OTHERWISE** degrade\n",
        )
        .unwrap();

        let change = propose_change(
            &asn,
            ProposeChangeInput {
                kind: AlignChangeKind::Remove,
                target_spec: "demo.ought.md".into(),
                source_files: vec!["src/deleted.rs".into()],
                summary: "source removed".into(),
                rationale: "source mapping no longer exists".into(),
                confidence: None,
                proposed_content: None,
            },
        )
        .unwrap();

        assert!(matches!(
            change.applied_status,
            AlignAppliedStatus::MarkedStale { .. }
        ));
        let updated = std::fs::read_to_string(spec_path).unwrap();
        assert!(updated.contains("- **PENDING MUST** work"));
        assert!(updated.contains("  - **PENDING OTHERWISE** degrade"));
    }

    #[test]
    fn malformed_proposed_spec_is_rejected_before_write() {
        let tmp = tempdir().unwrap();
        let asn = assignment(tmp.path(), true);
        let change = propose_change(
            &asn,
            ProposeChangeInput {
                kind: AlignChangeKind::Add,
                target_spec: "demo.ought.md".into(),
                source_files: vec![],
                summary: "add invalid".into(),
                rationale: "draft was malformed".into(),
                confidence: None,
                proposed_content: Some("# Demo\n\n- **PENDING GIVEN** invalid\n".into()),
            },
        )
        .unwrap();

        assert!(matches!(
            change.applied_status,
            AlignAppliedStatus::Rejected { .. }
        ));
        assert!(!tmp.path().join("ought/demo.ought.md").exists());
    }

    #[test]
    fn apply_rejects_path_escape() {
        let tmp = tempdir().unwrap();
        let asn = assignment(tmp.path(), true);
        let change = propose_change(
            &asn,
            ProposeChangeInput {
                kind: AlignChangeKind::Add,
                target_spec: "../demo.ought.md".into(),
                source_files: vec![],
                summary: "add demo".into(),
                rationale: "missing".into(),
                confidence: None,
                proposed_content: Some(VALID_SPEC.into()),
            },
        )
        .unwrap();

        assert!(matches!(
            change.applied_status,
            AlignAppliedStatus::Errored { .. }
        ));
        assert!(!tmp.path().join("demo.ought.md").exists());
    }

    #[test]
    fn mark_stale_adds_pending_to_testable_clauses() {
        let input = "# Demo\n\n## Rules\n\n- **GIVEN** a user exists\n  - **MUST** return the user\n  - **OTHERWISE** return 404\n- **PENDING SHOULD** already be pending\n";
        let output = mark_content_pending(input);
        assert!(output.contains("- **GIVEN** a user exists"));
        assert!(output.contains("  - **PENDING MUST** return the user"));
        assert!(output.contains("  - **PENDING OTHERWISE** return 404"));
        assert!(output.contains("- **PENDING SHOULD** already be pending"));
    }
}
