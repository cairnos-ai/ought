use std::io::Read;
use std::process::{Command, Stdio};

use ought_spec::Config;

use crate::agent::{AgentAssignment, AgentReport};

/// Orchestrates spawning LLM agents that connect to ought's MCP server
/// and drive the generation loop themselves.
pub struct Orchestrator {
    agent_command: String,
    model: Option<String>,
    parallelism: usize,
    verbose: bool,
}

/// The system prompt sent to each agent.
const AGENT_SYSTEM_PROMPT: &str = "\
You are a test generation agent for the ought behavioral test framework.

Use the provided MCP tools to generate tests:
1. Call get_assignment to see your assigned clause groups
2. For each group, use read_source and list_source_files to understand the implementation
3. Generate test functions and write them using write_test or write_tests_batch
4. Call check_compiles to verify tests compile, fix any errors
5. Call report_progress to report your status

Generate self-contained tests with the clause text as a doc comment.
Use #[test] for Rust, def test_ for Python, test() for JS/TS, func Test for Go.";

impl Orchestrator {
    pub fn new(config: &Config, verbose: bool) -> Self {
        // Determine the agent command from the provider.
        let agent_command = match config.generator.provider.to_lowercase().as_str() {
            "anthropic" | "claude" => "claude".to_string(),
            other => other.to_string(),
        };
        Self {
            agent_command,
            model: config.generator.model.clone(),
            parallelism: config.generator.parallelism.max(1),
            verbose,
        }
    }

    /// Run all assignments, spawning agents with MCP server connections.
    /// Uses threads for parallelism (not async).
    pub fn run(
        &self,
        assignments: Vec<AgentAssignment>,
    ) -> anyhow::Result<Vec<AgentReport>> {
        if assignments.is_empty() {
            return Ok(vec![]);
        }

        // Create a temp directory that lives for the duration of this run.
        let tmp_dir = tempfile::tempdir()
            .map_err(|e| anyhow::anyhow!("failed to create temp directory: {}", e))?;

        // Prepare all temp files up front.
        let prepared: Vec<(AgentAssignment, std::path::PathBuf, std::path::PathBuf)> = assignments
            .into_iter()
            .enumerate()
            .map(|(i, assignment)| {
                let assignment_path = tmp_dir.path().join(format!("assignment_{}.json", i));
                let mcp_config_path = tmp_dir.path().join(format!("mcp_config_{}.json", i));
                (assignment, assignment_path, mcp_config_path)
            })
            .collect();

        // Write assignment and MCP config files.
        for (assignment, assignment_path, mcp_config_path) in &prepared {
            let assignment_json = serde_json::to_string_pretty(assignment)
                .map_err(|e| anyhow::anyhow!("failed to serialize assignment: {}", e))?;
            std::fs::write(assignment_path, assignment_json)?;

            let mcp_config = serde_json::json!({
                "mcpServers": {
                    "ought-gen": {
                        "command": "ought",
                        "args": [
                            "mcp", "serve",
                            "--mode", "generation",
                            "--assignment", assignment_path.to_string_lossy()
                        ]
                    }
                }
            });
            let mcp_config_json = serde_json::to_string_pretty(&mcp_config)
                .map_err(|e| anyhow::anyhow!("failed to serialize MCP config: {}", e))?;
            std::fs::write(mcp_config_path, mcp_config_json)?;
        }

        // Spawn agents in batches of `parallelism`.
        let mut all_reports = Vec::new();
        let agent_command = self.agent_command.clone();
        let model = self.model.clone();
        let verbose = self.verbose;

        for chunk in prepared.chunks(self.parallelism) {
            let handles: Vec<std::thread::JoinHandle<AgentReport>> = chunk
                .iter()
                .map(|(assignment, _assignment_path, mcp_config_path)| {
                    let mcp_config_path = mcp_config_path.clone();
                    let agent_command = agent_command.clone();
                    let model = model.clone();
                    let assignment_id = assignment.id.clone();
                    let group_count = assignment.groups.len();
                    let clause_count: usize =
                        assignment.groups.iter().map(|g| g.clauses.len()).sum();

                    std::thread::spawn(move || {
                        if verbose {
                            eprintln!(
                                "  [agent {}] starting: {} groups, {} clauses",
                                assignment_id, group_count, clause_count
                            );
                        }

                        let report = run_single_agent(
                            &agent_command,
                            model.as_deref(),
                            &mcp_config_path,
                            verbose,
                        );

                        match report {
                            Ok(r) => {
                                if verbose {
                                    eprintln!(
                                        "  [agent {}] finished: {} generated, {} errors",
                                        assignment_id,
                                        r.generated,
                                        r.errors.len()
                                    );
                                }
                                r
                            }
                            Err(e) => {
                                let msg = format!(
                                    "agent {} failed: {}",
                                    assignment_id, e
                                );
                                eprintln!("  {}", msg);
                                AgentReport {
                                    generated: 0,
                                    errors: vec![msg],
                                }
                            }
                        }
                    })
                })
                .collect();

            for handle in handles {
                match handle.join() {
                    Ok(report) => all_reports.push(report),
                    Err(_) => {
                        all_reports.push(AgentReport {
                            generated: 0,
                            errors: vec!["agent thread panicked".to_string()],
                        });
                    }
                }
            }
        }

        Ok(all_reports)
    }
}

/// Spawn a single agent process and wait for it to complete.
fn run_single_agent(
    agent_command: &str,
    model: Option<&str>,
    mcp_config_path: &std::path::Path,
    verbose: bool,
) -> anyhow::Result<AgentReport> {
    let mut args: Vec<String> = vec![
        "--mcp-config".into(),
        mcp_config_path.to_string_lossy().into_owned(),
        "-p".into(),
        AGENT_SYSTEM_PROMPT.into(),
    ];

    if let Some(m) = model {
        args.push("--model".into());
        args.push(m.to_string());
    }

    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let stderr_cfg = if verbose {
        Stdio::inherit()
    } else {
        Stdio::piped()
    };

    let mut child = Command::new(agent_command)
        .args(&args_ref)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(stderr_cfg)
        .spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                anyhow::anyhow!(
                    "agent command '{}' not found. Is it installed and on PATH?",
                    agent_command
                )
            } else {
                anyhow::anyhow!("failed to spawn agent '{}': {}", agent_command, e)
            }
        })?;

    let mut stdout_output = String::new();
    if let Some(ref mut stdout) = child.stdout {
        stdout.read_to_string(&mut stdout_output)?;
    }

    let status = child.wait()?;

    if !status.success() {
        return Ok(AgentReport {
            generated: 0,
            errors: vec![format!(
                "agent exited with status {}: {}",
                status,
                stdout_output.chars().take(500).collect::<String>()
            )],
        });
    }

    // The agent writes tests via MCP tools. We estimate generated count
    // from the output, but the real work was done through the MCP server
    // which wrote files and updated the manifest directly.
    // We parse stdout for any progress reports the agent may have emitted.
    let generated = stdout_output
        .lines()
        .filter(|line| line.contains("write_test") || line.contains("write_tests_batch"))
        .count();

    Ok(AgentReport {
        generated,
        errors: vec![],
    })
}
