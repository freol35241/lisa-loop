# Lisa Loop

Without structure, AI agents vibe-code. They write code and tests from the same understanding, so bugs and tests share the same blind spots. There's no audit trail, no review gate, no way to tell if the answer is trustworthy or just plausible.

Lisa is a CLI tool that orchestrates [Claude Code](https://docs.anthropic.com/en/docs/claude-code) through a structured engineering process — scoping before coding, independent verification, iterative refinement, and human review at every stage.

## How It Works

```
  lisa init → scaffold .lisa/ + ASSIGNMENT.md
                    │
                    ▼
  ┌───────────────────────────────────────────────┐
  │  SCOPE (Pass 0)                               │
  │  Define methods, acceptance criteria, staged   │
  │  plan — no code yet                           │◄── human refine loop
  └──────────────────┬────────────────────────────┘
                     ▼
  ┌───────────────────────────────────────────────┐
  │  INDEPENDENT VERIFICATION (one-time prologue) │
  │  Test criteria from literature, before code    │
  └──────────────────┬────────────────────────────┘
                     ▼
  ┌───────────────────────────────────────────────┐
  │  SPIRAL PASSES (Pass 1..N)                    │
  │                                               │
  │  ┌─────────┐  ┌─────────┐  ┌────────┐        │
  │  │ REFINE  │→ │  BUILD  │→ │VALIDATE│        │
  │  └─────────┘  └─────────┘  └────────┘        │
  │                                               │
  │  Each pass widens scope & tightens tolerances │◄── human review gate
  └──────────────────┬────────────────────────────┘
                     ▼
  lisa finalize → answer.md + report.md
```

**Scope** locks down the methodology, acceptance criteria, and a staged plan — before any code is written. A human reviews and refines until satisfied.

**Independent Verification** runs once after scoping. A separate agent reads reference literature and writes verification scenarios (markdown, not code) that define what "correct" looks like — independently from the agent that will write the implementation.

**Spiral Passes** iterate through Refine → Build → Validate. Each pass increases fidelity: early passes get the structure right, later passes tighten tolerances and handle edge cases. After every pass, the human decides: **accept**, **continue** to the next pass, or **redirect** with guidance.

## Grounded in Engineering Practice

Lisa combines two established frameworks, adapted for AI-assisted work.

**The V-Model** says: define what "correct" means before you write code, then validate against those pre-defined criteria. In Lisa, the left arm of the V is the scoping phase (acceptance criteria) and the independent verification prologue (test scenarios from literature). The right arm is the Validate phase, which executes those pre-defined scenarios against the implementation. The criteria exist before the code does — so they can't be shaped by implementation assumptions.

**The Design Spiral** says: iterate with increasing fidelity, with human control at each turn. Lisa's spiral passes do exactly this — each pass refines the approach, builds with more detail, and validates against tighter standards. The human reviews every pass and controls when to stop.

## Independent Verification

When the same agent writes both code and tests, errors correlate — the tests pass because they share the code's assumptions, not because the code is correct.

Lisa breaks this correlation by separating three roles:

```
  DDV Agent (opus)          Build (sonnet)          Validate (opus)
  ┌───────────────┐        ┌───────────────┐       ┌───────────────┐
  │ Read papers    │        │ Read scenarios │       │ Read scenarios │
  │ Write scenarios│───────→│ Write code     │──────→│ Write tests    │
  │ (no code)      │        │ (no DDV tests) │       │ Run all tests  │
  └───────────────┘        └───────────────┘       └───────────────┘
    ▲ independent              ▲ separated              ▲ verified
```

The **DDV Agent** reads authoritative sources and writes verification scenarios — expected values, edge cases, domain constraints — as markdown. The **Builder** writes implementation code but never touches the verification tests. The **Validator** converts scenarios into executable tests and runs the full suite.

Disagreements between the DDV scenarios and the implementation are signals, not bugs to suppress. They surface misunderstandings that would otherwise hide until production.

This works for any domain with authoritative sources and testable expected values: physics, econometrics, regulatory standards, engineering benchmarks.

## Cornerstones

1. **Traceable methodology.** Every methodological choice traces to an authoritative source. No equation without a paper. This makes results defensible and reviewable — not just plausible.

2. **Auditable engineering judgment.** Assumptions, sanity checks, and parameter choices are written down, versioned, and executed — not left as implicit decisions buried in code comments.

3. **Visual evidence for review.** Plots, comparison charts, and diagrams surface problems that tables and numbers hide. Visual artifacts are the preferred way to present results for human review.

4. **The history is the deliverable.** The refinement record across spiral passes — not just the final answer — is evidence of trustworthiness. Every pass is preserved as a complete record of how the solution evolved.

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

## Credits

Lisa Loop extends the [Ralph Wiggum technique](https://ghuntley.com/ralph/) by [Geoffrey Huntley](https://github.com/ghuntley/how-to-ralph-wiggum) — an approach to iterative AI coding where an outer agent loop drives Claude Code through repeated build-test cycles. Lisa adds scoping, independent verification, human review gates, and structured methodology on top of that foundation. Named after Lisa Simpson, the rigorous counterpart to Ralph Wiggum.
