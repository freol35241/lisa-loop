# Lisa Loop v2 — Design Specification

This document is the authoritative design specification for Lisa Loop v2. It defines the architecture, processes, artifact formats, and rules. All implementation must be consistent with this specification.

## Core Philosophy

Lisa Loop is a methodology and toolbox for solving complex engineering and research problems with AI agents, grounded in peer-reviewed literature, with explicit verification, validation, and convergence tracking.

v2 fuses two established engineering paradigms:

- **The V-Model** (systems engineering): every level of specification is paired with a corresponding level of verification and validation. V&V criteria are defined *before* implementation, not after.
- **The Design Spiral** (Evans, 1959): the same problem is revisited iteratively at increasing fidelity and scope until the answer converges.

The fusion: **each revolution of the spiral runs five phases (Refine → DDV Red → Build → Execute → Validate), then the human reviews. The spiral terminates when the system-level answer has converged, not when tasks are complete.**

Three absolute rules:
1. **Every methodological choice must trace to a peer-reviewed source.** No equation without a paper. No method without a citation.
2. **Engineering judgment is a first-class, auditable artifact.** "Do these numbers make physical sense?" is always asked, and the checks are written down, versioned, and executed.
3. **The spiral history is the deliverable, not just the answer.** Every methodological choice, every refinement, every convergence step is preserved.

## Key Distinction: Verification vs. Validation

- **Verification** is per-function and per-model: "Did I implement my equations correctly?" Domain-Driven Verification (DDV) tests at Level 0 (individual functions) and Level 1 (model level). Software quality tests for edge cases and stability. Happens within each pass's DDV Red and Build phases.
- **Validation** is global, at the system level: "Does the assembled system answer the question with physically sensible numbers?" Integration tests, sanity checks, limiting cases, reference data, engineering judgment audit. Happens in the Execute and Validate phases.

## Domain-Driven Verification (DDV)

DDV is the core verification mechanism. It uses a two-stage red/green pattern with agent separation:

1. **DDV Red** (opus): Writes failing tests from authoritative domain sources (papers, standards, analytical solutions). Does NOT read implementation code. Tests encode what the domain knowledge SHOULD produce.
2. **Build** (sonnet): Implements code to make those tests pass. CANNOT modify DDV tests. If a test seems wrong, the disagreement is documented as a reconsideration — the next refine phase adjudicates.

This two-agent separation prevents correlated domain knowledge errors: the test writer and the implementer interpret the same papers independently. Disagreements are valuable signals, not bugs to suppress.

**DDV is domain-agnostic.** The pattern works whenever there are authoritative sources and testable expected values: a physics paper → expected Cf at Re=1e7, an econometrics study → expected Gini coefficient, a regulatory standard → expected classification threshold. The domain doesn't matter — the verification mechanism does.

**Engineering judgment** is a named concept preserved across all domains. It means: dimensional analysis, conservation law checks, order-of-magnitude estimation from first principles, and hard physical bounds. The DDV prompts require this standard of rigor — it anchors what "sanity checking" means in concrete, verifiable terms rather than vague "does this look right?" heuristics.

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

## Scope Progression

The spiral plan stages both fidelity AND scope per pass. Early passes test the methodology on a SUBSET of the full problem — not the full scope at low fidelity.

| Pass | Scope subset | Fidelity | Acceptance | Key question |
|------|-------------|----------|------------|--------------|
| 1    | Narrow subset | Low | Wide tolerance | Does the approach work at all? |
| 2    | Broader | Medium | Tighter | Does it generalize? |
| 3    | Full scope | Medium | Tighter | Does coupling work? |
| 4    | Full scope | High | Target | Converged? |

The refine phase reads the spiral plan to scope tasks for the current pass. The DDV Red phase writes tests only for the current pass's scope subset. Acceptance criteria are staged — early passes have wider tolerances.

## Spiral Passes

### Pass 0 — Scoping (with Human Refinement Loop)

The only non-repeating pass. Establishes what we're solving, how we'll know we've succeeded, what methods to use, and how to stage the work. **No code is written.**

The scope phase includes an iterative human refinement loop: the human can approve, provide written feedback for the agent to incorporate, or edit scope artifacts directly. The R→re-run→review cycle can repeat as many times as needed. This is cheap — no code exists yet.

**Agent produces:**

1. **`methodology/methodology.md`** — Central methodology document: phenomenon, candidate methods, recommended approach, key equations, assumptions, valid range. Organized by topic/section if the problem has multiple aspects.

2. **`methodology/plan.md`** — Implementation plan with tasks for Pass 1.

3. **`methodology/verification-cases.md`** — L0 and L1 test specifications with expected values and sources. These will be turned into executable tests by the DDV Red phase.

4. **System-level files:**
   - `spiral/pass-0/acceptance-criteria.md` — Quantitative success targets
   - `spiral/pass-0/validation-strategy.md` — Limiting cases, reference data, conservation laws, integration tests
   - `spiral/pass-0/sanity-checks.md` — Engineering judgment checks
   - `spiral/pass-0/literature-survey.md` — Methods survey organized by topic/phenomenon
   - `spiral/pass-0/spiral-plan.md` — Scope + fidelity progression per pass
   - `methodology/overview.md` — System description, modeling approach
   - `methodology/assumptions-register.md` — Cross-cutting assumptions
   - `validation/` living documents — sanity checks, limiting cases, reference data

5. **`AGENTS.md`** — Resolved technology stack with concrete, verified commands.

**Human review:** Mandatory. Options: Approve, Refine (provide feedback → re-run), Edit (manual), Quit.

**Exit marker:** `spiral/pass-0/PASS_COMPLETE.md`

### Pass N (N >= 1) — Five-Phase Spiral

Each pass runs five phases in sequence:

| Phase | Agent | Writes code? | Key question |
|-------|-------|-------------|--------------|
| Refine | opus + subagents | No | What methodology, what plan, what tech decisions? |
| DDV Red | opus | Tests only | What should correct results look like? |
| Build | sonnet (Ralph loop) | Yes | Make the red tests green + software quality tests |
| Execute | opus | Glue/runner code only | Does the assembled system produce an answer? |
| Validate | opus | No | Does the answer converge? Pass V&V? |

**Phase 1: Refine (opus)**
- Reads spiral-plan.md for scope progression and updates methodology/plan for this pass
- Uses subagents for research delegation: literature search, code audit, validation review
- Updates methodology, plan, verification cases, cross-cutting documents
- Produces `spiral/pass-N/refine-summary.md`

**Phase 2: DDV Red (opus)**
- Writes failing domain verification tests from authoritative sources
- Does NOT read implementation code — independence is the core guarantee
- Reads spiral-plan.md: tests scoped to current pass's scope subset only
- Categorizes tests using the mechanism defined in AGENTS.md (e.g., markers, directories, naming)
- Produces `spiral/pass-N/ddv-red-manifest.md`

**Phase 3: Build (sonnet, Ralph loop)**
- Autonomous iterative implementation: pick task → implement → make DDV tests green → software tests
- CANNOT modify DDV tests. Disagreements → reconsideration file → next refine phase adjudicates
- Writes software quality tests alongside implementation
- Produces code in `src/`, tests in `tests/software/`

**Phase 4: Execute (opus)**
- Writes/updates integration code that chains modules together
- Runs the system end-to-end, produces actual answer to BRIEF.md
- Engineering judgment audit: dimensional consistency, order of magnitude, trends, conservation, bounds
- Produces `spiral/pass-N/execution-report.md`

**Phase 5: Validate (opus)**
- Runs all test suites, executes sanity checks, limiting cases, reference data comparisons
- Checks acceptance criteria, methodology compliance
- Reads spiral-plan.md for staged per-pass acceptance criteria (early passes have wider tolerances)
- Convergence assessment: compares with previous pass
- Produces: system-validation.md, convergence.md, review-package.md, PASS_COMPLETE.md

**Phase 6: Human Review Gate (mandatory)**
- Accept: produce final output
- Continue: next spiral pass
- Redirect: guidance for next pass (opens $EDITOR)

## Subagent Usage in Refine Phase

The refine phase uses Claude Code's Task tool to delegate focused research tasks, managing context without architectural complexity:

- **Literature subagent:** Search for methods, evaluate alternatives, save to references/retrieved/
- **Code audit subagent:** Audit src/ and tests/, report structure and interfaces
- **Validation review subagent:** Summarize previous pass's execution, validation, convergence

The Ralph loop (build phase) stays single-agent because it modifies shared state sequentially.

## Implementation Plan Format

```markdown
### Task N: [Short name]
- **Status:** TODO | IN_PROGRESS | DONE | BLOCKED
- **Pass:** N
- **Methodology:** [section ref]
- **Checklist:**
  - [ ] [Implement X]
  - [ ] [Implement Y]
  - [ ] [Derivation doc for Z (only if non-trivial)]
  - [ ] [Software tests for edge cases / error handling]
  - [ ] [Plot: description]
- **Dependencies:** [task refs or "None"]
```

Task sizing: completable in one Ralph iteration. Max 5 checklist items, split if larger.

Note: DDV test items are NOT in the plan. DDV tests are written by the DDV Red phase, independently.

## Reconsideration Protocol

If methodology doesn't work in practice:
1. Create `spiral/pass-N/reconsiderations/[issue].md`
2. Mark task BLOCKED
3. Next refine phase addresses it

**DDV Disagreement** (special case): If the build agent believes a DDV test encodes wrong domain knowledge:
1. Create `spiral/pass-N/reconsiderations/ddv-disagreement-[test-name].md`
2. Include: test expects, implementation produces, analysis citing the same source paper
3. Mark task BLOCKED
4. The next refine phase (opus) adjudicates

This is a feature, not a bug. Independent interpretation of papers will sometimes disagree.

### Resolution in Refine Phase

The refine phase at the start of the next pass is responsible for resolving all reconsiderations:
- DDV disagreements: adjudicate by returning to the source paper, update verification cases or methodology
- Methodology issues: evaluate and accept/reject the proposed alternative
- In all cases: unblock affected tasks in plan.md (change BLOCKED → TODO or create replacement tasks)

Reconsiderations must not accumulate across passes without resolution.

## Derivation Documentation Policy

Derivation documents are mandatory only when the mapping from equation to code is non-trivial: discretization of continuous equations, coordinate transforms, rearrangement for numerical stability, non-obvious unit conversions, or interpolation scheme choices. A direct algebraic transcription of a formula does not require a derivation doc.

## Human Interaction

### Scope Review Gate (After Pass 0)

```
═══════════════════════════════════════════════════════
  PASS 0 (SCOPING) COMPLETE — REVIEW REQUIRED
═══════════════════════════════════════════════════════

  Methodology:       methodology/methodology.md
  Plan:              methodology/plan.md
  Acceptance:        spiral/pass-0/acceptance-criteria.md
  Scope progression: spiral/pass-0/spiral-plan.md

  Stack: Python 3.11.5

  Scope progression:
    | 1 | 12 kn, calm water | Low | ±50% | Does it work? |
    | 2 | 5-25 kn, calm     | Med | ±20% | Generalize?   |

  [A] APPROVE  — proceed to Pass 1
  [R] REFINE   — provide feedback, re-run scope agent
  [E] EDIT     — I'll edit the files directly, then approve
  [Q] QUIT     — stop here
```

### Pass Review Gate (After Each Pass)

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

```
═══════════════════════════════════════════════════════
  BUILD BLOCKED
═══════════════════════════════════════════════════════

  Completed: 3 / 5 tasks
  Blocked:   2 tasks

  Blocked tasks:
    • Task 4: Wave resistance
      Reason: DDV test disagrees with implementation
    • Task 5: Form factor
      Reason: Depends on Task 4

  [F] FIX — resolve blocks, then resume build
  [S] SKIP — continue to next phase
  [X] ABORT — stop this spiral pass
```

## Review Package Format

```markdown
# Spiral Pass N — Review Package

## Current Answer
[The quantitative answer to BRIEF.md]

## Convergence: [CONVERGED / NOT YET / DIVERGING]
| Quantity | Δ from prev | Converged? |
|----------|------------|------------|
| [qty]    | [X.X%]     | [yes/no]   |

## Tests
DDV: [pass/total] | Software: [pass/total] | Integration: [pass/total]
Failures: [list any, or "None"]

## Sanity Checks: [pass/total]
Failures: [list any, or "None"]

## Engineering Judgment Issues (from Execution)
[list any, or "None"]

## Engineering Judgment (HUMAN REVIEW)
1. [Plot: path] → [what to look for]
2. [Key result] → [is this reasonable?]

## Recommendation
[ACCEPT / CONTINUE: reason / BLOCKED: reason]

## If Continuing — Proposed Refinements
- [What to change and why]

## Details
- Execution report: spiral/pass-N/execution-report.md
- Full validation: spiral/pass-N/system-validation.md
- Convergence: spiral/pass-N/convergence.md
- Plots: plots/REVIEW.md
```

## Final Output

When the human accepts:

1. **`output/answer.md`** — Direct response to the question in BRIEF.md.
2. **`output/report.md`** — Full development report: problem statement, methodology with citations, spiral history, V&V summaries, convergence tables, assumptions, limitations, traceability.

## Traceability Chain

```
BRIEF.md → acceptance criteria
  → scope (spiral/pass-0/) — human-refined scope, fidelity progression, validation strategy
    → methodology (methodology/methodology.md)
      → authoritative domain source (references/)
        → governing equations
          → DDV tests (tests/ddv/) — domain specification as executable tests
            → implementation (src/)
              → derivations (methodology/derivations/) — non-trivial mappings only
                → software tests (tests/software/) — edge cases, stability
                  → execution (src/runner) — end-to-end, engineering judgment audit
                    → system validation — sanity checks, convergence
                      → human acceptance
                        → final answer + report
```

## Configuration

**lisa.conf:**
```bash
# Claude Code model selection per phase
CLAUDE_MODEL_SCOPE="opus"
CLAUDE_MODEL_REFINE="opus"       # Methodology + plan refinement
CLAUDE_MODEL_DDV="opus"          # Domain-Driven Verification (test writing)
CLAUDE_MODEL_BUILD="sonnet"      # Implementation (Ralph loop)
CLAUDE_MODEL_EXECUTE="opus"      # System assembly + execution
CLAUDE_MODEL_VALIDATE="opus"     # System-level V&V and convergence

# Loop limits
MAX_SPIRAL_PASSES=5
MAX_RALPH_ITERATIONS=50

# Human review
NO_PAUSE=false

# Git
NO_PUSH=false

# Terminal
COLLAPSE_OUTPUT=true
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
├── BRIEF.md
├── AGENTS.md
│
├── prompts/
│   ├── PROMPT_scope.md
│   ├── PROMPT_refine.md
│   ├── PROMPT_ddv_red.md
│   ├── PROMPT_build.md
│   ├── PROMPT_execute.md
│   └── PROMPT_validate.md
│
├── methodology/
│   ├── methodology.md              # Single methodology document
│   ├── plan.md                     # Single implementation plan
│   ├── verification-cases.md       # Verification case specifications
│   ├── overview.md                 # System description
│   ├── assumptions-register.md     # Cross-cutting assumptions
│   └── derivations/                # Code ↔ equations mapping (non-trivial only)
│
├── spiral/
│   ├── current-state.md
│   ├── pass-0/
│   │   ├── acceptance-criteria.md
│   │   ├── validation-strategy.md
│   │   ├── sanity-checks.md
│   │   ├── literature-survey.md
│   │   ├── spiral-plan.md          # Scope + fidelity progression per pass
│   │   ├── scope-feedback.md       # Human feedback (created during scope refinement loop)
│   │   └── PASS_COMPLETE.md
│   └── pass-N/
│       ├── refine-summary.md
│       ├── ddv-red-manifest.md
│       ├── execution-report.md
│       ├── system-validation.md
│       ├── convergence.md
│       ├── review-package.md
│       ├── reconsiderations/
│       ├── human-redirect.md
│       └── PASS_COMPLETE.md
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
│
├── src/
├── tests/
│   ├── ddv/
│   ├── software/
│   └── integration/
│
└── output/
    ├── answer.md
    └── report.md
```

## Relationship to v1

| v1 Concept | v2 Evolution |
|-----------|-------------|
| BRIEF.md | Unchanged |
| Methodology phase | Distributed: Pass 0 (survey + scope) + per-pass Refine phases |
| Planning phase | Single plan in `methodology/plan.md`, created Pass 0, updated each Refine |
| Building phase | Ralph loop within each pass's Build phase |
| Review phase | Execute + Validate phases of each pass |
| Triage phase | Eliminated — validation/convergence failures drive next pass |
| IMPLEMENTATION_PLAN.md | Replaced by `methodology/plan.md` |
| Reconsideration protocol | Preserved and extended with DDV disagreement handling |
| Hierarchical verification | DDV tests (domain, red/green) + software tests + integration tests |
| methodology/coupling-strategy.md | Replaced by scope progression in spiral-plan.md |
| methodology/verification-cases.md | Preserved — DDV Red turns these into executable tests |
| METHODOLOGY_COMPLETE.md | No equivalent — methodology grows continuously |
| [BUILD_COMPLETE] | Replaced by SPIRAL_COMPLETE.md (human acceptance of convergence) |
| Agent-agnostic | Claude Code only |
