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

/// A group of related clauses from one section, to be generated in a single
/// LLM call. GIVEN conditions are included as context, not as testable clauses.
#[derive(Debug, Clone)]
pub struct ClauseGroup<'a> {
    /// Section path for display (e.g. "Auth API > Login").
    pub section_path: String,
    /// Testable clauses (MUST, SHOULD, MAY, WONT, MUST ALWAYS, MUST BY, OTHERWISE).
    pub clauses: Vec<&'a Clause>,
    /// GIVEN conditions that scope clauses in this group (context, not testable).
    pub conditions: Vec<String>,
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

    /// Generate tests for a batch of related clauses in a single LLM call.
    /// Returns one GeneratedTest per clause in the group.
    ///
    /// Default implementation falls back to per-clause generation.
    fn generate_batch(
        &self,
        group: &ClauseGroup<'_>,
        context: &GenerationContext,
    ) -> anyhow::Result<Vec<GeneratedTest>> {
        let mut results = Vec::new();
        for clause in &group.clauses {
            results.push(self.generate(clause, context)?);
        }
        Ok(results)
    }
}
