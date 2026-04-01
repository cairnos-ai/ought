use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde_json::Value;

use ought_gen::agent::AgentAssignment;
use ought_gen::manifest::{Manifest, ManifestEntry};

/// Handler for generation-mode MCP tool invocations.
///
/// These tools are called by LLM agents to drive the test generation loop:
/// reading assignments, writing tests, checking compilation, etc.
pub struct GenToolHandler {
    assignment: AgentAssignment,
    manifest: Arc<Mutex<Manifest>>,
    manifest_path: PathBuf,
}

impl GenToolHandler {
    pub fn new(
        assignment: AgentAssignment,
        manifest: Arc<Mutex<Manifest>>,
        manifest_path: PathBuf,
    ) -> Self {
        Self {
            assignment,
            manifest,
            manifest_path,
        }
    }

    /// Returns the assignment as JSON so the agent knows what to generate.
    pub fn get_assignment(&self, _args: Value) -> anyhow::Result<Value> {
        let val = serde_json::to_value(&self.assignment)
            .map_err(|e| anyhow::anyhow!("failed to serialize assignment: {}", e))?;
        Ok(val)
    }

    /// Read a source file relative to the project root.
    pub fn read_source(&self, args: Value) -> anyhow::Result<Value> {
        let path_str = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing required argument: path"))?;

        let project_root = Path::new(&self.assignment.project_root);
        let resolved = project_root.join(path_str);

        // Security: ensure the resolved path is within the project root.
        let canonical_root = project_root.canonicalize().unwrap_or_else(|_| project_root.to_path_buf());
        let canonical_path = resolved.canonicalize()
            .map_err(|e| anyhow::anyhow!("cannot resolve path '{}': {}", path_str, e))?;

        if !canonical_path.starts_with(&canonical_root) {
            anyhow::bail!("path '{}' is outside the project root", path_str);
        }

        let content = std::fs::read_to_string(&canonical_path)
            .map_err(|e| anyhow::anyhow!("failed to read '{}': {}", path_str, e))?;

        Ok(serde_json::json!({
            "path": path_str,
            "content": content,
        }))
    }

    /// List source files matching a glob pattern within the project.
    pub fn list_source_files(&self, args: Value) -> anyhow::Result<Value> {
        let pattern = args
            .get("pattern")
            .and_then(|v| v.as_str())
            .unwrap_or("**/*.rs");

        let project_root = Path::new(&self.assignment.project_root);

        let mut files = Vec::new();
        collect_files_matching(project_root, pattern, &mut files);

        // Return paths relative to project root.
        let relative_paths: Vec<String> = files
            .iter()
            .filter_map(|p| {
                p.strip_prefix(project_root)
                    .ok()
                    .map(|r| r.to_string_lossy().to_string())
            })
            .collect();

        Ok(serde_json::json!({
            "pattern": pattern,
            "files": relative_paths,
            "count": relative_paths.len(),
        }))
    }

    /// Write a single test file for a clause.
    pub fn write_test(&self, args: Value) -> anyhow::Result<Value> {
        let clause_id = args
            .get("clause_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing required argument: clause_id"))?;
        let code = args
            .get("code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing required argument: code"))?;

        let file_path = self.write_test_file(clause_id, code)?;

        Ok(serde_json::json!({
            "clause_id": clause_id,
            "file_path": file_path.to_string_lossy(),
            "status": "written",
        }))
    }

    /// Write multiple test files at once.
    pub fn write_tests_batch(&self, args: Value) -> anyhow::Result<Value> {
        let tests = args
            .get("tests")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("missing required argument: tests (array)"))?;

        let mut results = Vec::new();

        for test in tests {
            let clause_id = test
                .get("clause_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("each test must have clause_id"))?;
            let code = test
                .get("code")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("each test must have code"))?;

            match self.write_test_file(clause_id, code) {
                Ok(file_path) => {
                    results.push(serde_json::json!({
                        "clause_id": clause_id,
                        "file_path": file_path.to_string_lossy(),
                        "status": "written",
                    }));
                }
                Err(e) => {
                    results.push(serde_json::json!({
                        "clause_id": clause_id,
                        "status": "error",
                        "error": format!("{}", e),
                    }));
                }
            }
        }

        Ok(serde_json::json!({
            "results": results,
            "total": results.len(),
        }))
    }

    /// Check if written test files compile.
    pub fn check_compiles(&self, args: Value) -> anyhow::Result<Value> {
        let clause_ids = args
            .get("clause_ids")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("missing required argument: clause_ids (array)"))?;

        let test_dir = Path::new(&self.assignment.test_dir);
        let lang = self.assignment.target_language.as_str();

        let mut results = Vec::new();

        for id_val in clause_ids {
            let clause_id = id_val.as_str().unwrap_or("");
            if clause_id.is_empty() {
                continue;
            }

            let file_path = derive_test_file_path(test_dir, clause_id, lang);
            if !file_path.exists() {
                results.push(serde_json::json!({
                    "clause_id": clause_id,
                    "status": "missing",
                    "error": format!("file not found: {}", file_path.display()),
                }));
                continue;
            }

            let compile_result = check_file_compiles(&file_path, lang);
            match compile_result {
                Ok(()) => {
                    results.push(serde_json::json!({
                        "clause_id": clause_id,
                        "status": "ok",
                    }));
                }
                Err(error_msg) => {
                    results.push(serde_json::json!({
                        "clause_id": clause_id,
                        "status": "error",
                        "error": error_msg,
                    }));
                }
            }
        }

        Ok(serde_json::json!({
            "results": results,
        }))
    }

    /// Report progress to the parent process via stderr.
    pub fn report_progress(&self, args: Value) -> anyhow::Result<Value> {
        let status = args
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("in_progress");
        let message = args
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let clauses_completed = args
            .get("clauses_completed")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let clauses_total = args
            .get("clauses_total")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        eprintln!(
            "  [agent {}] {}: {} ({}/{})",
            self.assignment.id, status, message, clauses_completed, clauses_total
        );

        Ok(serde_json::json!({
            "acknowledged": true,
        }))
    }

    // ── Internal helpers ────────────────────────────────────────────────

    /// Write a test file and update the manifest (saved to disk immediately).
    fn write_test_file(&self, clause_id: &str, code: &str) -> anyhow::Result<PathBuf> {
        let test_dir = Path::new(&self.assignment.test_dir);
        let lang = self.assignment.target_language.as_str();
        let file_path = derive_test_file_path(test_dir, clause_id, lang);

        // Ensure parent directory exists.
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&file_path, code)
            .map_err(|e| anyhow::anyhow!("failed to write test file {}: {}", file_path.display(), e))?;

        // Find the content_hash from the assignment.
        let content_hash = self.find_content_hash(clause_id);

        // Update manifest.
        {
            let mut manifest = self.manifest.lock().unwrap();
            manifest.entries.insert(
                clause_id.to_string(),
                ManifestEntry {
                    clause_hash: content_hash,
                    source_hash: String::new(),
                    generated_at: chrono::Utc::now(),
                    model: "agent".to_string(),
                },
            );
            // Save manifest to disk immediately (ctrl+c safe).
            manifest.save(&self.manifest_path)?;
        }

        Ok(file_path)
    }

    /// Look up the content hash for a clause ID from the assignment data.
    fn find_content_hash(&self, clause_id: &str) -> String {
        for group in &self.assignment.groups {
            for clause in &group.clauses {
                if clause.id == clause_id {
                    return clause.content_hash.clone();
                }
                for ow in &clause.otherwise {
                    if ow.id == clause_id {
                        return ow.content_hash.clone();
                    }
                }
            }
        }
        String::new()
    }
}

/// Derive the test file path from a clause ID and language.
fn derive_test_file_path(test_dir: &Path, clause_id: &str, lang: &str) -> PathBuf {
    let ext = match lang {
        "rust" => "_test.rs",
        "python" => "_test.py",
        "typescript" => ".test.ts",
        "javascript" => ".test.js",
        "go" => "_test.go",
        _ => "_test.rs",
    };
    let path_str = clause_id.replace("::", "/");
    test_dir.join(format!("{}{}", path_str, ext))
}

/// Check if a test file compiles for the given language.
fn check_file_compiles(file_path: &Path, lang: &str) -> Result<(), String> {
    use std::process::Command;

    let output = match lang {
        "rust" => Command::new("rustc")
            .args(["--edition", "2021", "--crate-type", "lib", "--out-dir"])
            .arg(std::env::temp_dir())
            .arg(file_path)
            .output(),
        "python" => Command::new("python")
            .args(["-m", "py_compile"])
            .arg(file_path)
            .output(),
        "typescript" => Command::new("npx")
            .args(["tsc", "--noEmit"])
            .arg(file_path)
            .output(),
        "javascript" => Command::new("node")
            .args(["--check"])
            .arg(file_path)
            .output(),
        "go" => Command::new("go")
            .args(["vet"])
            .arg(file_path)
            .output(),
        _ => return Err(format!("unsupported language for compile check: {}", lang)),
    };

    match output {
        Ok(out) => {
            if out.status.success() {
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr);
                let stdout = String::from_utf8_lossy(&out.stdout);
                let detail = if stderr.trim().is_empty() {
                    stdout.trim().to_string()
                } else {
                    stderr.trim().to_string()
                };
                Err(detail)
            }
        }
        Err(e) => Err(format!("failed to run compile check: {}", e)),
    }
}

/// Recursively collect files matching a simple glob pattern.
/// Supports patterns like "**/*.rs", "src/**/*.py", "*.go".
fn collect_files_matching(root: &Path, pattern: &str, results: &mut Vec<PathBuf>) {
    // Simple pattern matching: split on "/" and handle ** as recursive.
    collect_files_recursive(root, &mut |path| {
        let rel = path.strip_prefix(root).unwrap_or(path);
        let rel_str = rel.to_string_lossy();
        if simple_glob_match(pattern, &rel_str) {
            results.push(path.to_path_buf());
        }
    });
}

/// Simple glob matching for patterns like "**/*.rs".
fn simple_glob_match(pattern: &str, path: &str) -> bool {
    // Handle the common case: **/*.ext
    if let Some(ext_pattern) = pattern.strip_prefix("**/") {
        if ext_pattern.starts_with('*') {
            // Pattern like **/*.rs
            if let Some(ext) = ext_pattern.strip_prefix('*') {
                return path.ends_with(ext);
            }
        }
        // Pattern like **/foo.rs
        return path.ends_with(ext_pattern);
    }

    // Handle *.ext at root
    if pattern.starts_with('*')
        && let Some(ext) = pattern.strip_prefix('*') {
            return path.ends_with(ext) && !path.contains('/');
        }

    // Exact prefix match with glob
    if let Some((prefix, suffix)) = pattern.split_once("**") {
        let suffix = suffix.strip_prefix('/').unwrap_or(suffix);
        if let Some(rest) = path.strip_prefix(prefix) {
            if suffix.starts_with('*')
                && let Some(ext) = suffix.strip_prefix('*') {
                    return rest.ends_with(ext);
                }
            return rest.ends_with(suffix);
        }
        return false;
    }

    // Literal match
    path == pattern
}

/// Walk a directory tree, calling the callback for each file.
fn collect_files_recursive(dir: &Path, callback: &mut dyn FnMut(&Path)) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        // Skip hidden files/directories.
        if name.starts_with('.') {
            continue;
        }
        // Skip common build directories.
        if name == "target" || name == "node_modules" || name == "__pycache__" {
            continue;
        }

        if path.is_dir() {
            collect_files_recursive(&path, callback);
        } else if path.is_file() {
            callback(&path);
        }
    }
}
