# Lisa Loop v2

A methodology and toolbox for solving complex engineering and research problems with AI agents, grounded in peer-reviewed literature, with explicit verification, validation, and convergence tracking.

Lisa Loop v2 fuses two established engineering paradigms:

- **The V-Model** (systems engineering): every level of specification is paired with a corresponding level of verification and validation. V&V criteria are defined *before* implementation, not after.
- **The Design Spiral** (Evans, 1959): the same problem is revisited iteratively at increasing fidelity and scope until the answer converges.

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

1. **`output/answer.md`** — Direct response to the question in BRIEF.md
2. **`output/report.md`** — Full development report: problem statement, methodology with citations, spiral history, V&V summaries, convergence tables, assumptions, limitations, traceability

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

All configuration is in `lisa.conf`:

```bash
# Model selection per phase
CLAUDE_MODEL_SCOPE="opus"        # Pass 0 scoping
CLAUDE_MODEL_REFINE="opus"       # Methodology + plan refinement
CLAUDE_MODEL_DDV="opus"          # Domain-Driven Verification (test writing)
CLAUDE_MODEL_BUILD="sonnet"      # Implementation (Ralph loop)
CLAUDE_MODEL_EXECUTE="opus"      # System assembly + execution
CLAUDE_MODEL_VALIDATE="opus"     # System-level V&V and convergence

# Loop limits
MAX_SPIRAL_PASSES=5              # Max spiral passes
MAX_RALPH_ITERATIONS=50          # Max build iterations per pass

# Human review
NO_PAUSE=false                   # Skip all human review?

# Git
NO_PUSH=false                    # Skip git push?

# Terminal
COLLAPSE_OUTPUT=true             # Collapse agent output after completion?
```

## Directory Structure

```
project-root/
├── loop.sh
├── lisa.conf
├── BRIEF.md                            # Project description (user writes)
├── AGENTS.md                           # Build/test/plot commands (user writes)
│
├── prompts/
│   ├── PROMPT_scope.md                 # Pass 0: scoping + methodology
│   ├── PROMPT_refine.md                # Per-pass methodology + plan refinement
│   ├── PROMPT_ddv_red.md               # Domain-Driven Verification (test writing)
│   ├── PROMPT_build.md                 # Ralph loop implementation
│   ├── PROMPT_execute.md               # System assembly + execution
│   └── PROMPT_validate.md              # System-level V&V + convergence
│
├── methodology/
│   ├── methodology.md                  # Single methodology document
│   ├── plan.md                         # Single implementation plan
│   ├── verification-cases.md           # L0/L1 test specifications
│   ├── overview.md                     # System description
│   ├── assumptions-register.md         # Cross-cutting assumptions
│   └── derivations/                    # Code ↔ equations mapping
│
├── spiral/
│   ├── current-state.md
│   ├── pass-0/
│   │   ├── acceptance-criteria.md
│   │   ├── validation-strategy.md
│   │   ├── sanity-checks.md
│   │   ├── literature-survey.md
│   │   ├── spiral-plan.md              # Scope + fidelity progression
│   │   └── PASS_COMPLETE.md
│   └── pass-N/
│       ├── refine-summary.md
│       ├── ddv-red-manifest.md
│       ├── execution-report.md
│       ├── system-validation.md
│       ├── convergence.md
│       ├── review-package.md
│       ├── reconsiderations/
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
│   ├── ddv/                            # Domain-Driven Verification tests
│   ├── software/                       # Software quality tests
│   └── integration/                    # End-to-end tests
│
└── output/
    ├── answer.md
    └── report.md
```

## Lineage

Lisa Loop extends the [Ralph Wiggum technique](https://ghuntley.com/ralph/) created by [Geoffrey Huntley](https://github.com/ghuntley/how-to-ralph-wiggum) — a bash loop that feeds prompts to an AI agent with filesystem persistence as shared state.

**Lisa Loop v1** added methodology rigor, hierarchical verification, and a reconsideration protocol for engineering/scientific software where "passing tests" is necessary but insufficient — the tests themselves might encode wrong physics.

**Lisa Loop v2** restructures the process into a five-phase spiral architecture with Domain-Driven Verification (DDV): the scoping phase establishes methodology and scope progression with a human refinement loop, and each spiral pass runs Refine → DDV Red → Build → Execute → Validate with staged acceptance criteria. DDV provides a domain-agnostic verification pattern where two independent agents interpret the same authoritative sources, preventing correlated misinterpretation. The spiral history — not just the final answer — is the deliverable.

Named after Lisa Simpson — the rigorous counterpart to Ralph Wiggum.
