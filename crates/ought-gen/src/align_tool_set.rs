//! [`oharness_tools::ToolSet`] adapter for the alignment agent loop.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::{Value, json};

use oharness_tools::context::ToolContext;
use oharness_tools::{ToolOutcome, ToolSet, ToolSpec};

use crate::align::{AlignAssignment, AlignChange};
use crate::align_tools::{self, ProposeChangeInput};
use crate::tools::{self, DEFAULT_READ_SOURCE_LIMIT_BYTES};

/// Tracker for changes reported or applied by the alignment agent.
#[derive(Debug, Default, Clone)]
pub struct AlignUsage {
    pub changes: Vec<AlignChange>,
}

/// In-process tool set for alignment tasks.
pub struct AlignToolSet {
    assignment: AlignAssignment,
    specs: Vec<ToolSpec>,
    usage: Arc<Mutex<AlignUsage>>,
    read_source_limit_bytes: usize,
}

impl AlignToolSet {
    pub fn new(assignment: AlignAssignment) -> Self {
        Self::with_limits(assignment, DEFAULT_READ_SOURCE_LIMIT_BYTES)
    }

    pub fn with_limits(assignment: AlignAssignment, read_source_limit_bytes: usize) -> Self {
        Self {
            assignment,
            specs: tool_specs(),
            usage: Arc::new(Mutex::new(AlignUsage::default())),
            read_source_limit_bytes,
        }
    }

    pub fn usage(&self) -> AlignUsage {
        self.usage.lock().unwrap().clone()
    }
}

#[async_trait]
impl ToolSet for AlignToolSet {
    fn specs(&self) -> &[ToolSpec] {
        &self.specs
    }

    async fn execute(&self, name: &str, input: Value, _ctx: &ToolContext) -> ToolOutcome {
        match name {
            "get_assignment" => serde_outcome(&align_tools::get_assignment(&self.assignment)),

            "read_source" => {
                let path = match input.get("path").and_then(|v| v.as_str()) {
                    Some(p) => p.to_string(),
                    None => return err("missing required argument: path"),
                };
                let start_line = input
                    .get("start_line")
                    .and_then(|v| v.as_u64())
                    .map(|n| n as usize);
                let end_line = input
                    .get("end_line")
                    .and_then(|v| v.as_u64())
                    .map(|n| n as usize);
                let project_root = std::path::PathBuf::from(&self.assignment.project_root);
                let limit = self.read_source_limit_bytes;
                match tokio::task::spawn_blocking(move || {
                    tools::read_source_with(&project_root, &path, start_line, end_line, limit)
                })
                .await
                {
                    Ok(Ok(out)) => serde_outcome(&out),
                    Ok(Err(e)) => err(e.to_string()),
                    Err(e) => err(format!("read_source task panicked: {}", e)),
                }
            }

            "list_source_files" => {
                let pattern = input
                    .get("pattern")
                    .and_then(|v| v.as_str())
                    .unwrap_or("**/*.rs")
                    .to_string();
                let project_root = std::path::PathBuf::from(&self.assignment.project_root);
                match tokio::task::spawn_blocking(move || {
                    tools::list_source_files(&project_root, &pattern)
                })
                .await
                {
                    Ok(out) => serde_outcome(&out),
                    Err(e) => err(format!("list_source_files task panicked: {}", e)),
                }
            }

            "read_spec" => {
                let target_path = match input.get("target_path").and_then(|v| v.as_str()) {
                    Some(p) => p.to_string(),
                    None => return err("missing required argument: target_path"),
                };
                let assignment = self.assignment.clone();
                match tokio::task::spawn_blocking(move || {
                    align_tools::read_spec(&assignment, &target_path)
                })
                .await
                {
                    Ok(Ok(out)) => serde_outcome(&out),
                    Ok(Err(e)) => err(e.to_string()),
                    Err(e) => err(format!("read_spec task panicked: {}", e)),
                }
            }

            "validate_spec" => {
                let content = match input.get("content").and_then(|v| v.as_str()) {
                    Some(s) => s.to_string(),
                    None => return err("missing required argument: content"),
                };
                match tokio::task::spawn_blocking(move || align_tools::validate_spec(&content))
                    .await
                {
                    Ok(out) => serde_outcome(&out),
                    Err(e) => err(format!("validate_spec task panicked: {}", e)),
                }
            }

            "propose_change" => {
                let input: ProposeChangeInput = match serde_json::from_value(input) {
                    Ok(input) => input,
                    Err(e) => return err(format!("invalid propose_change input: {}", e)),
                };
                let assignment = self.assignment.clone();
                let usage = self.usage.clone();
                match tokio::task::spawn_blocking(move || {
                    align_tools::propose_change(&assignment, input)
                })
                .await
                {
                    Ok(Ok(change)) => {
                        usage.lock().unwrap().changes.push(change.clone());
                        serde_outcome(&change)
                    }
                    Ok(Err(e)) => err(e.to_string()),
                    Err(e) => err(format!("propose_change task panicked: {}", e)),
                }
            }

            "report_progress" => {
                let status = input
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("in_progress");
                let message = input.get("message").and_then(|v| v.as_str()).unwrap_or("");
                let completed = input
                    .get("candidates_completed")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let total = input
                    .get("candidates_total")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                tools::report_progress(&self.assignment.id, status, message, completed, total);
                ok(json!({ "acknowledged": true }).to_string())
            }

            other => err(format!("unknown tool: {}", other)),
        }
    }
}

fn serde_outcome<T: serde::Serialize>(value: &T) -> ToolOutcome {
    match serde_json::to_string(value) {
        Ok(s) => ok(s),
        Err(e) => err(format!("serialization error: {}", e)),
    }
}

fn ok(s: impl Into<String>) -> ToolOutcome {
    ToolOutcome::success_text(s)
}

fn err(s: impl Into<String>) -> ToolOutcome {
    ToolOutcome::error(s, true)
}

fn tool_specs() -> Vec<ToolSpec> {
    vec![
        ToolSpec {
            name: "get_assignment".into(),
            description: "Return the reconciliation assignment: mode, candidate changes, target \
                 specs, source files, optional user focus, and whether apply mode is active."
                .into(),
            input_schema: json!({ "type": "object", "properties": {} }),
        },
        ToolSpec {
            name: "read_source".into(),
            description: "Read a source file relative to the project root. Reads are capped; call \
                 again with start_line / end_line if truncated."
                .into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "start_line": { "type": "integer" },
                    "end_line": { "type": "integer" }
                },
                "required": ["path"]
            }),
        },
        ToolSpec {
            name: "list_source_files".into(),
            description: "List source files matching a glob pattern under the project root.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "pattern": { "type": "string" }
                }
            }),
        },
        ToolSpec {
            name: "read_spec".into(),
            description: "Read an existing .ought.md spec relative to specs_root.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "target_path": { "type": "string" }
                },
                "required": ["target_path"]
            }),
        },
        ToolSpec {
            name: "validate_spec".into(),
            description: "Parse proposed .ought.md content with the canonical parser. Returns \
                 {ok, errors}; call before propose_change when content is present."
                .into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "content": { "type": "string" }
                },
                "required": ["content"]
            }),
        },
        ToolSpec {
            name: "propose_change".into(),
            description: "Record one structured alignment or discovery change. In apply mode this \
                 can write proposed spec content or stale-mark remove content. Never deletes specs."
                .into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "kind": { "type": "string", "enum": ["add", "update", "remove"] },
                    "target_spec": { "type": "string" },
                    "source_files": { "type": "array", "items": { "type": "string" } },
                    "summary": { "type": "string" },
                    "rationale": { "type": "string" },
                    "confidence": { "type": "number" },
                    "proposed_content": { "type": "string" }
                },
                "required": ["kind", "target_spec", "summary", "rationale"]
            }),
        },
        ToolSpec {
            name: "report_progress".into(),
            description: "Emit a one-line progress update to the human user.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "status": { "type": "string" },
                    "message": { "type": "string" },
                    "candidates_completed": { "type": "integer" },
                    "candidates_total": { "type": "integer" }
                }
            }),
        },
    ]
}
