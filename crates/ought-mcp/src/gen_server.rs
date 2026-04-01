use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use ought_gen::agent::AgentAssignment;
use ought_gen::manifest::Manifest;

use crate::gen_tools::GenToolHandler;

/// MCP server specialized for generation-mode agents.
///
/// Each agent gets its own server instance with its own assignment
/// and shared manifest.
pub struct GenMcpServer {
    assignment: AgentAssignment,
    manifest: Arc<Mutex<Manifest>>,
    manifest_path: PathBuf,
}

impl GenMcpServer {
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

    /// Load from an assignment file on disk.
    pub fn from_assignment_path(assignment_path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(assignment_path)
            .map_err(|e| anyhow::anyhow!("failed to read assignment file: {}", e))?;
        let assignment: AgentAssignment = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("failed to parse assignment: {}", e))?;

        let test_dir = PathBuf::from(&assignment.test_dir);
        let manifest_path = test_dir.join("manifest.toml");
        let manifest = Manifest::load(&manifest_path).unwrap_or_default();

        Ok(Self {
            assignment,
            manifest: Arc::new(Mutex::new(manifest)),
            manifest_path,
        })
    }

    /// Serve over stdio. Reads JSON-RPC from stdin, writes responses to stdout.
    pub async fn serve_stdio(self) -> anyhow::Result<()> {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();

        let tool_handler = GenToolHandler::new(
            self.assignment,
            self.manifest,
            self.manifest_path,
        );

        while let Some(line) = lines.next_line().await? {
            let line = line.trim().to_string();
            if line.is_empty() {
                continue;
            }

            let response = Self::handle_request(&line, &tool_handler);

            // Notifications produce null responses; skip them.
            if response.is_null() {
                continue;
            }

            let response_str = serde_json::to_string(&response)
                .unwrap_or_else(|_| {
                    r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"internal serialization error"}}"#.to_string()
                });

            stdout.write_all(response_str.as_bytes()).await?;
            stdout.write_all(b"\n").await?;
            stdout.flush().await?;
        }

        Ok(())
    }

    /// Parse and route a JSON-RPC request.
    fn handle_request(raw: &str, tool_handler: &GenToolHandler) -> Value {
        let req: Value = match serde_json::from_str(raw) {
            Ok(v) => v,
            Err(_) => return jsonrpc_error(Value::Null, -32700, "Parse error"),
        };

        let id = req.get("id").cloned().unwrap_or(Value::Null);
        let method = match req.get("method").and_then(|m| m.as_str()) {
            Some(m) => m,
            None => return jsonrpc_error(id, -32600, "Invalid Request: missing method"),
        };
        let params = req.get("params").cloned().unwrap_or(serde_json::json!({}));

        match method {
            "initialize" => Self::handle_initialize(id),
            "tools/list" => Self::handle_tools_list(id),
            "resources/list" => {
                // No resources in generation mode.
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": { "resources": [] }
                })
            }
            "tools/call" => Self::handle_tool_call(id, &params, tool_handler),
            "notifications/initialized" => Value::Null,
            _ => jsonrpc_error(id, -32601, &format!("Method not found: {}", method)),
        }
    }

    fn handle_initialize(id: Value) -> Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "ought-gen",
                    "version": "0.1.0"
                }
            }
        })
    }

    fn handle_tools_list(id: Value) -> Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": gen_tool_descriptors()
        })
    }

    fn handle_tool_call(id: Value, params: &Value, handler: &GenToolHandler) -> Value {
        let tool_name = match params.get("name").and_then(|n| n.as_str()) {
            Some(n) => n,
            None => return jsonrpc_error(id, -32602, "Invalid params: missing tool name"),
        };
        let args = params
            .get("arguments")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        let result = match tool_name {
            "get_assignment" => handler.get_assignment(args),
            "read_source" => handler.read_source(args),
            "list_source_files" => handler.list_source_files(args),
            "write_test" => handler.write_test(args),
            "write_tests_batch" => handler.write_tests_batch(args),
            "check_compiles" => handler.check_compiles(args),
            "report_progress" => handler.report_progress(args),
            _ => return jsonrpc_error(id, -32602, &format!("Unknown tool: {}", tool_name)),
        };

        match result {
            Ok(value) => serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&value).unwrap_or_default()
                    }]
                }
            }),
            Err(e) => jsonrpc_error(id, -32000, &format!("{:#}", e)),
        }
    }
}

/// Tool descriptors for generation-mode MCP server.
fn gen_tool_descriptors() -> Value {
    serde_json::json!({
        "tools": [
            {
                "name": "get_assignment",
                "description": "Get the test generation assignment (clause groups with text, keywords, conditions, hints)",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "read_source",
                "description": "Read a source file relative to the project root",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "File path relative to project root"
                        }
                    },
                    "required": ["path"]
                }
            },
            {
                "name": "list_source_files",
                "description": "List source files matching a glob pattern in the project",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "pattern": {
                            "type": "string",
                            "description": "Glob pattern (e.g. '**/*.rs', 'src/**/*.py'). Defaults to '**/*.rs'"
                        }
                    }
                }
            },
            {
                "name": "write_test",
                "description": "Write a test file for a single clause. Updates the manifest immediately.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "clause_id": {
                            "type": "string",
                            "description": "The clause ID (e.g. 'auth::login::must_return_jwt')"
                        },
                        "code": {
                            "type": "string",
                            "description": "The complete test code to write"
                        }
                    },
                    "required": ["clause_id", "code"]
                }
            },
            {
                "name": "write_tests_batch",
                "description": "Write test files for multiple clauses at once",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "tests": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "clause_id": { "type": "string" },
                                    "code": { "type": "string" }
                                },
                                "required": ["clause_id", "code"]
                            },
                            "description": "Array of {clause_id, code} objects"
                        }
                    },
                    "required": ["tests"]
                }
            },
            {
                "name": "check_compiles",
                "description": "Check if written test files compile successfully",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "clause_ids": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Array of clause IDs to check"
                        }
                    },
                    "required": ["clause_ids"]
                }
            },
            {
                "name": "report_progress",
                "description": "Report generation progress to the parent process",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "status": {
                            "type": "string",
                            "description": "Status: 'in_progress', 'completed', or 'error'"
                        },
                        "message": {
                            "type": "string",
                            "description": "Human-readable progress message"
                        },
                        "clauses_completed": {
                            "type": "integer",
                            "description": "Number of clauses completed so far"
                        },
                        "clauses_total": {
                            "type": "integer",
                            "description": "Total number of clauses assigned"
                        }
                    }
                }
            }
        ]
    })
}

/// Build a JSON-RPC error response.
fn jsonrpc_error(id: Value, code: i64, message: &str) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}
