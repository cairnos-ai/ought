# Diagnostics

context: The diagnostic commands go beyond basic test generation and execution. They may use LLMs when they need causal explanation. Source/spec drift reporting belongs to `ought align`; missing-spec discovery belongs to `ought discover`.

source: src/analysis/

requires: [parser](../engine/parser.ought.md), [generator](../engine/generator.ought.md), [runner](../engine/runner.ought.md)

## Blame

`ought debug blame <clause>` — explains why a clause is failing by correlating with source changes.

- **MUST** accept a clause identifier (e.g. `auth::login::must_return_401`)
- **MUST** retrieve the clause, its generated test, and the failure output
- **MUST** use git history to find when the clause last passed and what changed since
- **MUST** use the LLM to correlate the source diff with the failure and produce a causal explanation
- **MUST** output the timeline: last passing run, first failure, relevant commits
- **MUST** output a narrative explanation of what broke and why
- **SHOULD** identify the specific commit and file change most likely responsible
- **SHOULD** name the author of the likely-responsible commit
- **SHOULD** suggest a fix when the cause is clear
- **MUST NOT** require a running LLM if the clause has never passed (just report "never passed")

## Bisect

`ought debug bisect <clause>` — automated binary search through git history to find the breaking commit.

- **MUST** accept a clause identifier
- **MUST** perform a git-bisect-style binary search: checkout commit, generate test for clause, run it, narrow range
- **MUST** report the first commit where the clause fails
- **MUST** show the commit message, author, date, and diff summary for the breaking commit
- **MUST ALWAYS** restore the working tree to its original state after completion (never leave on detached HEAD)
- **SHOULD** use the generated test from the current manifest (not regenerate per commit) unless `--regenerate` is passed
- **SHOULD** cache test results per commit to avoid redundant runs
- **SHOULD** support `--range <from>..<to>` to limit the search space
- **GIVEN** the bisect is interrupted (SIGINT, crash):
  - **MUST** restore the working tree to the original branch
  - **SHOULD** save progress so `ought debug bisect --continue` can resume
