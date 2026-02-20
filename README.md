# Lisa Loop v2

A methodology and toolbox for solving complex engineering and research problems with AI agents, grounded in peer-reviewed literature, with explicit verification, validation, and convergence tracking.

Lisa Loop v2 fuses two established engineering paradigms:

- **The V-Model** (systems engineering): every level of decomposition is paired with a corresponding level of verification and validation. V&V criteria are defined *before* implementation, not after.
- **The Design Spiral** (Evans, 1959): the same aspects are revisited iteratively at increasing fidelity until the design converges.

**The fusion:** each revolution of the spiral passes through the full V — from requirements through implementation and back up through V&V — at increasing fidelity. The spiral terminates when the answer has converged, not when tasks are complete.

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
┌─────────────────────────────────────────────────────────────┐
│                    OUTER LOOP: SPIRAL                        │
│              (convergence-driven, human-gated)               │
│                                                              │
│   ┌──────────┐    ┌──────────────────┐    ┌──────────────┐  │
│   │ DESCEND  │    │     BUILD        │    │   ASCEND     │  │
│   │          │ →  │                  │ →  │              │  │
│   │ refine   │    │ ┌──────────────┐ │    │ verify       │  │
│   │ methodol │    │ │ INNER LOOP:  │ │    │ validate     │  │
│   │ + update │    │ │ RALPH        │ │    │ converge?    │  │
│   │ plan     │    │ │              │ │    │              │  │
│   │          │    │ │ pick task →  │ │    │ [agent]      │  │
│   │ [agent]  │    │ │ implement → │ │    │              │  │
│   │          │    │ │ test →      │ │    │              │  │
│   │          │    │ │ fix → next  │ │    │              │  │
│   │          │    │ │              │ │    │              │  │
│   │          │    │ │ [agent]      │ │    │              │  │
│   │          │    │ └──────────────┘ │    │              │  │
│   └──────────┘    └──────────────────┘    └──────┬───────┘  │
│        ↑                                         │           │
│        │            ┌────────────────────────┐   │           │
│        │            │   HUMAN REVIEW GATE    │ ←─┘           │
│        │            │                        │               │
│        │            │  • Accept (exit loop)  │               │
│        │            │  • Continue            │               │
│        │            │  • Redirect            │               │
│        │            └────────────┬───────────┘               │
│        │                         │                           │
│        └─────────────────────────┘                           │
└─────────────────────────────────────────────────────────────┘
```

**Outer Loop (Spiral):** Convergence-driven. Each pass produces a progressively refined answer. Human-gated: the human reviews results after each pass and decides to accept, continue, or redirect.

**Inner Loop (Ralph Loop):** Autonomous task execution within a single spiral pass. The agent picks tasks from the implementation plan, implements them, runs tests, and fixes issues.

## Pass 0 — Scoping

The only non-repeating pass. Establishes what we're trying to answer and how we'll know we've succeeded. No code is written.

**Produces:**
- Acceptance criteria — what "correct" looks like, quantitatively
- Validation strategy — how results will be validated (limiting cases, reference data, conservation laws)
- Sanity checks — engineering judgment tests
- Literature survey — candidate methods with citations
- Spiral plan — anticipated progression across passes
- Initial implementation plan

**Human review:** Mandatory after scoping.

## Pass N (N ≥ 1) — Three Phases

### Descend (Left Leg of V)

Refines methodology based on previous pass results. Reads convergence data, human feedback, and existing methodology. Updates the implementation plan with tasks for this pass.

- Every method choice must cite a peer-reviewed source
- Alternatives must be documented
- Tasks tagged with current pass number

### Build (Bottom of V — Ralph Loop)

Autonomous iterative implementation. Each iteration:

1. Pick next TODO task with dependencies met
2. Implement code matching methodology exactly
3. Write derivation documentation
4. Run hierarchical verification tests
5. Generate/update plots
6. Mark task DONE or BLOCKED

If methodology doesn't work in practice, the agent raises a formal reconsideration rather than silently changing the approach.

### Ascend (Right Leg of V)

Verification, validation, and convergence assessment:

- **Verification:** Run full test suite (all levels), check methodology compliance, derivation completeness, traceability
- **Validation:** Execute sanity checks, check limiting cases, compare reference data, dimensional analysis
- **Convergence:** Compare key outputs with previous pass, assess whether changes are within accuracy bounds

Produces a review package for the human.

## Human Interaction

### After Ascend (Mandatory)

```
  SPIRAL PASS N COMPLETE — REVIEW REQUIRED

  [A] ACCEPT — Answer has converged. Produce final report.
  [C] CONTINUE — Proceed to Pass N+1.
  [R] REDIRECT — Provide guidance for Pass N+1.
```

### During Build (When Blocked)

```
  BUILD PHASE BLOCKED — HUMAN INPUT NEEDED

  [F] FIX — Resolve the blocks, then resume build.
  [S] SKIP — Skip blocked items, proceed to Ascend.
  [X] ABORT — Stop this spiral pass.
```

### After Descend (Optional, configurable)

```
  DESCEND COMPLETE — METHODOLOGY REVIEW

  [P] PROCEED — Start building.
  [R] REDIRECT — Adjust before building.
```

## The Implementation Plan

`IMPLEMENTATION_PLAN.md` is a single, cumulative, living document that spans the entire project. Created in Pass 0, updated by every descend phase, executed by the Ralph loop.

Each task specifies: status, spiral pass, subsystem, methodology reference, implementation items, derivation, verification tests, plots, and dependencies.

## Verification, Validation, and Convergence

These are distinct concepts:

- **Verification:** "Did we build the thing right?" — code matches methodology, tests pass, derivations are complete
- **Validation:** "Did we build the right thing?" — results match physical reality (sanity checks, limiting cases, reference data, conservation laws)
- **Convergence:** "Has the answer stabilized?" — key outputs are no longer changing significantly between passes

The spiral terminates when the human accepts that all three are satisfied.

## Final Deliverables

When the human accepts, the loop produces:

1. **`output/answer.md`** — Direct response to the question in BRIEF.md
2. **`output/report.md`** — Full development report: problem statement, methodology with citations, spiral history, V&V summaries, convergence tables, assumptions, limitations, traceability

## Traceability Chain

Every claim in the final output traces through:

```
BRIEF.md → acceptance criteria → methodology → peer-reviewed source →
governing equations → discrete implementation → source code →
verification test → verification result → validation check →
convergence assessment → human acceptance → final answer
```

## Configuration

All configuration is in `lisa.conf`:

```bash
# Model selection per phase
CLAUDE_MODEL_SCOPE="opus"       # Pass 0 scoping
CLAUDE_MODEL_DESCEND="opus"     # Methodology refinement
CLAUDE_MODEL_BUILD="sonnet"     # Implementation (inner loop)
CLAUDE_MODEL_ASCEND="opus"      # V&V and convergence

# Loop limits
MAX_SPIRAL_PASSES=5             # Max spiral passes
MAX_RALPH_ITERATIONS=50         # Max build iterations per pass
MAX_RALPH_BLOCKED_RETRIES=1     # Unblock attempts before surfacing

# Human review
REVIEW_DESCEND=false            # Review methodology after descend?
NO_PAUSE=false                  # Skip all human review?

# Git
NO_PUSH=false                   # Skip git push?
```

## Directory Structure

```
project-root/
├── loop.sh                         # The spiral-V loop script
├── lisa.conf                       # Configuration
├── BRIEF.md                        # Project description (you write this)
├── AGENTS.md                       # Build/test/plot commands (you write this)
├── IMPLEMENTATION_PLAN.md          # Cumulative plan (created Pass 0, updated each Descend)
│
├── prompts/
│   ├── PROMPT_scope.md             # Pass 0: scoping
│   ├── PROMPT_descend.md           # Left leg of V: methodology + plan
│   ├── PROMPT_build.md             # Bottom of V: Ralph loop iteration
│   └── PROMPT_ascend.md            # Right leg of V: V&V + convergence
│
├── spiral/
│   ├── current-state.md            # Loop state for resume
│   ├── pass-0/                     # Scoping artifacts
│   │   ├── acceptance-criteria.md
│   │   ├── validation-strategy.md
│   │   ├── sanity-checks.md
│   │   ├── literature-survey.md
│   │   ├── spiral-plan.md
│   │   └── PASS_COMPLETE.md
│   ├── pass-N/                     # Per-pass artifacts
│   │   ├── descend-summary.md
│   │   ├── reconsiderations/
│   │   ├── verification-report.md
│   │   ├── validation-report.md
│   │   ├── convergence.md
│   │   ├── review-package.md
│   │   ├── human-redirect.md       # (if redirected)
│   │   └── PASS_COMPLETE.md
│   └── SPIRAL_COMPLETE.md          # Created on acceptance
│
├── methodology/
│   ├── overview.md
│   ├── [subsystem].md              # One per physical subsystem
│   ├── assumptions-register.md
│   ├── coupling-strategy.md
│   ├── verification-cases.md
│   └── reconsiderations/           # Formal methodology change requests
│
├── validation/
│   ├── sanity-checks.md            # Living engineering judgment checks
│   ├── limiting-cases.md
│   ├── reference-data.md
│   └── convergence-log.md          # Cumulative convergence table
│
├── references/
│   ├── core/                       # Your papers (PDFs)
│   └── retrieved/                  # Papers found by agent
│
├── derivations/                    # Code ↔ equations mapping
├── plots/
│   └── REVIEW.md                   # Visual review summary
├── src/                            # Source code
├── tests/                          # Test suite
│
└── output/
    ├── answer.md                   # Final answer
    └── report.md                   # Development report
```

## Lineage

Lisa Loop extends the [Ralph Wiggum technique](https://ghuntley.com/ralph/) created by [Geoffrey Huntley](https://github.com/ghuntley/how-to-ralph-wiggum) — a bash loop that feeds prompts to an AI agent with filesystem persistence as shared state.

**Lisa Loop v1** added methodology rigor, hierarchical verification, and a reconsideration protocol for engineering/scientific software where "passing tests" is necessary but insufficient — the tests themselves might encode wrong physics.

**Lisa Loop v2** restructures the waterfall-with-feedback model into a spiral-V architecture: each revolution of the spiral passes through the full V-model, methodology is refined continuously rather than locked upfront, and convergence tracking determines when the answer is ready. The spiral history — not just the final answer — is the deliverable.

Named after Lisa Simpson — the rigorous counterpart to Ralph Wiggum.
