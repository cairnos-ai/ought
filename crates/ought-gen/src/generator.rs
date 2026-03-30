use std::path::PathBuf;

use ought_spec::{Clause, ClauseId};

use crate::context::GenerationContext;

/// A test generated from a single clause.
#[derive(Debug, Clone)]
pub struct GeneratedTest {
    pub clause_id: ClauseId,
    pub code: String,
    pub language: Language,
    pub file_path: PathBuf,
}

/// Target language for test generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    Python,
    TypeScript,
    JavaScript,
    Go,
}

/// Trait implemented by each LLM provider.
///
/// Providers are invoked by exec-ing their CLI tools (e.g. `claude`, `chatgpt`,
/// `ollama`) rather than calling APIs directly. This avoids all auth management.
pub trait Generator: Send + Sync {
    /// Generate a test for a single clause, given the assembled context.
    fn generate(
        &self,
        clause: &Clause,
        context: &GenerationContext,
    ) -> anyhow::Result<GeneratedTest>;
}
