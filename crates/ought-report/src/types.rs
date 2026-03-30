use std::path::PathBuf;

use ought_spec::ClauseId;

/// LLM-generated diagnosis of why a test failed.
#[derive(Debug, Clone)]
pub struct Diagnosis {
    pub clause_id: ClauseId,
    pub explanation: String,
    pub suggested_fix: Option<SuggestedFix>,
}

/// A suggested code change to fix a failing clause.
#[derive(Debug, Clone)]
pub struct SuggestedFix {
    pub file: PathBuf,
    pub line: usize,
    pub description: String,
}

/// LLM-generated quality grade for a generated test.
#[derive(Debug, Clone)]
pub struct Grade {
    pub clause_id: ClauseId,
    pub grade: char,
    pub explanation: Option<String>,
}

/// Options controlling report output.
#[derive(Debug, Clone, Default)]
pub struct ReportOptions {
    pub diagnose: bool,
    pub grade: bool,
    pub quiet: bool,
    pub color: ColorChoice,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ColorChoice {
    #[default]
    Auto,
    Always,
    Never,
}
