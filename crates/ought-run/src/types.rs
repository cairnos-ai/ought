use std::time::Duration;

use ought_spec::ClauseId;
use serde::{Deserialize, Serialize};

/// Result of running a single clause's generated test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub clause_id: ClauseId,
    pub status: TestStatus,
    pub message: Option<String>,
    pub duration: Duration,
    pub details: TestDetails,
}

/// Pass/fail/error/skip status for a test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestStatus {
    Passed,
    Failed,
    Errored,
    Skipped,
}

/// Additional details captured from the test harness.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TestDetails {
    pub failure_message: Option<String>,
    pub stack_trace: Option<String>,
    /// Number of iterations tested (for MUST ALWAYS).
    pub iterations: Option<u64>,
    /// Measured wall-clock duration (for MUST BY).
    pub measured_duration: Option<Duration>,
}

/// Aggregated results from a full test run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResult {
    pub results: Vec<TestResult>,
    pub total_duration: Duration,
}

impl RunResult {
    pub fn passed(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.status == TestStatus::Passed)
            .count()
    }

    pub fn failed(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.status == TestStatus::Failed)
            .count()
    }

    pub fn errored(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.status == TestStatus::Errored)
            .count()
    }
}
