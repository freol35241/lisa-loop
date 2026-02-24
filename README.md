# Lisa Loop v2

A methodology and toolbox for solving complex engineering and research problems with AI agents, grounded in peer-reviewed literature, with explicit verification, validation, and convergence tracking.

Lisa Loop v2 fuses two established engineering paradigms:

- **The V-Model** (systems engineering): every level of decomposition is paired with a corresponding level of verification and validation. V&V criteria are defined *before* implementation, not after.
- **The Design Spiral** (Evans, 1959): the same subsystems are revisited iteratively at increasing fidelity until the design converges.

**The fusion:** each revolution of the spiral visits every subsystem in sequence, each doing a half-V (refine methodology → build → verify), then the system is validated as a whole. The spiral terminates when the system-level answer has converged, not when tasks are complete.

## Three Absolute Rules

1. **Every methodological choice must trace to a peer-reviewed source.** No equation without a paper. No method without a citation.
2. **Engineering judgment is a first-class, auditable artifact.** "Do these numbers make physical sense?" is always asked, and the checks are written down, versioned, and executed.
3. **The spiral history is the deliverable, not just the answer.** Every methodological choice, every refinement, every convergence step is preserved as a complete record.

## Quick Start

1. Click **"Use this template"** on GitHub to create your repo
2. Edit `BRIEF.md` with your project description
3. Add reference papers to `references/core/`
4. Edit `AGENTS.md` with your build/test commands
5. Run:

```bash
chmod +x loop.sh

# Run the full spiral — scoping through convergence
./loop.sh run

# Or step by step:
./loop.sh scope                  # Pass 0: scoping only
./loop.sh run --max-passes 3     # Limit spiral passes
./loop.sh resume                 # Resume from where you left off
./loop.sh status                 # Check current state
```

## Architecture

```
┌──────────────────────────────────────────────────────────────────────────┐
│                         OUTER LOOP: SPIRAL                               │
│                   (convergence-driven, human-gated)                      │
│                                                                          │
│   ┌─ For each subsystem (in dependency order): ──────────────────────┐   │
│   │                                                                  │   │
│   │   ┌────────────────┐    ┌──────────────────────────────────┐     │   │
│   │   │    REFINE      │    │         BUILD + VERIFY           │     │   │
│   │   │                │    │                                  │     │   │
│   │   │  methodology   │ →  │  ┌────────────────────────────┐  │     │   │
│   │   │  + plan for    │    │  │ RALPH LOOP                 │  │     │   │
│   │   │  THIS subsys   │    │  │                            │  │     │   │
│   │   │                │    │  │ pick task → implement →    │  │     │   │
│   │   │  [opus]        │    │  │ test (L0,L1) → next       │  │     │   │
│   │   │                │    │  │                            │  │     │   │
│   │   │                │    │  │ [sonnet]                   │  │     │   │
│   │   │                │    │  └────────────────────────────┘  │     │   │
│   │   └────────────────┘    └──────────────────────────────────┘     │   │
│   │                                                                  │   │
│   └──────────────────────────── repeat for each subsystem ───────────┘   │
│                                                                          │
│   ┌──────────────────────────────────────────────────────────────────┐   │
│   │                    SYSTEM VALIDATION                              │   │
│   │                                                                  │   │
│   │  integration tests (L2, L3) · sanity checks · limiting cases     │   │
│   │  reference data · convergence assessment                         │   │
│   │                                                                  │   │
│   │  [opus]                                                          │   │
│   └──────────────────────────────────┬───────────────────────────────┘   │
│                                      │                                   │
│              ┌───────────────────────┴───────────────────────┐           │
│              │           HUMAN REVIEW GATE                   │           │
│              │                                               │           │
│              │  • Accept (answer converged, exit spiral)     │           │
│              │  • Continue (next revolution)                 │           │
│              │  • Redirect (guidance for next revolution)    │           │
│              └───────────────────────┬───────────────────────┘           │
│                                      │                                   │
│                     ┌────────────────┘                                   │
│                     ↓                                                    │
│              (next spiral pass)                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

**Outer Loop (Spiral):** Convergence-driven. Each pass iterates over all subsystems, then validates the system as a whole. Human-gated: the human reviews results after each pass and decides to accept, continue, or redirect.

**Per-Subsystem Half-V:** Each subsystem goes through refine (methodology + plan update) → build (autonomous Ralph loop implementing tasks) → local verification (L0, L1 tests).

**System Validation:** After all subsystems are updated, integration tests (L2), system tests (L3), sanity checks, limiting cases, reference data comparisons, and convergence assessment.

## Key Distinction: Verification vs. Validation

- **Verification** is local to each subsystem: "Did I implement my equations correctly?" Tests at Level 0 (individual functions) and Level 1 (subsystem models). Happens within each subsystem's build phase using a strict red/green TDD cycle.
- **Validation** is global, at the system level: "Does the assembled system answer the question with physically sensible numbers?" Tests at Level 2 (coupled subsystems) and Level 3 (full system), plus sanity checks, limiting cases, reference data. Happens once per spiral pass after all subsystems are updated.

## Subsystem Decomposition

The problem is decomposed into subsystems during Pass 0 (scoping). The decomposition is captured in `SUBSYSTEMS.md` — the most critical artifact in the process. Each subsystem:

- Models one distinct physical phenomenon or sub-question
- Has typed interfaces: named quantities with units flowing between subsystems
- Can be verified in isolation with synthetic inputs
- Has its own methodology, implementation plan, and verification cases

Within a spiral pass, subsystems run in dependency order. Each subsystem uses the latest available values from subsystems that already ran (Gauss-Seidel pattern), and previous-pass values from subsystems that haven't yet. Circular dependencies are expected and resolved by the spiral iteration.

## Pass 0 — Scoping

The only non-repeating pass. Establishes what we're solving, how we'll know we've succeeded, what subsystems exist, and how they connect. **No code is written.**

**Produces:**
- `SUBSYSTEMS.md` — Subsystem definitions, interfaces, iteration order (the central new artifact)
- Per-subsystem initial files: methodology stubs, implementation plans, verification cases
- Acceptance criteria, validation strategy, sanity checks, literature survey, spiral plan
- System-level methodology overview

**Human review:** Mandatory. The decomposition is the most critical review item.

## Pass N (N >= 1) — Subsystem Iteration + System Validation

### Phase 1: Subsystem Iteration

For each subsystem in dependency order:

**Refine (opus):** Refines methodology for this subsystem at this pass's fidelity. Updates equations, assumptions, plan tasks. Reads previous pass validation results and human feedback.

**Build (sonnet, Ralph loop):** Autonomous iterative implementation using a strict red/green TDD cycle. Each iteration picks a task, writes a failing test from a verification case (red), implements code to make it pass (green), and repeats for each test/implement pair. Derivation docs are written only when the equation-to-code mapping is non-trivial. If methodology doesn't work in practice, raises a formal reconsideration.

### Phase 2: System Validation (opus)

Runs after all subsystems have been updated:
- Integration tests (L2) — coupled subsystem pairs
- System tests (L3) — full system
- Sanity checks, limiting cases, reference data, acceptance criteria
- Convergence assessment — compares key outputs with previous pass

### Phase 3: Human Review Gate (mandatory)

The review gate extracts key information from the review package and displays it directly in the terminal:

```
  ═══════════════════════════════════════════════════════
    SPIRAL PASS N COMPLETE — REVIEW REQUIRED
  ═══════════════════════════════════════════════════════

    Answer:      [current quantitative answer]
    Convergence: [CONVERGED / NOT YET / DIVERGING]
    Tests:       L0: X/Y | L1: X/Y | L2: X/Y | L3: X/Y
    Agent recommends: [ACCEPT / CONTINUE / BLOCKED]

    [A] ACCEPT — converged, produce final report
    [C] CONTINUE — next spiral pass
    [R] REDIRECT — provide guidance (opens $EDITOR)
```

## Human Interaction

### During Subsystem Build (When Blocked)

The block gate shows blocked task names and reasons:

```
  ═══════════════════════════════════════════════════════
    BUILD BLOCKED: [subsystem-name]
  ═══════════════════════════════════════════════════════

    Completed: X / Y tasks
    Blocked:   Z tasks

    Blocked tasks:
      • Task N: [name]
        Reason: [why blocked]

    [F] FIX — resolve blocks, then resume build
    [S] SKIP — continue to next subsystem
    [X] ABORT — stop this spiral pass
```

## Final Deliverables

When the human accepts:

1. **`output/answer.md`** — Direct response to the question in BRIEF.md
2. **`output/report.md`** — Full development report: problem statement, subsystem decomposition, per-subsystem methodology with citations, spiral history, V&V summaries, convergence tables, assumptions, limitations, traceability

## Traceability Chain

```
BRIEF.md → acceptance criteria
  → subsystem decomposition (SUBSYSTEMS.md)
    → per-subsystem methodology (subsystems/[name]/methodology.md)
      → peer-reviewed source (references/)
        → governing equations
          → discrete implementation (subsystems/[name]/derivations/)
            → source code (src/)
              → subsystem verification (L0, L1 tests)
                → system validation (L2, L3, sanity checks)
                  → convergence assessment
                    → human acceptance
                      → final answer + report
```

## Configuration

All configuration is in `lisa.conf`:

```bash
# Model selection per phase
CLAUDE_MODEL_SCOPE="opus"        # Pass 0 scoping
CLAUDE_MODEL_REFINE="opus"       # Per-subsystem methodology refinement
CLAUDE_MODEL_BUILD="sonnet"      # Per-subsystem implementation (Ralph loop)
CLAUDE_MODEL_VALIDATE="opus"     # System-level V&V and convergence

# Loop limits
MAX_SPIRAL_PASSES=5              # Max spiral passes
MAX_RALPH_ITERATIONS=50          # Max build iterations per subsystem per pass

# Human review
NO_PAUSE=false                   # Skip all human review?

# Git
NO_PUSH=false                    # Skip git push?
```

## Directory Structure

```
project-root/
├── loop.sh
├── lisa.conf
├── BRIEF.md                            # Project description (user writes)
├── AGENTS.md                           # Build/test/plot commands (user writes)
├── SUBSYSTEMS.md                       # Subsystem manifest + interfaces (created Pass 0)
│
├── prompts/
│   ├── PROMPT_scope.md                 # Pass 0: decomposition + scoping
│   ├── PROMPT_subsystem_refine.md      # Per-subsystem methodology + plan
│   ├── PROMPT_subsystem_build.md       # Per-subsystem Ralph loop iteration
│   └── PROMPT_system_validate.md       # System-level V&V + convergence
│
├── subsystems/
│   └── [name]/                         # One directory per subsystem
│       ├── methodology.md              # Equations, assumptions, citations
│       ├── plan.md                     # Implementation tasks
│       ├── verification-cases.md       # L0 and L1 test specifications
│       └── derivations/                # Code ↔ equations mapping
│
├── spiral/
│   ├── current-state.md
│   ├── pass-0/
│   │   ├── acceptance-criteria.md
│   │   ├── validation-strategy.md
│   │   ├── sanity-checks.md
│   │   ├── literature-survey.md
│   │   ├── spiral-plan.md
│   │   └── PASS_COMPLETE.md
│   ├── pass-N/
│   │   ├── subsystems/
│   │   │   └── [name]/
│   │   │       ├── refine-summary.md
│   │   │       └── reconsiderations/
│   │   ├── system-validation.md
│   │   ├── convergence.md
│   │   ├── review-package.md
│   │   ├── human-redirect.md          # (if redirected)
│   │   └── PASS_COMPLETE.md
│   └── SPIRAL_COMPLETE.md
│
├── methodology/
│   ├── overview.md                     # System-level: description, decomposition
│   └── assumptions-register.md         # Cross-cutting assumptions
│
├── validation/
│   ├── sanity-checks.md
│   ├── limiting-cases.md
│   ├── reference-data.md
│   └── convergence-log.md
│
├── references/
│   ├── core/
│   └── retrieved/
│
├── plots/
│   └── REVIEW.md
├── src/
├── tests/
│
└── output/
    ├── answer.md
    └── report.md
```

## Lineage

Lisa Loop extends the [Ralph Wiggum technique](https://ghuntley.com/ralph/) created by [Geoffrey Huntley](https://github.com/ghuntley/how-to-ralph-wiggum) — a bash loop that feeds prompts to an AI agent with filesystem persistence as shared state.

**Lisa Loop v1** added methodology rigor, hierarchical verification, and a reconsideration protocol for engineering/scientific software where "passing tests" is necessary but insufficient — the tests themselves might encode wrong physics.

**Lisa Loop v2** restructures the process into a subsystem-based spiral-V architecture: the scoping phase decomposes the problem into subsystems with typed interfaces, and each spiral pass iterates over subsystems individually — each doing its own half-V (refine → build → verify) — followed by system-level validation and convergence checking. This is a faithful mapping of Evans' design spiral, where each revolution visits distinct sectors in sequence, each using updated inputs from the sectors before it. The spiral history — not just the final answer — is the deliverable.

Named after Lisa Simpson — the rigorous counterpart to Ralph Wiggum.
