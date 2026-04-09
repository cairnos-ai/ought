# Ought

Behavioral test framework: specs in `.ought.md`, LLM-generated tests.

## Key references

- **Grammar**: `docs/grammar.md` is the source of truth for the `.ought.md` spec grammar. The parser in `crates/ought-spec/src/parser.rs` must conform to this grammar. When changing parsing behavior, update the grammar first.
- **Design**: `docs/design.md` for architecture and philosophy.
- **Specs as requirements**: the specs in `ought/` ARE the project's own requirements. Use `ought check` to validate.

## Build & test

The workspace contains both Rust (cargo) and TypeScript (the Svelte UI in
`crates/ought-server/ui/`). The `justfile` at the repo root orchestrates both;
prefer it over raw `cargo` / `npm`, since the rust-embed macro in `ought-server`
needs the Vite `dist/` directory to exist before the workspace will compile.

```
just build       # build everything (UI + Rust); pass `release` for an optimized build
just test        # run all tests
just lint        # lint UI (svelte-check) + Rust (clippy)
just ci          # full CI pipeline (test + lint)
just install     # build a release binary and install ought to ~/.local/bin
just --list      # list all recipes (grouped: all / rust / typescript)
```

Lint enforcement is workspace-wide via `[workspace.lints.rust] warnings = "deny"`
and `[workspace.lints.clippy] all = { level = "deny", priority = -1 }` in the
root `Cargo.toml`, so `cargo build` / `cargo test` will also fail on warnings.

## Workspace layout

```
crates/
  ought-spec/        # parser + clause IR (the open standard, zero LLM deps)
  ought-gen/         # generator trait + providers
  ought-run/         # runner trait + language runners
  ought-report/      # reporter + TUI
  ought-analysis/    # survey, audit, blame, bisect
  ought-mcp/         # MCP server
  ought-server/      # viewer web UI (Svelte + shadcn)
  ought-cli/         # CLI binary
```
