# Lisa Loop

Without structure, AI agents vibe-code. They write code and tests from the same understanding, so bugs and tests share the same blind spots. There's no audit trail, no review gate, no way to tell if the answer is trustworthy or just plausible.

Lisa is a CLI tool that orchestrates [Claude Code](https://docs.anthropic.com/en/docs/claude-code) through a structured engineering process — scoping before coding, engineering judgment skills, iterative refinement, and human review at every stage.

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
  │  SPIRAL PASSES (Pass 1..N)                    │
  │                                               │
  │  ┌─────────┐  ┌─────────┐  ┌────────┐        │
  │  │ REFINE  │→ │  BUILD  │→ │ AUDIT  │        │
  │  └─────────┘  └─────────┘  └────────┘        │
  │                                               │
  │  Each pass widens scope & tightens tolerances │◄── human review gate
  └──────────────────┬────────────────────────────┘
                     ▼
  Finalize at review gate → LISA-REPORT.md
```

**Scope** locks down the methodology, acceptance criteria, and a staged plan — before any code is written. A human reviews and refines until satisfied.

**Spiral Passes** iterate through Refine → Build → Audit. Each pass increases fidelity: early passes get the structure right, later passes tighten tolerances and handle edge cases. After every pass, the human decides: **finalize**, **continue** to the next pass, **redirect** with guidance, or **explore** an alternative on a side-branch.

## Grounded in Engineering Practice

Lisa combines two established frameworks, adapted for AI-assisted work.

**The V-Model** says: define what "correct" means before you write code, then validate against those pre-defined criteria. In Lisa, the left arm of the V is the scoping phase — acceptance criteria, sanity checks, limiting cases, and reference data are defined before any code is written. The right arm is the Audit phase, which validates the implementation against those pre-defined criteria. Engineering judgment skills (bounding tests, dimensional analysis, numerical stability, literature grounding) ensure verification is rigorous.

**The Design Spiral** says: iterate with increasing fidelity, with human control at each turn. Lisa's spiral passes do exactly this — each pass refines the approach, builds with more detail, and audits against tighter standards. The human reviews every pass and controls when to stop.

## Engineering Judgment Skills

When AI agents write code and tests from the same understanding, bugs and tests share the same blind spots. Lisa addresses this by equipping agents with structured engineering judgment skills:

- **Three-level bounding** — phenomenon bounds (first-principles), composition bounds (between components), and system bounds (independent cross-checks)
- **Dimensional analysis** — unit tracking through computation chains catches implicit conversion errors
- **Numerical stability** — convergence criteria, condition numbers, floating-point awareness
- **Literature grounding** — reference data comparison with source verification and condition matching

These skills are embedded as markdown files in `.lisa/skills/` and injected into agent prompts during the Scope, Build, and Audit phases. They define *how* agents should verify their work, not just *what* to verify.

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
lisa run                     # Full spiral: scope → passes → finalize
lisa run --max-passes 3      # Limit spiral passes
lisa run --follow-up "..."   # Continue after finalization with a new question
lisa run --no-pause          # Skip all human review gates (autonomous)
lisa resume                  # Resume from saved state
lisa status                  # Print current spiral state and pass history
lisa rollback <pass>         # Roll back to a pass boundary
lisa eject-prompts           # Copy prompts to .lisa/prompts/ for customization
lisa doctor                  # Check environment
```

Configuration lives in `lisa.toml` (project root). Models, limits, review gates, paths, and commands. Run `lisa init` to see the full default config with comments.

## Human Interaction

### Pass Review Gate

```
═══════════════════════════════════════════════════════
  SPIRAL PASS N COMPLETE — REVIEW REQUIRED
═══════════════════════════════════════════════════════

  Answer:      142.3 kN total resistance
  Progress:    Δ 12% from prev
  Tests:       Bounds: 8/8 | Software: 15/15 | Integration: 2/2
  Agent recommends: CONTINUE

  [F] FINALIZE — results are satisfactory, produce the final report
  [C] CONTINUE — run another spiral pass to improve results
  [R] REDIRECT — write guidance to steer the next pass
  [E] EXPLORE  — create a side-branch to investigate an alternative
  [Q] QUIT     — stop the spiral here (resume later)
```

### Scope Review Gate

After Pass 0, review methodology and acceptance criteria before any code is written. Options: **Approve**, **Refine** (agent re-runs with feedback), **Edit** (modify files directly), or **Quit**.

## Credits

Lisa Loop extends the [Ralph Wiggum technique](https://ghuntley.com/ralph/) by [Geoffrey Huntley](https://github.com/ghuntley/how-to-ralph-wiggum) — an approach to iterative AI coding where an outer agent loop drives Claude Code through repeated build-test cycles. Lisa adds scoping, engineering judgment skills, human review gates, and structured methodology on top of that foundation. Named after Lisa Simpson, the rigorous counterpart to Ralph Wiggum.
