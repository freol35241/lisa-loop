# Lisa Loop v2 — Design Specification

This document is the authoritative design specification for Lisa Loop v2. It defines the architecture, processes, artifact formats, and rules. All implementation must be consistent with this specification.

## Core Philosophy

Lisa Loop is a methodology and toolbox for solving complex engineering and research problems with AI agents, grounded in peer-reviewed literature, with explicit verification, validation, and convergence tracking.

v2 fuses two established engineering paradigms:

- **The V-Model** (systems engineering): every level of decomposition is paired with a corresponding level of verification and validation. V&V criteria are defined *before* implementation, not after.
- **The Design Spiral** (Evans, 1959): the same aspects (sectors/subsystems) are revisited iteratively at increasing fidelity until the design converges.

The fusion: **each revolution of the spiral visits every subsystem in sequence, each doing a half-V (refine → build → verify), then the system is validated as a whole. The spiral terminates when the system-level answer has converged, not when tasks are complete.**

Three absolute rules:
1. **Every methodological choice must trace to a peer-reviewed source.** No equation without a paper. No method without a citation.
2. **Engineering judgment is a first-class, auditable artifact.** "Do these numbers make physical sense?" is always asked, and the checks are written down, versioned, and executed.
3. **The spiral history is the deliverable, not just the answer.** Every methodological choice, every refinement, every convergence step is preserved.

## Key Distinction: Verification vs. Validation

- **Verification** is local to each subsystem: "Did I implement my equations correctly?" Tests at Level 0 (individual functions) and Level 1 (subsystem models). Happens within each subsystem's half-V.
- **Validation** is global, at the system level: "Does the assembled system answer the question with physically sensible numbers?" Tests at Level 2 (coupled subsystems) and Level 3 (full system), plus sanity checks, limiting cases, reference data. Happens once per spiral pass after all subsystems are updated.

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

## How Subsystems Interact Across a Pass

Within a spiral pass, subsystems run in dependency order. When subsystem B runs after subsystem A:
- B uses A's **newly updated** outputs from this pass
- B uses outputs from subsystems that haven't run yet from the **previous pass**

This is the Gauss-Seidel pattern: use the latest available information, let the spiral converge. Circular dependencies are expected and handled by the iteration.

For Pass 1, when no previous-pass values exist, the initial estimates from `SUBSYSTEMS.md` are used as the starting values. These estimates bootstrap the first coupled computation. By Pass 2, actual computed values replace the estimates, and convergence tracking begins.

## Decomposition Heuristics

The decomposition into subsystems is the highest-leverage decision in the process. The scope phase must follow these heuristics:

**What makes a good subsystem:**
- Models one distinct physical phenomenon or answers one clear sub-question
- Can be verified in isolation with synthetic inputs — if you can't test it without the whole system, it's not separable enough
- Has a narrow, well-defined interface — a handful of named quantities with units, not complex shared state
- Fits in a single agent's working context — if the methodology exceeds ~15–20 pages of equations and assumptions, split it
- Aligns with how the literature treats the problem — if separate papers cover separate aspects, those are natural boundaries

**What makes a good interface:**
- Physically meaningful quantities with units and expected ranges
- Directional: subsystem A *provides* X to subsystem B
- Stable across passes — the values change, the quantities exchanged don't
- Circular dependencies are expected — that's what the spiral resolves

**Anti-patterns:**
- More than ~7 subsystems — interface overhead dominates
- Fewer than 2 — you've collapsed to a monolith with no focused verification
- Subsystems defined by code structure ("the solver," "the I/O layer") rather than by physical phenomenon
- A subsystem that needs outputs from every other subsystem — sign of a missing abstraction

## Spiral Passes

### Pass 0 — Scoping

The only non-repeating pass. Establishes what we're solving, how we'll know we've succeeded, what subsystems exist, and how they connect. **No code is written.**

**Agent produces:**

1. **`SUBSYSTEMS.md`** — The subsystem manifest. This is the key artifact:

```markdown
# Subsystems

## Iteration Order
[Ordered list of subsystems. Approximately topological — exact ordering matters less than getting the general flow right, since the spiral converges regardless.]

1. [subsystem-a]
2. [subsystem-b]
3. [subsystem-c]

## Subsystem Definitions

### [subsystem-a]
- **Models:** [What physical phenomenon or sub-question]
- **Provides:** [Named outputs with units and expected ranges]
- **Consumes:** [Named inputs with units, and which subsystem provides each]
- **Key references:** [Primary papers for this subsystem]

### [subsystem-b]
...

## Interface Map

### [subsystem-a] → [subsystem-b]
- **Quantities:** [What flows from A to B, with units]
- **Expected range:** [Order of magnitude bounds]
- **Initial estimate:** [Best-guess numerical value for Pass 1 bootstrapping — from literature, reference design, or engineering judgment. Must be a specific number with units, not a range.]
- **Coupling strength:** [Weak / moderate / strong — how much does B's answer depend on A?]

### [subsystem-b] → [subsystem-a]  (circular dependency)
- **Quantities:** [What flows from B back to A]
- **Initial estimate:** [Starting value for the first spiral pass]
- **Resolution:** Previous pass values used; convergence tracked.

## Dependency Notes
[Any circular dependencies and how they're resolved by the spiral iteration. Which links use "latest from this pass" vs. "carry forward from last pass."]
```

2. **Per-subsystem initial files:**
   - `subsystems/[name]/methodology.md` — Initial methodology stub: phenomenon, candidate methods from literature, recommended approach, key equations (from papers, not fabricated), assumptions
   - `subsystems/[name]/plan.md` — Initial implementation plan for this subsystem. Tasks for Pass 1 detailed, later passes sketched.
   - `subsystems/[name]/verification-cases.md` — L0 and L1 test specifications for this subsystem

3. **System-level files:**
   - `spiral/pass-0/acceptance-criteria.md` — What "correct" looks like, quantitatively
   - `spiral/pass-0/validation-strategy.md` — How system-level results will be validated
   - `spiral/pass-0/sanity-checks.md` — Engineering judgment checks
   - `spiral/pass-0/literature-survey.md` — Methods survey organized by subsystem
   - `spiral/pass-0/spiral-plan.md` — Anticipated progression across passes
   - `validation/sanity-checks.md` — Living copy of sanity checks
   - `validation/limiting-cases.md` — Populated from validation strategy
   - `validation/reference-data.md` — Populated from validation strategy
   - `methodology/overview.md` — System description, subsystem decomposition, modeling approach
   - `methodology/assumptions-register.md` — Cross-cutting assumptions that span subsystems

**Human review:** Mandatory after Pass 0. The decomposition and interfaces are the most critical review.

**Exit marker:** `spiral/pass-0/PASS_COMPLETE.md`

### Pass N (N >= 1)

**Phase 1: Subsystem Iteration** — For each subsystem in dependency order:

**Step A: Refine (opus)** — Single agent invocation, scoped to one subsystem.
- Reads: `SUBSYSTEMS.md` (for interfaces), own `subsystems/[name]/methodology.md`, own `subsystems/[name]/plan.md`, previous pass system validation results, human redirect if any
- Refines methodology for this subsystem at this pass's fidelity: method with citation, equations, assumptions, valid range
- Updates own `subsystems/[name]/methodology.md`, `subsystems/[name]/plan.md`, `subsystems/[name]/verification-cases.md`
- Updates `methodology/assumptions-register.md` if cross-cutting assumptions change
- Produces `spiral/pass-N/subsystems/[name]/refine-summary.md`

**Step B: Build + Verify (sonnet, Ralph loop)** — Multiple autonomous agent invocations, scoped to one subsystem.
- Each iteration: read subsystem plan, find next TODO task, read methodology, implement, write derivation, run L0/L1 tests, generate plots, mark DONE
- Same rules as build prompt: methodology adherence, derivation documentation, hierarchical verification (L0 + L1), reconsideration protocol
- Exit when: all subsystem tasks DONE, or all remaining BLOCKED, or max iterations
- If blocked: surface block gate to human (scoped to this subsystem)

**Phase 2: System Validation (opus)** — Single agent invocation, system-level.
- Runs L2 tests (coupled subsystem pairs) and L3 tests (full system)
- Executes sanity checks from `validation/sanity-checks.md`
- Checks limiting cases, reference data, acceptance criteria
- Convergence: compares key system-level outputs with previous pass
- Produces: `spiral/pass-N/system-validation.md`, `spiral/pass-N/convergence.md`, `spiral/pass-N/review-package.md`
- Updates `validation/convergence-log.md`

**Phase 3: Human Review Gate** — Mandatory.
- Accept: answer converged, produce final output
- Continue: next spiral pass
- Redirect: guidance for next pass

## Per-Subsystem Plan Format

Each `subsystems/[name]/plan.md`:

```markdown
# [Subsystem Name] — Implementation Plan

## Tasks

### Task N: [Short descriptive name]
- **Status:** TODO | IN_PROGRESS | DONE | BLOCKED
- **Spiral pass:** [Which pass introduced/last revised this task]
- **Methodology ref:** [Section in subsystems/[name]/methodology.md]
- **Implementation:**
  - [ ] [Specific code to write]
  - [ ] [Specific code to write]
- **Derivation:**
  - [ ] Document discretization / mapping from continuous equations to code
- **Verification:**
  - [ ] [Specific L0 or L1 test from verification-cases.md]
- **Plots:**
  - [ ] [Specific plot for visual verification]
- **Dependencies:** [Other tasks in THIS subsystem that must complete first]
```

Task sizing: completable in one Ralph iteration. Max 5 implementation checkboxes, split if larger.

## Human Interaction

### Review Gate: After System Validation (Mandatory)

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

### Block Surfacing: During Subsystem Build (When Needed)

```
═══════════════════════════════════════════════════════
  BUILD BLOCKED: [subsystem-name] — HUMAN INPUT NEEDED
═══════════════════════════════════════════════════════

  Subsystem:  [name]
  Completed:  X/Y tasks
  Blocked:    Z tasks

  See subsystems/[name]/plan.md for blocked items.

  [F] FIX — Resolve the blocks, then resume build.
  [S] SKIP — Skip blocked items, continue to next subsystem.
  [X] ABORT — Stop this spiral pass.

═══════════════════════════════════════════════════════
```

## Review Package Format

```markdown
# Spiral Pass N — Review Package

## Summary
[One paragraph: what was done this pass, which subsystems were refined, why]

## Current Answer
[The actual answer to the user's question, as of this pass. Specific and quantitative.]

## Subsystem Status
| Subsystem | Tasks Done | Tasks Blocked | Key Result |
|-----------|-----------|---------------|------------|
| [name]    | X/Y       | Z             | [value]    |

## Convergence
| Quantity          | Pass N-1   | Pass N     | Δ (%)  | Converged? |
|-------------------|-----------|-----------|--------|------------|
| [key output 1]   | [value]   | [value]   | [X.X]  | [yes/no]   |

Overall assessment: [CONVERGED / NOT YET CONVERGED / DIVERGING]

## Verification (per subsystem)
| Subsystem | L0 Tests | L1 Tests | Issues |
|-----------|----------|----------|--------|
| [name]    | X/Y      | X/Y      | [any]  |

## Validation — System Level
- Integration tests (L2): X/Y passing
- Full system tests (L3): X/Y passing
- [ ] Order of magnitude: [result]
- [ ] Trends: [result]
- [ ] Conservation: [result]
- [ ] Dimensional analysis: [result]
- [ ] Limiting cases: [result]
- [ ] Reference data comparison: [result]

## Validation — Engineering Judgment (YOUR REVIEW)
1. [Plot: path] → Does [quantity] vs [parameter] show expected shape?
2. [Key result]: [quantity] = [value] [units] → Reasonable for [context]?
3. [Trend]: When [parameter] increases, [quantity] [direction] → Expected?

## Agent Recommendation
[ACCEPT / CONTINUE: refine X because Y / BLOCKED: need Z]

## If Continuing — Proposed Refinements for Pass N+1
- [What to refine per subsystem and why, with literature pointers]
```

## Final Output

When the human accepts:

1. **`output/answer.md`** — Direct response to the question in BRIEF.md.
2. **`output/report.md`** — Full development report: problem statement, subsystem decomposition, per-subsystem methodology with citations, spiral history, V&V summaries, convergence tables, assumptions, limitations, traceability.

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

**lisa.conf:**
```bash
# Claude Code model selection per phase
CLAUDE_MODEL_SCOPE="opus"
CLAUDE_MODEL_REFINE="opus"
CLAUDE_MODEL_BUILD="sonnet"
CLAUDE_MODEL_VALIDATE="opus"

# Loop limits
MAX_SPIRAL_PASSES=5
MAX_RALPH_ITERATIONS=50              # per subsystem per pass

# Human review
NO_PAUSE=false

# Git
NO_PUSH=false
```

## CLI

```bash
./loop.sh scope                  # Run Pass 0 only
./loop.sh run [--max-passes N]   # Full spiral
./loop.sh resume                 # Resume from current state
./loop.sh status                 # Print current state
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

## Relationship to v1

| v1 Concept | v2 Evolution |
|-----------|-------------|
| BRIEF.md | Unchanged |
| Methodology phase | Distributed: Pass 0 (survey + decomposition) + per-subsystem Refine phases |
| Planning phase | Per-subsystem plans in `subsystems/[name]/plan.md`, created Pass 0, updated each Refine |
| Building phase | Per-subsystem Ralph loop within each pass's Build phase |
| Review phase | System Validation phase of each pass |
| Triage phase | Eliminated — validation/convergence failures drive next pass |
| IMPLEMENTATION_PLAN.md | Replaced by per-subsystem `subsystems/[name]/plan.md` |
| Reconsideration protocol | Per-subsystem: blocks surface during build, next refine adjusts methodology |
| Hierarchical verification | Split: L0/L1 per-subsystem during build, L2/L3 during system validation |
| methodology/coupling-strategy.md | Replaced by interface map in SUBSYSTEMS.md |
| methodology/verification-cases.md | Replaced by per-subsystem verification-cases.md |
| METHODOLOGY_COMPLETE.md | No equivalent — methodology grows continuously per subsystem |
| [BUILD_COMPLETE] | Replaced by SPIRAL_COMPLETE.md (human acceptance of convergence) |
| Agent-agnostic | Claude Code only |
