//! Per-assignment in-process agent loops for source/spec alignment.

use std::sync::Arc;

use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use oharness_core::Task;
use oharness_llm::Llm;
use oharness_loop::{Agent, ReactLoop};

use crate::align::{AlignAssignment, AlignMode, AlignReport};
use crate::align_tool_set::AlignToolSet;
use crate::config::GeneratorConfig;

/// The canonical `.ought.md` grammar, embedded at compile time so agents
/// draft against the grammar that matches this binary's parser.
const GRAMMAR_MD: &str = include_str!("../grammar.md");

pub struct AlignOrchestrator {
    config: GeneratorConfig,
    verbose: bool,
}

impl AlignOrchestrator {
    pub fn new(config: GeneratorConfig, verbose: bool) -> Self {
        Self { config, verbose }
    }

    pub async fn run(self, assignments: Vec<AlignAssignment>) -> anyhow::Result<AlignReport> {
        if assignments.is_empty() {
            return Ok(AlignReport::from_parts(false, vec![], vec![]));
        }

        let applied = assignments.iter().any(|assignment| assignment.apply);
        let llm = crate::orchestrator::build_llm(&self.config).await?;
        let max_turns = self.config.max_turns;
        let read_source_limit = self.config.read_source_limit_bytes;

        let parallelism = self.config.parallelism.max(1);
        let sem = Arc::new(Semaphore::new(parallelism));
        let mut tasks = JoinSet::new();

        for assignment in assignments {
            let permit = sem.clone().acquire_owned().await?;
            let llm = llm.clone();
            let verbose = self.verbose;

            tasks.spawn(async move {
                let _permit = permit;
                run_one_assignment(assignment, llm, max_turns, read_source_limit, verbose).await
            });
        }

        let mut reports = Vec::new();
        while let Some(joined) = tasks.join_next().await {
            match joined {
                Ok(report) => reports.push(report),
                Err(e) => reports.push(AlignReport::from_parts(
                    applied,
                    vec![],
                    vec![crate::error_detail::error_detail(
                        "align agent task panicked",
                        &e,
                    )],
                )),
            }
        }

        Ok(AlignReport::merge(applied, reports))
    }
}

async fn run_one_assignment(
    assignment: AlignAssignment,
    llm: Arc<dyn Llm>,
    max_turns: u32,
    read_source_limit_bytes: usize,
    verbose: bool,
) -> AlignReport {
    let assignment_id = assignment.id.clone();
    let candidate_count = assignment.candidates.len();
    let apply = assignment.apply;

    if verbose {
        eprintln!(
            "  [align agent {}] starting: {} candidates",
            assignment_id, candidate_count
        );
    }

    let tools_concrete = Arc::new(AlignToolSet::with_limits(
        assignment.clone(),
        read_source_limit_bytes,
    ));
    let tools_for_agent: Arc<dyn oharness_tools::ToolSet> = tools_concrete.clone();

    let system = build_system_prompt(&assignment);
    let initial = build_initial_user_message(&assignment);

    let agent = match Agent::builder()
        .with_llm(llm)
        .with_tools(tools_for_agent)
        .with_event_sink(Arc::new(crate::terminal_events::TerminalEventSink::new(
            assignment_id.clone(),
        )))
        .with_loop(Box::new(ReactLoop::new().with_system_prompt(system)))
        .with_max_turns(max_turns)
        .build()
    {
        Ok(agent) => agent,
        Err(e) => {
            return AlignReport::from_parts(
                apply,
                vec![],
                vec![crate::error_detail::error_detail(
                    &format!("agent build for {assignment_id}"),
                    &e,
                )],
            );
        }
    };

    let task = Task::new(initial).with_id(assignment_id.clone());
    let result = agent.run(task).await;
    let usage_snapshot = tools_concrete.usage();
    let mut errors = Vec::new();

    match result {
        Ok(outcome) => {
            if let Some(error) = crate::error_detail::termination_error_detail(&outcome.termination)
            {
                errors.push(error);
            }
            if verbose {
                eprintln!(
                    "  [align agent {}] finished: {} changes, {} turns",
                    assignment_id,
                    usage_snapshot.changes.len(),
                    outcome.usage.turns
                );
            }
        }
        Err(e) => {
            errors.push(crate::error_detail::error_detail(
                &format!("agent loop failed for {assignment_id}"),
                &e,
            ));
            if verbose {
                eprintln!("  [align agent {}] errored: {}", assignment_id, e);
            }
        }
    }

    AlignReport::from_parts(apply, usage_snapshot.changes, errors)
}

fn build_system_prompt(assignment: &AlignAssignment) -> String {
    let mut prompt = match assignment.mode {
        AlignMode::Align => String::from(
            "You are a spec-alignment agent for the ought behavioral test framework.\n\n\
             Your job: inspect existing `.ought.md` specs that already map to source code, \
             compare them with the mapped code and generated tests where relevant, and report \
             where the current specs no longer reflect the codebase. This mode is report-only: \
             never ask to write files.\n\n",
        ),
        AlignMode::Discover => String::from(
            "You are a spec-discovery agent for the ought behavioral test framework.\n\n\
             Your job: inspect source code that appears uncovered by existing `.ought.md` specs, \
             use the existing specs as context for style and boundaries, and report possible new \
             specs to add. Code is the source of truth for this command, but specs are changed \
             only when apply mode is active.\n\n",
        ),
    };

    prompt.push_str("## Workflow\n\n");
    match assignment.mode {
        AlignMode::Align => prompt.push_str(
            "1. Call get_assignment to inspect candidate update/remove work.\n\
             2. For each candidate, call read_spec on target_spec_path and read the mapped \
             source files. If a mapped source entry is a directory, call list_source_files and \
             read the relevant files inside it. Read generated tests too when they clarify \
             spec/code drift.\n\
             3. If no drift is found after review, do not call propose_change for that \
             candidate.\n\
             4. For each real update/remove finding, call propose_change exactly once with \
             summary, rationale, confidence, and source file evidence. proposed_content is \
             optional in report-only align mode.\n\
             5. Call report_progress when done.\n\n",
        ),
        AlignMode::Discover => prompt.push_str(
            "1. Call get_assignment to inspect candidate add work.\n\
             2. For each candidate, read the relevant source files and read existing specs \
             when they clarify project language or nearby behavior.\n\
             3. If the behavior is already covered by an existing spec or is not worth a \
             behavioral spec, do not call propose_change for that candidate.\n\
             4. For each real missing spec, draft a complete `.ought.md` file as \
             proposed_content, call validate_spec, then call propose_change exactly once.\n\
             5. Call report_progress when done.\n\n",
        ),
    }

    prompt.push_str("## Change kinds\n\n");
    prompt.push_str(
        "- add: source behavior exists without a matching spec. Provide a complete new \
         `.ought.md` file as proposed_content.\n\
         - update: an existing spec still maps to code, but the code behavior has drifted.\n\
         - remove: an existing spec or clause no longer appears supported by code. Never \
         delete it; report the unsupported behavior clearly.\n\n",
    );

    if let Some(only) = assignment.only {
        prompt.push_str(&format!(
            "This run is restricted to `{}` changes. Ignore other candidate kinds.\n\n",
            only
        ));
    }
    if let Some(focus) = assignment
        .focus
        .as_deref()
        .filter(|focus| !focus.trim().is_empty())
    {
        let focus_json = serde_json::to_string(focus.trim()).unwrap_or_else(|_| "\"\"".into());
        prompt.push_str("## User focus\n\n");
        prompt.push_str(&format!(
            "The user asked discovery to focus on this area: {}.\n\
             Treat this as a hard discovery boundary. You may read nearby code or existing specs \
             to understand the area, but only propose missing specs for behavior inside this \
             focus. Do not propose changes outside this area.\n\n",
            focus_json
        ));
    }
    if assignment.apply {
        match assignment.mode {
            AlignMode::Align => prompt.push_str(
                "APPLY MODE is unexpectedly active for align mode. Continue to validate proposed \
                 content before propose_change, but only report findings unless the assignment \
                 explicitly contains apply-safe candidates.\n\n",
            ),
            AlignMode::Discover => prompt.push_str(
                "APPLY MODE is active: propose_change will write new spec content for add \
                 findings. Validate proposed content first.\n\n",
            ),
        }
    } else {
        prompt.push_str(
            "REPORT MODE is active: propose_change records structured report entries only. \
             It must not write to disk.\n\n",
        );
    }

    prompt.push_str("## Spec style\n\n");
    prompt.push_str(
        "- Keep specs focused on observable behavior, not private helper mechanics.\n\
         - Preserve existing human wording where it still matches code.\n\
         - Prefer fewer, sharper clauses over vague coverage lists.\n\
         - Use `source:` metadata so future alignment can map specs back to code.\n\
         - If behavior exists but its product intent is uncertain, mark it \
         `**PENDING MUST**` rather than overstating certainty.\n\n",
    );

    prompt.push_str(&format!(
        "## Output paths\n\nAll target_spec values are relative to specs_root: `{}`.\n\n",
        assignment.specs_root
    ));

    prompt.push_str("## Grammar reference\n\n");
    prompt.push_str("```text\n");
    prompt.push_str(GRAMMAR_MD);
    prompt.push_str("\n```\n");

    prompt
}

fn build_initial_user_message(assignment: &AlignAssignment) -> String {
    let label = match assignment.mode {
        AlignMode::Align => "alignment",
        AlignMode::Discover => "discovery",
    };
    let mut message = format!(
        "Begin {} assignment {}. Call get_assignment first, then review all candidates.",
        label, assignment.id
    );
    if let Some(focus) = assignment
        .focus
        .as_deref()
        .filter(|focus| !focus.trim().is_empty())
    {
        message.push_str(&format!(
            " User focus: {}. Do not propose work outside this focus.",
            focus.trim()
        ));
    }
    message
}
