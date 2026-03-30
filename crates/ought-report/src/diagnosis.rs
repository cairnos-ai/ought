use ought_run::RunResult;
use ought_spec::Spec;

use crate::types::Diagnosis;

/// Run LLM-powered failure diagnosis on all failing clauses.
///
/// Sends the failing clause, generated test, failure output, and relevant
/// source code to the LLM. Returns a narrative explanation and suggested fix.
///
/// Currently stubbed — returns an empty list. Will be filled in when
/// LLM integration is more mature.
pub fn diagnose(
    _results: &RunResult,
    _specs: &[Spec],
) -> anyhow::Result<Vec<Diagnosis>> {
    Ok(vec![])
}
