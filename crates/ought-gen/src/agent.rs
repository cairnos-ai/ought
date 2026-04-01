use serde::{Deserialize, Serialize};

/// A unit of work assigned to a single agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAssignment {
    pub id: String,
    pub project_root: String,
    pub config_path: String,
    pub test_dir: String,
    pub target_language: String,
    pub groups: Vec<AssignmentGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentGroup {
    pub section_path: String,
    pub clauses: Vec<AssignmentClause>,
    pub conditions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentClause {
    pub id: String,
    pub keyword: String,
    pub text: String,
    pub condition: Option<String>,
    pub temporal: Option<String>,
    pub content_hash: String,
    pub hints: Vec<String>,
    pub otherwise: Vec<AssignmentClause>,
}

/// Results from one agent's work.
#[derive(Debug, Default)]
pub struct AgentReport {
    pub generated: usize,
    pub errors: Vec<String>,
}
