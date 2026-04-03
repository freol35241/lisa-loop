# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
cargo build              # Build the project (dev, dynamically linked)
cargo test               # Run all unit tests
cargo test <test_name>   # Run a single test (e.g., cargo test test_parse_tasks)
cargo test <mod>::tests  # Run all tests in a module (e.g., cargo test config::tests)
cargo clippy             # Lint
cargo fmt                # Format
cargo install --path .   # Install the `lisa` binary

# Portable release build (statically linked, no glibc dependency):
rustup target add x86_64-unknown-linux-musl
sudo apt-get install -y musl-tools
cargo build --release --target x86_64-unknown-linux-musl
# Binary: target/x86_64-unknown-linux-musl/release/lisa
```

## What This Project Is

Lisa Loop is a CLI tool (`lisa`) that orchestrates AI agents (Claude CLI) through a rigorous engineering problem-solving methodology combining the V-Model and Design Spiral. It does NOT do engineering work itself — it manages prompt construction, state transitions, git commits, and human review gates across iterative spiral passes.

## Architecture

### Core Flow

1. **`lisa init`** scaffolds a `.lisa/` directory with config, templates, and state file
2. **`lisa run`** drives a spiral: Pass 0 (Research → [human gate] → Validation Design → Planning → [human gate]) → Passes 1..N (Refine → per-task [Bounds → Build] → Audit) → Finalize
3. Each phase: save state → build prompt → spawn `claude` subprocess → parse NDJSON output → git commit → review gate
4. **Filesystem is shared state** — agents have no memory between invocations; all inter-invocation communication happens through markdown files in `.lisa/`

### Module Map

- **`cli.rs`** — Clap-derived CLI definition (`Commands` enum)
- **`main.rs`** — Entry point, dispatches commands
- **`orchestrator.rs`** — Core spiral logic: `run()`, `resume()`, per-phase runners (`run_scope` [decomposes into `run_research`, `run_validation_design`, `run_planning`], `run_refine`, `run_build_loop` [per-task `run_bounds` + build], `run_audit`, `run_explore`, `finalize`)
- **`state.rs`** — `SpiralState`/`PassPhase` enum state machine, TOML serialized to `.lisa/state.toml`
- **`agent.rs`** — Spawns `claude` CLI subprocess, pipes prompts to stdin, parses streaming NDJSON, tracks tool calls, renders progress UX with elapsed time ticker thread
- **`prompt.rs`** — Loads prompts (local `.lisa/prompts/` overrides compiled-in defaults), renders `{{placeholder}}` substitutions, assembles context preamble
- **`config.rs`** — TOML config from `lisa.toml` in project root (project, models, limits, review, git, paths, commands)
- **`review.rs`** — Nine interactive gate types: methodology review (A/R/E/Q), scope review (A/R/E/Q), refine review (A/R/E/Q), pass review (F/C/R/E/Q), explore review (M/D), block gate (F/S/X), finalize gate (A/R), budget gate (C/S), environment gate
- **`tasks.rs`** — Regex parser for `### Task N` blocks in `plan.md` (status tracking, dependency resolution, orchestrator-driven task selection, stall detection)
- **`git.rs`** — Git commit/push/reset operations respecting config flags
- **`terminal.rs`** — Colored output utilities via crossterm
- **`init/scaffold.rs`** — Scaffolds `.lisa/` directory tree with embedded templates

### Key Design Patterns

- **Compiled-in resources**: Prompts (`prompts/`) and templates (`templates/`) are embedded via `include_str!`. Users can eject prompts with `lisa eject-prompts` for customization without recompiling.
- **Decomposed scope**: Pass 0 is split into three focused agents: Research (methodology + criteria + stack), Validation Design (sanity checks + limiting cases + reference data), and Planning (spiral plan + task breakdown). A human reviews the methodology before validation/planning build on it.
- **Structural verification independence**: Each build task runs a Bounds agent (derives first-principles bounds from methodology, never sees implementation) followed by a Build agent (implements code to satisfy the bounds). Independence is enforced by the invocation boundary, not prompt instructions.
- **Orchestrator-driven task selection**: The orchestrator reads `plan.md`, resolves task dependencies, selects the next eligible task, and assigns it to agents. Agents no longer pick their own tasks.
- **Engineering judgment skills**: Skills (bounding tests, dimensional analysis, numerical stability, literature grounding) are embedded as markdown in `skills/` and injected into agent prompts. The Bounds agent uses engineering judgment for derivation; the Audit agent uses it for coverage auditing.
- **Build "Ralph loop"**: The build phase iterates up to `max_ralph_iterations`, with orchestrator-driven task selection and dual-signal stall detection (task status hash + source file changes). Stalls trigger a block gate.
- **Model assignment**: Scope/Research/Refine/Bounds use "opus"; Build uses "sonnet"; Audit uses "opus". Configurable in `lisa.toml`.
- **Error handling**: `anyhow::Result` throughout, propagated with `?`.
- **No async**: Fully synchronous. Only threading is the elapsed-time ticker in agent.rs.

### Important Non-Obvious Details

- The `tests/` directory at the repo root is NOT for lisa-loop's own tests — it contains `.gitkeep` scaffolding that `lisa init` creates for target projects. Unit tests are inline in each module under `#[cfg(test)]`.
- The Claude CLI is invoked with `--dangerously-skip-permissions`, giving agents unrestricted filesystem access.
- Modifying files in `prompts/` or `templates/` requires recompilation since they're embedded with `include_str!`.
