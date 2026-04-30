use std::error::Error;
use std::fmt::Debug;

use oharness_core::{RunError, Termination};

pub(crate) fn termination_error_detail(termination: &Termination) -> Option<String> {
    match termination {
        Termination::Completed { .. } => None,
        Termination::Truncated { limit } => Some(format!("agent truncated: {limit:?}")),
        Termination::Failed { error, at_turn } => Some(run_error_detail(error, *at_turn)),
        Termination::Interrupted { reason } => Some(format!("agent interrupted: {reason:?}")),
    }
}

pub(crate) fn run_error_detail(error: &RunError, at_turn: u32) -> String {
    format!(
        "agent failed at turn {at_turn}\ncategory: {:?}\nmessage: {}\nraw: {:#?}",
        error.category, error.message, error
    )
}

pub(crate) fn error_detail<E>(context: &str, error: &E) -> String
where
    E: Error + Debug,
{
    let mut out = format!("{context}: {error}");
    let debug = format!("{error:#?}");
    if debug != error.to_string() {
        out.push_str("\nraw: ");
        out.push_str(&debug);
    }

    let mut source = error.source();
    if source.is_some() {
        out.push_str("\ncaused by:");
    }
    let mut index = 1;
    while let Some(err) = source {
        out.push_str(&format!("\n  {index}. {err}"));
        source = err.source();
        index += 1;
    }

    out
}

#[cfg(test)]
mod tests {
    use oharness_core::{RunErrorCategory, TruncationLimit};

    use super::*;

    #[test]
    fn termination_failure_includes_category_turn_message_and_raw_debug() {
        let termination = Termination::Failed {
            error: RunError {
                category: RunErrorCategory::Llm,
                message: "provider unavailable".into(),
            },
            at_turn: 7,
        };

        let detail = termination_error_detail(&termination).unwrap();

        assert!(detail.contains("turn 7"));
        assert!(detail.contains("category: Llm"));
        assert!(detail.contains("provider unavailable"));
        assert!(detail.contains("raw: RunError"));
    }

    #[test]
    fn truncation_reports_limit() {
        let termination = Termination::Truncated {
            limit: TruncationLimit::MaxTurns(3),
        };

        assert_eq!(
            termination_error_detail(&termination).unwrap(),
            "agent truncated: MaxTurns(3)"
        );
    }
}
