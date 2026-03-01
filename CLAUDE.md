# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
cargo build              # Build the project
cargo test               # Run all 18 unit tests
cargo test <test_name>   # Run a single test (e.g., cargo test test_parse_tasks)
cargo test <mod>::tests  # Run all tests in a module (e.g., cargo test config::tests)
cargo clippy             # Lint
cargo fmt                # Format
cargo install --path .   # Install the `lisa` binary
```

## What This Project Is

Lisa Loop is a CLI tool (`lisa`) that orchestrates AI agents (Claude CLI) through a rigorous engineering problem-solving methodology combining the V-Model and Design Spiral. It does NOT do engineering work itself — it manages prompt construction, state transitions, DDV isolation enforcement, git commits, and human review gates across iterative spiral passes.

## Architecture

### Core Flow

1. **`lisa init`** scaffolds a `.lisa/` directory with config, templates, and state file
2. **`lisa run`** drives a spiral: Pass 0 (Scoping) then Passes 1..N (Refine → DDV Red → Build → Execute → Validate)
3. Each phase: save state → build prompt → spawn `claude` subprocess → parse NDJSON output → git commit → review gate
4. **Filesystem is shared state** — agents have no memory between invocations; all inter-invocation communication happens through markdown files in `.lisa/`

### Module Map

- **`cli.rs`** — Clap-derived CLI definition (`Commands` enum)
- **`main.rs`** — Entry point, dispatches commands
- **`orchestrator.rs`** — Core spiral logic: `run()`, `resume()`, per-phase runners (`run_scope`, `run_refine`, `run_ddv_red`, `run_build_loop`, `run_execute`, `run_validate`, `finalize`)
- **`state.rs`** — `SpiralState`/`PassPhase` enum state machine, TOML serialized to `.lisa/state.toml`
- **`agent.rs`** — Spawns `claude` CLI subprocess, pipes prompts to stdin, parses streaming NDJSON, tracks tool calls, renders progress UX with elapsed time ticker thread
- **`prompt.rs`** — Loads prompts (local `.lisa/prompts/` overrides compiled-in defaults), renders `{{placeholder}}` substitutions, assembles context preamble
- **`config.rs`** — TOML config from `.lisa/lisa.toml` (project, models, limits, review, git, paths, commands)
- **`review.rs`** — Three interactive gate types: scope review (A/R/E/Q), pass review (A/C/R), block gate (F/S/X)
- **`enforcement.rs`** — Post-hoc DDV isolation: verifies DDV Red agent didn't touch source files (tool log inspection), verifies Build agent didn't modify DDV test files (git revert if so)
- **`tasks.rs`** — Regex parser for `### Task N` blocks in `plan.md` (status tracking, stall detection)
- **`git.rs`** — Git commit/push/reset operations respecting config flags
- **`terminal.rs`** — Colored output utilities via crossterm
- **`init/scaffold.rs`** — Scaffolds `.lisa/` directory tree with embedded templates

### Key Design Patterns

- **Compiled-in resources**: Prompts (`prompts/`) and templates (`templates/`) are embedded via `include_str!`. Users can eject prompts with `lisa eject-prompts` for customization without recompiling.
- **Two-agent DDV separation**: DDV Red (opus) writes failing tests without seeing source. Build (sonnet) implements code without modifying DDV tests. Enforcement is post-hoc via tool log and git diff — violations are hard errors.
- **Build "Ralph loop"**: The build phase iterates up to `max_ralph_iterations`, with stall detection based on content hashing of `plan.md`. Stalls trigger a block gate.
- **Model assignment**: Most phases use "opus"; Build uses "sonnet". Configurable in `lisa.toml`.
- **Error handling**: `anyhow::Result` throughout, propagated with `?`.
- **No async**: Fully synchronous. Only threading is the elapsed-time ticker in agent.rs.

### Important Non-Obvious Details

- The `tests/` directory at the repo root is NOT for lisa-loop's own tests — it contains `.gitkeep` scaffolding that `lisa init` creates for target projects. Unit tests are inline in each module under `#[cfg(test)]`.
- The Claude CLI is invoked with `--dangerously-skip-permissions`, giving agents unrestricted filesystem access.
- Modifying files in `prompts/` or `templates/` requires recompilation since they're embedded with `include_str!`.
