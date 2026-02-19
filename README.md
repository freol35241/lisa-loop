# Lisa Loop

Methodology-rigorous development loop for engineering and scientific software.

Lisa Loop extends the [Ralph Wiggum technique](https://ghuntley.com/ralph/) — a bash loop that repeatedly feeds a prompt to an AI coding agent — for domains where **methodological correctness** matters as much as functional correctness. Ship performance models, structural analysis, control systems, thermodynamics: a test can pass perfectly while implementing the wrong physics. Lisa adds what Ralph lacks:

1. A **methodology phase** that establishes the correct approach before any code is written, grounded in peer-reviewed literature
2. **Hierarchical verification** that catches physical regression across coupled subsystems
3. A **reconsideration protocol** that lets the implementation phase push back on methodology through a controlled channel

Named after Lisa Simpson — the rigorous counterpart to Ralph Wiggum.

## Quick Start

1. Click **"Use this template"** on GitHub to create your repo
2. Edit `BRIEF.md` with your project description
3. Add reference papers to `references/core/`
4. Edit `AGENTS.md` with your build/test commands
5. Run the loop:

```bash
chmod +x loop.sh

# Phase 1: Develop methodology (review & approve)
./loop.sh methodology

# Phase 2: Plan implementation
./loop.sh plan

# Phase 3: Build with verification
./loop.sh build

# Optional: Compliance audit
./loop.sh review

# Optional: Triage review findings
./loop.sh triage
```

## Phases

```
Phase 1: METHODOLOGY  →  methodology/*.md
                              ↓
Phase 2: PLANNING     →  IMPLEMENTATION_PLAN.md
                              ↓
Phase 3: BUILDING     →  src/ + tests/ + plots/ + derivations/
                              ↑
                    (methodology reconsideration)
                              ↓
         REVIEW       →  REVIEW_REPORT.md
                              ↓
         TRIAGE       →  TRIAGE_SUMMARY.md + reconsiderations + plan updates
                              ↓
                   ┌─────────────────────────┐
                   │ methodology findings?    │
                   │  YES → ./loop.sh methodology → plan → build │
                   │  NO  → ./loop.sh plan → build               │
                   └─────────────────────────┘
```

### Phase 1: Methodology

Each iteration, the agent reads your brief, existing methodology docs, and reference papers. It identifies the most important methodological gap and addresses it — specifying a method for an unaddressed phenomenon, resolving inconsistencies between subsystems, adding verification cases.

All method choices must trace to peer-reviewed sources. Equations must come from papers the agent has actually read. The agent must never fabricate equations from memory.

Terminates when the agent creates `METHODOLOGY_COMPLETE.md`. Human reviews and approves.

### Phase 2: Planning

Translates the methodology into an implementation plan with tasks covering code, derivation documentation, verification tests, and plots. Ordered bottom-up through the verification hierarchy.

### Phase 3: Building

Standard iterative building with three additions:

- **Methodology adherence** — code is checked against methodology specs after every implementation
- **Hierarchical verification** — changes at level N trigger re-verification at levels N through 3
- **Reconsideration protocol** — if the methodology doesn't work in practice, the agent raises a formal reconsideration instead of silently changing the approach

### Triage

After a review, triage categorizes every finding as either **METHODOLOGY** (wrong approach) or **IMPLEMENTATION** (wrong translation of approach to code). Methodology findings are routed to `methodology/reconsiderations/` for the methodology phase to resolve. Implementation findings are added as priority items at the top of `IMPLEMENTATION_PLAN.md`.

Triage tells you what to run next:
- If methodology reconsiderations exist: `./loop.sh methodology` first, then `plan`, then `build`
- If only implementation findings exist: `./loop.sh plan` and `./loop.sh build` can proceed directly

## Hierarchical Verification

```
Level 0: Individual functions     (equation → known output)
Level 1: Subsystem models         (correct over valid range)
Level 2: Coupled subsystem pairs  (sensible combined results)
Level 3: Full system              (known limiting cases)
```

Each level has automated tests (backpressure) and plots (human review). When anything at level N changes, all levels N through 3 re-run.

Verification cases are specified during the methodology phase, before code exists.

## Directory Structure

```
project-root/
├── loop.sh                         # The bash loop script
├── PROMPT_methodology.md           # Phase 1 prompt
├── PROMPT_plan.md                  # Phase 2 prompt
├── PROMPT_build.md                 # Phase 3 prompt
├── PROMPT_review.md                # One-shot review/audit prompt
├── PROMPT_triage.md                # Triage review findings prompt
├── BRIEF.md                        # Project description (you write this)
├── AGENTS.md                       # Build/test/plot commands (you write this)
├── IMPLEMENTATION_PLAN.md          # Generated by planning phase
├── methodology/
│   ├── overview.md
│   ├── [subsystem].md              # One per physical subsystem
│   ├── coupling-strategy.md
│   ├── assumptions-register.md
│   ├── verification-cases.md
│   └── reconsiderations/           # Formal methodology change requests
├── references/
│   ├── core/                       # Your papers (PDFs)
│   └── retrieved/                  # Papers found by the agent
├── derivations/                    # Code ↔ equations mapping
├── plots/
│   ├── REVIEW.md                   # Visual review summary
│   └── [subsystem]/
├── src/                            # Source code
└── tests/                          # Test suite
```

## Configuration

### Agent CLI

By default, Lisa Loop uses Claude Code. Override with environment variables:

```bash
# Use a different agent
AGENT_CMD=amp ./loop.sh build

# Custom arguments
AGENT_ARGS="-p --model sonnet" ./loop.sh methodology
```

### Other Options

```bash
# Skip git push after build iterations
NO_PUSH=1 ./loop.sh build

# Skip human review pauses (fully autonomous)
NO_PAUSE=1 ./loop.sh methodology

# Limit iterations
./loop.sh methodology 10
./loop.sh build 50
```

## Human Review Points

Lisa Loop pauses for human review at three points:

1. **Methodology complete** — review the full methodology before proceeding to planning
2. **Plots updated** — visual verification of model behavior after build iterations
3. **Reconsideration raised** — the agent found the methodology doesn't work and proposes a change

These are the primary human touchpoints. Engineers look at methodology docs and plots, not code.

## Reconsideration Protocol

During building, the agent might discover the specified methodology is problematic (doesn't converge, assumption too restrictive, outside valid range). Rather than silently changing the approach:

1. Creates `methodology/reconsiderations/[topic]-[issue].md` with evidence
2. Marks the task as BLOCKED
3. The loop pauses for human review

The human either approves the change (updates methodology) or rejects it with guidance. This is a feature: the implementation phase can push back on methodology through a controlled channel.

## Key Principles

1. **Methodology is a first-class artifact.** Version-controlled, reviewable, and the code is checked against it.
2. **The agent cannot silently make methodological choices.** It follows the spec or raises a formal reconsideration.
3. **Verification is derived from methodology.** Test cases are specified before code exists.
4. **Literature grounding is mandatory.** No method without a peer-reviewed source.
5. **Hierarchical verification catches physical regression.** Changes trigger re-verification up the tree.
6. **Visual verification for humans.** Plots are mandatory. Engineers spot wrong physics in plots faster than in code.
7. **Front-load methodology, automate implementation.** Human expertise validates the approach. The loop translates equations to code.
8. **Transparency over abstraction.** Raw bash loop, visible prompt files, no hidden state.
9. **Agent CLI agnostic.** Works with Claude Code, amp, codex, opencode, or any CLI that accepts piped prompts.

## Lineage

Lisa Loop extends the [Ralph Wiggum technique](https://ghuntley.com/ralph/) created by [Geoffrey Huntley](https://github.com/ghuntley/how-to-ralph-wiggum). Ralph is a bash loop that repeatedly feeds an AI agent a prompt, with filesystem persistence as shared state. Lisa adds methodology rigor, hierarchical verification, and visual human review for engineering/scientific software where correctness goes beyond passing tests.
