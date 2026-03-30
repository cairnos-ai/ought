use ought_run::RunResult;
use ought_spec::Spec;

use crate::types::Grade;

/// Run LLM-powered test quality grading on generated tests.
///
/// Evaluates whether each generated test actually validates its clause,
/// or would pass even if the behavior was broken.
///
/// Currently stubbed — returns an empty list. Will be filled in when
/// LLM integration is more mature.
pub fn grade(
    _results: &RunResult,
    _specs: &[Spec],
) -> anyhow::Result<Vec<Grade>> {
    Ok(vec![])
}
