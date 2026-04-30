# ought-analysis

Failure diagnostics over specs, run results, and git history.

Where `ought-gen` writes tests and `ought-run` executes them, this crate
reasons about *why* things look the way they do — the likely cause of a
regression and related diagnostics.

## Responsibilities

- Correlate a failing clause with git history and produce a narrative
  explanation (`blame`).
- Binary-search git history to find the breaking commit for a clause
  (`bisect`).

## Notable public API

- `blame::blame(&clause_id, &specs, &run_result) -> BlameResult` — timeline,
  likely commit, and LLM narrative for a failing clause.
- `bisect::bisect(&clause_id, &specs, runner, &BisectOptions) -> BisectResult`
  — identifies the `CommitInfo` where a clause started failing.
