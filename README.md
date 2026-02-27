# Lisa Loop

A methodology and toolbox for solving complex engineering and research problems with AI agents, grounded in peer-reviewed literature, with explicit verification, validation, and convergence tracking.

Lisa Loop fuses two established engineering paradigms:

- **The V-Model** (systems engineering): every level of specification is paired with a corresponding level of verification and validation. V&V criteria are defined *before* implementation, not after.
- **The Design Spiral** (Evans, 1959): the same problem is revisited iteratively at increasing fidelity and scope until the answer converges.

## Three Absolute Rules

1. **Every methodological choice must trace to a peer-reviewed source.** No equation without a paper. No method without a citation.
2. **Engineering judgment is a first-class, auditable artifact.** "Do these numbers make physical sense?" is always asked, and the checks are written down, versioned, and executed.
3. **The spiral history is the deliverable, not just the answer.** Every methodological choice, every refinement, every convergence step is preserved as a complete record.

## Quick Start

1. Install the `lisa` CLI:

```bash
cargo install --path .
```

2. Initialize a new project:

```bash
lisa init resolve-assignment
```

3. Edit `.lisa/BRIEF.md` with your project description
4. Add reference papers to `.lisa/references/core/`
5. Run:

```bash
# Run the full spiral — scoping through convergence
lisa run

# Or step by step:
lisa scope                   # Pass 0: scoping only
lisa run --max-passes 3      # Limit spiral passes
lisa resume                  # Resume from where you left off
lisa status                  # Check current state
lisa doctor                  # Check environment and prerequisites
lisa finalize                # Produce final deliverables
lisa eject-prompts           # Copy prompts to .lisa/prompts/ for customization
```

## Architecture

```
┌──────────────────────────────────────────────────────────────────────────┐
│                    PASS 0: SCOPE (with human refinement loop)             │
│                                                                          │
│   ┌────────────────┐     ┌────────────────┐     ┌────────────────┐      │
│   │  SCOPE AGENT    │────→│  HUMAN REVIEW   │────→│  APPROVED       │     │
│   │  methodology,   │     │  [A]pprove      │     │  proceed to     │     │
│   │  acceptance,    │  ┌──│  [R]efine       │     │  Pass 1         │     │
│   │  spiral plan    │  │  │  [E]dit         │     └────────┬───────┘     │
│   └────────────────┘  │  └────────────────┘               │             │
│          ↑            │                                    │             │
│          └── feedback ┘                                    │             │
└────────────────────────────────────────────────────────────┼─────────────┘
                                                             ↓
┌──────────────────────────────────────────────────────────────────────────┐
│                         OUTER LOOP: SPIRAL                               │
│                   (convergence-driven, human-gated)                      │
│                                                                          │
│   ┌────────────────┐                                                     │
│   │    REFINE       │  opus + subagents                                  │
│   │    methodology  │  (literature search, code audit, validation review)│
│   │    + plan       │  reads spiral-plan.md for scope progression        │
│   └───────┬────────┘                                                     │
│           ↓                                                              │
│   ┌────────────────┐                                                     │
│   │   DDV RED       │  opus (independent of implementation)              │
│   │   write failing │  tests only for current pass's scope subset        │
│   │   domain tests  │                                                    │
│   └───────┬────────┘                                                     │
│           ↓                                                              │
│   ┌────────────────┐                                                     │
│   │   BUILD         │  sonnet (Ralph loop)                               │
│   │   make tests    │  implement → DDV green → software tests            │
│   │   green         │                                                    │
│   └───────┬────────┘                                                     │
│           ↓                                                              │
│   ┌────────────────┐                                                     │
│   │   EXECUTE       │  opus                                              │
│   │   assemble +    │  integration code → run → engineering judgment     │
│   │   run + audit   │                                                    │
│   └───────┬────────┘                                                     │
│           ↓                                                              │
│   ┌────────────────┐                                                     │
│   │   VALIDATE      │  opus                                              │
│   │   V&V +         │  sanity checks → convergence → recommendation     │
│   │   convergence   │                                                    │
│   └───────┬────────┘                                                     │
│           ↓                                                              │
│   ┌───────────────────────────────────────────┐                          │
│   │           HUMAN REVIEW GATE               │                          │
│   │  Accept / Continue / Redirect             │                          │
│   └───────────────────────┬───────────────────┘                          │
│                           ↓                                              │
│                    (next spiral pass)                                     │
└──────────────────────────────────────────────────────────────────────────┘
```

## Domain-Driven Verification (DDV)

DDV is the core verification mechanism, using two-agent separation to prevent correlated domain knowledge errors:

1. **DDV Red** (opus): Writes failing tests from authoritative sources (papers, standards, analytical solutions). Does NOT read implementation code.
2. **Build** (sonnet): Implements code to make those tests pass. CANNOT modify DDV tests. Disagreements are documented and adjudicated by the next refine phase.

**DDV is domain-agnostic.** The pattern works for any domain with authoritative sources and testable expected values: physics papers, econometrics studies, regulatory standards, engineering benchmarks.

**Engineering judgment** is a named rigor standard: dimensional analysis, conservation law checks, order-of-magnitude estimation from first principles, and hard bounds. This standard applies regardless of domain.

## Scope Progression

The spiral plan stages both fidelity AND scope per pass. Early passes test the methodology on a subset of the full problem:

| Pass | Pattern | Key question |
|------|---------|--------------|
| 1 | Narrow scope, simplest method, wide tolerance | Does the approach work at all? |
| 2 | Broader scope, add corrections | Does it generalize? |
| 3 | Full scope, moderate fidelity | Does coupling work? |
| 4 | Full scope, refined methods | Converged? |

## Subagent Usage

The refine phase uses Claude Code's Task tool to delegate focused research tasks:
- **Literature subagent:** Search for methods, evaluate alternatives
- **Code audit subagent:** Audit existing code structure and interfaces
- **Validation review subagent:** Summarize previous pass results

This manages context without architectural complexity. The build phase stays single-agent.

## Pass 0 — Scoping (with Human Refinement Loop)

The only non-repeating pass. Establishes methodology, acceptance criteria, validation strategy, and scope progression. **No code is written.**

The human can iteratively refine scope artifacts before any code exists:
- **Approve** — proceed to Pass 1
- **Refine** — provide feedback, scope agent re-runs with corrections
- **Edit** — modify files directly, then approve
- **Quit** — stop

## Pass N — Five-Phase Spiral

| Phase | Agent | Writes code? | Key question |
|-------|-------|-------------|--------------|
| Refine | opus + subagents | No | What methodology, what plan? |
| DDV Red | opus | Tests only | What should correct results look like? |
| Build | sonnet (Ralph loop) | Yes | Make the red tests green + software quality |
| Execute | opus | Runner + integration tests | Does the system produce an answer? |
| Validate | opus | No | Does the answer converge? |

## Human Interaction

### Pass Review Gate

```
═══════════════════════════════════════════════════════
  SPIRAL PASS N COMPLETE — REVIEW REQUIRED
═══════════════════════════════════════════════════════

  Answer:      142.3 kN total resistance
  Convergence: NOT YET (Δ 12% from prev)
  Tests:       DDV: 8/8 | Software: 15/15 | Integration: 2/2
  Agent recommends: CONTINUE

  [A] ACCEPT — converged, produce final report
  [C] CONTINUE — next spiral pass
  [R] REDIRECT — provide guidance (opens $EDITOR)
```

### Block Gate (During Build)

When tasks are blocked (methodology issues, DDV disagreements), the block gate shows specific task names and reasons, with options to Fix, Skip, or Abort.

## Final Deliverables

When the human accepts:

1. **`.lisa/output/answer.md`** — Direct response to the question in BRIEF.md
2. **`.lisa/output/report.md`** — Full development report: problem statement, methodology with citations, spiral history, V&V summaries, convergence tables, assumptions, limitations, traceability

## Traceability Chain

```
.lisa/BRIEF.md → acceptance criteria
  → scope (.lisa/spiral/pass-0/) — human-refined scope, fidelity progression, validation strategy
    → methodology (.lisa/methodology/methodology.md)
      → authoritative domain source (.lisa/references/)
        → governing equations
          → DDV tests (tests/ddv/) — domain specification as executable tests
            → implementation (src/)
              → derivations (.lisa/methodology/derivations/) — non-trivial mappings only
                → software tests (tests/software/) — edge cases, stability
                  → execution (src/runner) — end-to-end, engineering judgment audit
                    → system validation — sanity checks, convergence
                      → human acceptance
                        → final answer + report
```

## Configuration

All configuration is in `.lisa/lisa.toml`:

```toml
[project]
name = "my-project"

[models]
scope = "opus"
refine = "opus"
ddv = "opus"
build = "sonnet"
execute = "opus"
validate = "opus"

[limits]
max_spiral_passes = 5
max_ralph_iterations = 50

[review]
# Human review gates. When false, loop runs fully autonomously.
pause = true

[git]
auto_commit = true
auto_push = false

[terminal]
# Collapse agent streaming output to summary lines after completion
collapse_output = true

[paths]
# Where process artifacts live (relative to project root)
lisa_root = ".lisa"

# Where deliverable code goes (relative to project root)
source = ["src"]

# Test directories (relative to project root)
tests_ddv = "tests/ddv"
tests_software = "tests/software"
tests_integration = "tests/integration"

[commands]
# These get populated by the scope agent, but can be pre-filled
setup = ""
build = ""
test_all = ""
test_ddv = ""
test_software = ""
test_integration = ""
lint = ""
```

## Prompt Customization

Prompts are compiled into the binary by default. To customize them:

```bash
lisa eject-prompts
```

This copies all prompts to `.lisa/prompts/`. Edit them freely — the CLI uses local prompts when present, falling back to the compiled-in defaults.

## Directory Structure

```
project-root/
├── Cargo.toml                          # Rust project configuration
├── src/                                # Lisa Loop CLI source code
│
├── .lisa/                              # Process artifacts root
│   ├── lisa.toml                       # Configuration
│   ├── BRIEF.md                        # Project description (user writes)
│   ├── AGENTS.md                       # Build/test/plot commands
│   │
│   ├── methodology/
│   │   ├── methodology.md              # Single methodology document
│   │   ├── plan.md                     # Single implementation plan
│   │   ├── verification-cases.md       # L0/L1 test specifications
│   │   ├── overview.md                 # System description
│   │   ├── assumptions-register.md     # Cross-cutting assumptions
│   │   └── derivations/               # Code ↔ equations mapping
│   │
│   ├── spiral/
│   │   ├── state.toml                  # Machine-readable spiral state
│   │   ├── pass-0/
│   │   │   ├── acceptance-criteria.md
│   │   │   ├── validation-strategy.md
│   │   │   ├── sanity-checks.md
│   │   │   ├── literature-survey.md
│   │   │   ├── spiral-plan.md          # Scope + fidelity progression
│   │   │   └── PASS_COMPLETE.md
│   │   └── pass-N/
│   │       ├── refine-summary.md
│   │       ├── ddv-red-manifest.md
│   │       ├── execution-report.md
│   │       ├── system-validation.md
│   │       ├── convergence.md
│   │       ├── review-package.md
│   │       ├── reconsiderations/
│   │       └── PASS_COMPLETE.md
│   │
│   ├── validation/
│   │   ├── sanity-checks.md
│   │   ├── limiting-cases.md
│   │   ├── reference-data.md
│   │   └── convergence-log.md
│   │
│   ├── references/
│   │   ├── core/
│   │   └── retrieved/
│   │
│   ├── plots/
│   │   └── REVIEW.md
│   │
│   └── output/
│       ├── answer.md
│       └── report.md
│
├── src/                                # Deliverable implementation code
├── tests/
│   ├── ddv/                            # Domain-Driven Verification tests
│   ├── software/                       # Software quality tests
│   └── integration/                    # End-to-end tests
│
├── prompts/                            # Compiled-in prompt templates
└── templates/                          # Compiled-in init templates
```

## Lineage

Lisa Loop extends the [Ralph Wiggum technique](https://ghuntley.com/ralph/) created by [Geoffrey Huntley](https://github.com/ghuntley/how-to-ralph-wiggum) — a bash loop that feeds prompts to an AI agent with filesystem persistence as shared state.

**Lisa Loop v1** added methodology rigor, hierarchical verification, and a reconsideration protocol for engineering/scientific software where "passing tests" is necessary but insufficient — the tests themselves might encode wrong physics.

**Lisa Loop v2** restructured the process into a five-phase spiral architecture with Domain-Driven Verification (DDV).

**Lisa Loop v3** (current) reimplements the orchestrator as a Rust CLI (`lisa`), replacing the bash script with a compiled binary that embeds prompts and templates, uses TOML configuration, and provides enum-based state management with structured serialization.

Named after Lisa Simpson — the rigorous counterpart to Ralph Wiggum.
