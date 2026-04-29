use chrono::{DateTime, Utc};
use ought_spec::ClauseId;

// -- Blame --

/// Results from `ought debug blame`.
#[derive(Debug, Clone)]
pub struct BlameResult {
    pub clause_id: ClauseId,
    pub last_passed: Option<DateTime<Utc>>,
    pub first_failed: Option<DateTime<Utc>>,
    pub likely_commit: Option<CommitInfo>,
    pub narrative: String,
    pub suggested_fix: Option<String>,
}

/// Information about a git commit.
#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub date: DateTime<Utc>,
}

// -- Bisect --

/// Results from `ought debug bisect`.
#[derive(Debug, Clone)]
pub struct BisectResult {
    pub clause_id: ClauseId,
    pub breaking_commit: CommitInfo,
    pub diff_summary: String,
}
