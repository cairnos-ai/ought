use serde_json::Value;

/// Handler for MCP tool invocations.
///
/// Each tool maps to an `ought` CLI command and returns structured JSON.
pub struct ToolHandler {
    // will hold references to config, specs, etc.
}

impl ToolHandler {
    pub fn ought_run(&self, args: Value) -> anyhow::Result<Value> {
        todo!()
    }

    pub fn ought_generate(&self, args: Value) -> anyhow::Result<Value> {
        todo!()
    }

    pub fn ought_check(&self, args: Value) -> anyhow::Result<Value> {
        todo!()
    }

    pub fn ought_inspect(&self, args: Value) -> anyhow::Result<Value> {
        todo!()
    }

    pub fn ought_status(&self, args: Value) -> anyhow::Result<Value> {
        todo!()
    }

    pub fn ought_survey(&self, args: Value) -> anyhow::Result<Value> {
        todo!()
    }

    pub fn ought_audit(&self, args: Value) -> anyhow::Result<Value> {
        todo!()
    }

    pub fn ought_blame(&self, args: Value) -> anyhow::Result<Value> {
        todo!()
    }

    pub fn ought_bisect(&self, args: Value) -> anyhow::Result<Value> {
        todo!()
    }
}
