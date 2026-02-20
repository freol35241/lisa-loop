# Lisa Loop v2 — Design Specification

This document is the authoritative design specification for Lisa Loop v2. It defines the architecture, processes, artifact formats, and rules. All implementation must be consistent with this specification.

## Core Philosophy

Lisa Loop is a methodology and toolbox for solving complex engineering and research problems with AI agents, grounded in peer-reviewed literature, with explicit verification, validation, and convergence tracking.

v2 fuses two established engineering paradigms:

- **The V-Model** (systems engineering): every level of decomposition is paired with a corresponding level of verification and validation. V&V criteria are defined *before* implementation, not after.
- **The Design Spiral** (Evans, 1959): the same aspects are revisited iteratively at increasing fidelity until the design converges.

The fusion: **each revolution of the spiral passes through the full V — from requirements through implementation and back up through V&V — at increasing fidelity. The spiral terminates when the answer has converged, not when tasks are complete.**

Three absolute rules:
1. **Every methodological choice must trace to a peer-reviewed source.** No equation without a paper. No method without a citation. No exception, regardless of spiral pass.
2. **Engineering judgment is a first-class, auditable artifact.** "Do these numbers make physical sense?" is always asked, and the checks are written down, versioned, and executed.
3. **The spiral history is the deliverable, not just the answer.** Every methodological choice, every refinement, every convergence step is preserved as a complete record of how the answer was developed.

## Architecture: Loops within Loops

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

**Outer Loop (Spiral):** Convergence-driven. Each pass produces a progressively refined answer. Human-gated: the human reviews results after each pass and decides to accept, continue, or redirect. Configurable maximum number of passes.

**Inner Loop (Ralph Loop):** Autonomous task execution within a single spiral pass. The agent picks tasks from the implementation plan, implements them, runs tests, and fixes issues. High autonomy: the agent attempts to resolve blocked items at least once before surfacing to the human. Configurable maximum number of iterations.

## Spiral Passes

### Pass 0 — Scoping

The only non-repeating pass. Establishes what we're trying to answer and how we'll know we've succeeded. No code is written.

**Agent produces:**
- `spiral/pass-0/acceptance-criteria.md` — What "a correct answer" looks like. Quantitative where possible. What accuracy is needed? What decisions will be made based on this answer?
- `spiral/pass-0/validation-strategy.md` — How we'll validate. Known limiting cases, reference data, conservation laws, dimensional constraints, order-of-magnitude estimates from first principles.
- `spiral/pass-0/sanity-checks.md` — Engineering judgment checks: expected magnitudes, expected trends, physical bounds, things that would indicate a clearly wrong answer.
- `spiral/pass-0/literature-survey.md` — Survey of candidate methods at varying fidelity. What does the state of the art support? What is the fidelity ceiling? Cites papers found, summarizes approaches, notes which papers are available and which need `[NEEDS_PAPER]`.
- `spiral/pass-0/spiral-plan.md` — High-level roadmap: anticipated methods at each pass, expected convergence path. A plan of intent, not a commitment.
- `IMPLEMENTATION_PLAN.md` — Initial cumulative plan. Tasks for Pass 1 are fleshed out; later passes are sketched.

**Human review:** Mandatory after Pass 0.

**Exit marker:** `spiral/pass-0/PASS_COMPLETE.md`

### Pass N (N ≥ 1) — Three Phases

**Phase 1: Descend (Left Leg of V)** — Single agent invocation.
- Reads previous pass results, convergence analysis, human feedback (if redirected), existing methodology docs, current IMPLEMENTATION_PLAN.md
- Refines/extends methodology: method selected with full citation, alternatives considered, justification, governing equations written out, assumptions listed, valid range documented
- Updates cumulative methodology docs: `methodology/[subsystem].md`, `methodology/assumptions-register.md`, `methodology/coupling-strategy.md`, `methodology/verification-cases.md`
- Updates `IMPLEMENTATION_PLAN.md`: new tasks added, existing tasks revised, tasks tagged with current pass number. Tasks for this pass fully detailed; future tasks may be sketched.
- Produces `spiral/pass-N/descend-summary.md`
- Human review: Optional (configurable)

**Phase 2: Build (Bottom of V)** — The Ralph loop, multiple autonomous agent invocations.
- Each iteration: read IMPLEMENTATION_PLAN.md, find next TODO task with dependencies DONE, read methodology, implement, write derivation docs, run tests at affected level and above, generate plots, mark task DONE
- If blocked: attempt resolution at least once, then mark BLOCKED with explanation of what was tried and what is needed, move to next unblocked task
- If methodology doesn't work in practice: document in `spiral/pass-N/reconsiderations/`, mark task BLOCKED
- Exit when: all pass tasks DONE, or all remaining BLOCKED, or max iterations reached

**Phase 3: Ascend (Right Leg of V)** — Single agent invocation.
- Verification: run full test suite (all levels), regenerate affected plots, spot-check code matches methodology
- Validation: execute sanity checks from `validation/sanity-checks.md`, check limiting cases, compare reference data, check acceptance criteria, dimensional analysis
- Convergence: compare key outputs with previous pass, compute relative change, assess whether changes are within accuracy bounds of methods used
- Produces: `spiral/pass-N/verification-report.md`, `spiral/pass-N/validation-report.md`, `spiral/pass-N/convergence.md`, `spiral/pass-N/review-package.md`
- Updates `validation/convergence-log.md` and `plots/REVIEW.md`
- Marks `spiral/pass-N/PASS_COMPLETE.md`
- Human review: Mandatory

## The Implementation Plan

`IMPLEMENTATION_PLAN.md` is a single, cumulative, living document that spans the entire project. Created in Pass 0, updated by every descend phase.

```markdown
# Implementation Plan

## Architecture Overview
[High-level code architecture, updated as understanding deepens]

## Dependencies
[External libraries, tools, data files]

## Task List

### Task N: [Short descriptive name]
- **Status:** TODO | IN_PROGRESS | DONE | BLOCKED
- **Spiral pass:** [Which pass introduced/last revised this task]
- **Subsystem:** [Which methodology subsystem]
- **Methodology ref:** [Section in methodology/*.md]
- **Implementation:**
  - [ ] [Specific code to write]
  - [ ] [Specific code to write]
- **Derivation:**
  - [ ] Document discretization / mapping from continuous equations to code
- **Verification:**
  - [ ] [Specific test from verification-cases.md]
- **Plots:**
  - [ ] [Specific plot for visual verification]
- **Dependencies:** [Other tasks that must complete first]
```

**How it evolves:** Pass 0 creates it with full structure; tasks for Pass 1 detailed, later passes sketched. Each descend updates it: new tasks added, existing tasks revised, completed tasks remain as historical record. The Ralph loop executes TODO tasks and marks them DONE. Task sizing: each task completable in a single Ralph iteration. If >5 implementation checkboxes, split it.

## Human Interaction

### Review Gate: After Ascend (Mandatory)

```
═══════════════════════════════════════════════════════
  SPIRAL PASS N COMPLETE — REVIEW REQUIRED
═══════════════════════════════════════════════════════

  Review package: spiral/pass-N/review-package.md
  Plots:          plots/REVIEW.md

  [A] ACCEPT — Answer has converged. Produce final report.
  [C] CONTINUE — Proceed to Pass N+1.
  [R] REDIRECT — Provide guidance for Pass N+1.

═══════════════════════════════════════════════════════
```

If REDIRECT: human provides free-text saved to `spiral/pass-N/human-redirect.md`.

### Block Surfacing: During Build (When Needed)

```
═══════════════════════════════════════════════════════
  BUILD PHASE BLOCKED — HUMAN INPUT NEEDED
═══════════════════════════════════════════════════════

  Completed: X/Y tasks
  Blocked:   Z tasks

  See IMPLEMENTATION_PLAN.md for blocked items and details.

  [F] FIX — Resolve the blocks, then resume build.
  [S] SKIP — Skip blocked items, proceed to Ascend.
  [X] ABORT — Stop this spiral pass.

═══════════════════════════════════════════════════════
```

### Descend Review: Optional (Configurable)

```
═══════════════════════════════════════════════════════
  DESCEND COMPLETE — METHODOLOGY REVIEW
═══════════════════════════════════════════════════════

  Methodology: spiral/pass-N/descend-summary.md
  Updated plan: IMPLEMENTATION_PLAN.md

  [P] PROCEED — Start building.
  [R] REDIRECT — Adjust before building.

═══════════════════════════════════════════════════════
```

## Review Package Format

```markdown
# Spiral Pass N — Review Package

## Summary
[One paragraph: what was done this pass and why]

## Current Answer
[The actual answer to the user's question, as of this pass.]

## Convergence
| Quantity          | Pass N-1   | Pass N     | Δ (%)  | Converged? |
|-------------------|-----------|-----------|--------|------------|
| [key output 1]   | [value]   | [value]   | [X.X]  | [yes/no]   |

Overall assessment: [CONVERGED / NOT YET CONVERGED / DIVERGING]

## Verification
- Tests: X/Y passing
- [Any failures noted]

## Validation — Automated Checks
- [ ] Order of magnitude: [result]
- [ ] Trends: [result]
- [ ] Conservation: [result]
- [ ] Dimensional analysis: [result]
- [ ] Limiting cases: [result]
- [ ] Reference data comparison: [result]

## Validation — Engineering Judgment (YOUR REVIEW)
1. [Plot: path/to/plot.png] → Does [quantity] vs [parameter] show expected shape?
2. [Key result]: [quantity] = [value] [units] → Reasonable for [context]?
3. [Trend]: When [parameter] increases, [quantity] [direction] → Expected?

## Agent Recommendation
[ACCEPT / CONTINUE: refine X because Y / BLOCKED: need Z]

## If Continuing — Proposed Refinements for Pass N+1
- [What to refine and why, with literature pointers]
```

## Final Output

When the human accepts, the loop produces two deliverables:

**1. The Answer** — `output/answer.md` — Direct response to the question in BRIEF.md.

**2. The Development Report** — `output/report.md`:

```markdown
# Development Report

## Problem Statement
[From BRIEF.md]

## Acceptance Criteria
[From spiral/pass-0/acceptance-criteria.md]

## Methodology
[Cumulative methodology summary with citations]

## Spiral History
### Pass 1
- Focus, methods (with citations), key result
### Pass 2
- Focus, refinements, key result, convergence from Pass 1
### Pass N (final)
- Final convergence assessment

## Verification Summary
## Validation Summary
## Convergence Summary
[Table showing key quantities across all passes]

## Assumptions and Limitations
## References
## Traceability
[Chain from acceptance criterion → methodology → code → V&V → final value]
```

## Traceability Chain

For every claim in the final output:

```
User's question (BRIEF.md)
  → Acceptance criteria (spiral/pass-0/acceptance-criteria.md)
    → Methodology choice (methodology/[subsystem].md)
      → Peer-reviewed source (references/)
        → Governing equations (methodology/[subsystem].md)
          → Discrete implementation (derivations/)
            → Source code (src/)
              → Verification test (tests/)
                → Verification result (spiral/pass-N/verification-report.md)
                  → Validation check (spiral/pass-N/validation-report.md)
                    → Convergence assessment (spiral/pass-N/convergence.md)
                      → Human acceptance (SPIRAL_COMPLETE.md)
                        → Final answer + development report (output/)
```

No link in this chain may be skipped.

## Configuration

**lisa.conf:**
```bash
# Claude Code model selection per phase
CLAUDE_MODEL_SCOPE="opus"
CLAUDE_MODEL_DESCEND="opus"
CLAUDE_MODEL_BUILD="sonnet"
CLAUDE_MODEL_ASCEND="opus"

# Loop limits
MAX_SPIRAL_PASSES=5
MAX_RALPH_ITERATIONS=50
MAX_RALPH_BLOCKED_RETRIES=1

# Human review
REVIEW_DESCEND=false
NO_PAUSE=false

# Git
NO_PUSH=false
```

## CLI

```bash
./loop.sh scope                  # Run Pass 0 only
./loop.sh run                    # Full spiral (scope if needed, then iterate)
./loop.sh run --max-passes 3     # Limit spiral passes
./loop.sh resume                 # Resume from current state
./loop.sh status                 # Print current state
```

## Directory Structure

```
project-root/
├── loop.sh
├── lisa.conf
├── BRIEF.md
├── AGENTS.md
├── IMPLEMENTATION_PLAN.md          # Cumulative (created Pass 0, updated each Descend)
│
├── prompts/
│   ├── PROMPT_scope.md
│   ├── PROMPT_descend.md
│   ├── PROMPT_build.md
│   └── PROMPT_ascend.md
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
│   │   ├── descend-summary.md
│   │   ├── reconsiderations/
│   │   ├── verification-report.md
│   │   ├── validation-report.md
│   │   ├── convergence.md
│   │   ├── review-package.md
│   │   ├── human-redirect.md
│   │   └── PASS_COMPLETE.md
│   └── SPIRAL_COMPLETE.md
│
├── methodology/
│   ├── overview.md
│   ├── [subsystem].md
│   ├── assumptions-register.md
│   ├── coupling-strategy.md
│   └── verification-cases.md
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
├── derivations/
├── plots/
│   └── REVIEW.md
├── src/
├── tests/
│
└── output/
    ├── answer.md
    └── report.md
```

## Relationship to v1

| v1 Concept | v2 Evolution |
|-----------|-------------|
| BRIEF.md | Unchanged |
| Methodology phase | Distributed: Pass 0 (survey) + Descend phases (per-pass refinement) |
| Planning phase | Cumulative IMPLEMENTATION_PLAN.md, created Pass 0, updated each Descend |
| Building phase | Ralph loop within each pass's Build phase |
| Review phase | Ascend phase of each pass |
| Triage phase | Eliminated — validation/convergence failures drive next pass |
| Reconsideration protocol | Blocks surface during build, next descend adjusts methodology |
| Hierarchical verification | Unchanged, runs every Ascend |
| METHODOLOGY_COMPLETE.md | No equivalent — methodology grows continuously |
| [BUILD_COMPLETE] | Replaced by SPIRAL_COMPLETE.md (human acceptance of convergence) |
| Agent-agnostic | Claude Code only |
