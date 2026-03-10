# Lisa Loop

A CLI tool that orchestrates AI agents through rigorous engineering problem-solving, combining the V-Model (verification before implementation) with the Design Spiral (iterative refinement until human acceptance).

## How It Works

```
  lisa init → scaffold .lisa/ + ASSIGNMENT.md
                    │
                    ▼
  ┌───────────────────────────────────────┐
  │  Pass 0: SCOPE                        │
  │  methodology, acceptance, spiral plan │◄── human refine loop
  └──────────────────┬────────────────────┘
                     ▼
  ┌───────────────────────────────────────┐
  │  DDV Agent (one-time prologue)        │
  │  literature → verification scenarios  │
  │  (markdown, no code)                  │
  └──────────────────┬────────────────────┘
                     ▼
  ┌───────────────────────────────────────┐
  │  Pass 1..N: SPIRAL                    │
  │                                       │
  │  ┌─────────┐  ┌─────────┐  ┌───────┐ │
  │  │ REFINE  │→ │ BUILD   │→ │VALIDATE│ │
  │  │ (opus)  │  │(sonnet) │  │ (opus) │ │
  │  └─────────┘  └─────────┘  └───────┘ │
  │                                       │◄── human review gate
  └──────────────────┬────────────────────┘
                     ▼
  lisa finalize → answer.md + report.md
```

Each pass increases fidelity and scope. The human reviews after every pass and decides: **accept**, **continue** to the next spiral pass, or **redirect** with guidance.

## Domain-Driven Verification (DDV)

DDV prevents correlated errors by separating *who writes scenarios* from *who writes code* from *who writes tests*:

```
  DDV Agent (opus)          Build (sonnet)          Validate (opus)
  ┌───────────────┐        ┌───────────────┐       ┌───────────────┐
  │ Read papers    │        │ Read scenarios │       │ Read scenarios │
  │ Write scenarios│───────→│ Write code     │──────→│ Write tests    │
  │ (no code)      │        │ (no DDV tests) │       │ Run all tests  │
  └───────────────┘        └───────────────┘       └───────────────┘
    ▲ independent              ▲ separated              ▲ verified
```

Works for any domain with authoritative sources and testable expected values: physics, econometrics, regulatory standards, engineering benchmarks.

## Three Absolute Rules

1. **Every methodological choice must trace to a peer-reviewed source.** No equation without a paper.
2. **Engineering judgment is a first-class, auditable artifact.** Sanity checks are written down, versioned, and executed.
3. **The spiral history is the deliverable, not just the answer.** Every refinement is preserved as a complete record.

## Getting Started

### Prerequisites

- [Claude Code](https://docs.anthropic.com/en/docs/claude-code) installed and authenticated
- Git configured with `user.name` and `user.email`

### Install

Download the latest release binary (Linux x86_64):

```bash
curl -fsSL https://github.com/freol35241/lisa-loop/releases/latest/download/lisa-linux-x86_64 -o lisa && chmod +x lisa && sudo mv lisa /usr/local/bin/
```

Or build from source:

```bash
cargo install --path .
```

### Usage

```bash
lisa doctor                  # Check prerequisites
lisa init                    # Scaffold .lisa/ and ASSIGNMENT.md
# Edit ASSIGNMENT.md with your problem description
# Add reference papers to .lisa/references/core/
lisa run                     # Run the full spiral
```

## Commands

```bash
lisa run                     # Full spiral: scope → DDV → passes → finalize
lisa run --max-passes 3      # Limit spiral passes
lisa scope                   # Pass 0 only (scoping)
lisa resume                  # Resume from saved state
lisa status                  # Print current spiral state
lisa history                 # Pass-by-pass history
lisa rollback <pass>         # Roll back to a pass boundary
lisa continue "<question>"   # Follow-up question after acceptance
lisa finalize                # Produce answer.md + report.md
lisa eject-prompts           # Copy prompts to .lisa/prompts/ for customization
lisa doctor                  # Check environment
```

Configuration lives in `.lisa/lisa.toml` (models, limits, review gates, paths, commands). Run `lisa init` to see the full default config with comments.

## Human Interaction

### Pass Review Gate

```
═══════════════════════════════════════════════════════
  SPIRAL PASS N COMPLETE — REVIEW REQUIRED
═══════════════════════════════════════════════════════

  Answer:      142.3 kN total resistance
  Progress:    Δ 12% from prev
  Tests:       DDV: 8/8 | Software: 15/15 | Integration: 2/2
  Agent recommends: CONTINUE

  [A] ACCEPT — produce final report
  [C] CONTINUE — next spiral pass
  [R] REDIRECT — provide guidance
```

### Scope Review Gate

After Pass 0, review methodology and acceptance criteria before any code is written. Options: **Approve**, **Refine** (agent re-runs with feedback), **Edit** (modify files directly), or **Quit**.

## Traceability Chain

```
ASSIGNMENT.md → acceptance criteria
  → scope (.lisa/spiral/pass-0/)
    → methodology (.lisa/methodology/methodology.md)
      → authoritative source (.lisa/references/)
        → governing equations
          → DDV tests (tests/ddv/)
            → implementation (src/)
              → software tests (tests/software/)
                → integration tests (tests/integration/)
                  → system validation
                    → human acceptance
                      → final answer + report
```

## Lineage

Lisa Loop extends the [Ralph Wiggum technique](https://ghuntley.com/ralph/) created by [Geoffrey Huntley](https://github.com/ghuntley/how-to-ralph-wiggum) — a bash loop that feeds prompts to an AI agent with filesystem persistence as shared state.

**v1** added methodology rigor, hierarchical verification, and a reconsideration protocol for engineering/scientific software where "passing tests" is necessary but insufficient — the tests themselves might encode wrong physics.

**v2** restructured the process into a three-phase spiral architecture with Domain-Driven Verification (DDV).

**v3** (current) reimplements the orchestrator as a Rust CLI (`lisa`), replacing the bash script with a compiled binary that embeds prompts and templates, uses TOML configuration, and provides enum-based state management with structured serialization.

Named after Lisa Simpson — the rigorous counterpart to Ralph Wiggum.
