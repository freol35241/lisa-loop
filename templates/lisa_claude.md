# CLAUDE.md — Lisa Loop Artifact Guide

This directory (`.lisa/`) contains all process artifacts from a Lisa Loop run.
Use this guide to find and interpret results, methods, assumptions, and validation evidence.

## Quick Start

- **What was the assignment?** Read `../ASSIGNMENT.md`
- **What's the current status?** Read `state.toml`
- **What methodology was chosen?** Read `methodology/methodology.md`
- **What are the results?** Read `spiral/pass-N/review-package.md` (latest pass)
- **Did the results pass validation?** Read `spiral/pass-N/system-validation.md`

## Artifact Map

### Configuration & State

| File | Purpose |
|------|---------|
| `../lisa.toml` | Project configuration (models, limits, review gates, paths, commands) |
| `state.toml` | Current spiral state machine position |
| `CODEBASE.md` | Auto-discovered project structure summary |
| `STACK.md` | Resolved technology stack and build/test commands |

### Methodology (refined each pass)

| File | Purpose |
|------|---------|
| `methodology/methodology.md` | Full method specification: governing equations, assumptions, valid range |
| `methodology/plan.md` | Task breakdown with status (TODO/IN_PROGRESS/DONE/BLOCKED) per pass |
| `methodology/assumptions-register.md` | Explicit assumptions and known limitations |
| `methodology/derivations/*.md` | Non-trivial derivations (discretizations, transforms, numerical schemes) |

### Validation & Verification

| File | Purpose |
|------|---------|
| `ddv/scenarios.md` | Domain-Driven Verification scenarios with literature sources and tolerances |
| `validation/sanity-checks.md` | Order-of-magnitude, trend, conservation, and dimensional checks |
| `validation/limiting-cases.md` | Analytical limiting cases to verify |
| `validation/reference-data.md` | Published reference datasets for comparison |

### Per-Pass Artifacts (`spiral/pass-N/`)

| File | Purpose |
|------|---------|
| `acceptance-criteria.md` | (Pass 0 only) Final acceptance targets |
| `literature-survey.md` | (Pass 0 only) Method candidates surveyed |
| `spiral-plan.md` | (Pass 0 only) Scope progression strategy across passes |
| `refine-summary.md` | What methodology changed this pass |
| `execution-report.md` | Intermediate values and outputs from Build |
| `system-validation.md` | Full validation results: test counts, sanity checks, DDV status |
| `progress-tracking.md` | Cross-pass convergence metrics and coverage |
| `review-package.md` | Human-facing summary with key results and recommendations |
| `plots/REVIEW.md` | Index of all plots with descriptions and assessments |
| `plots/*.png` | Visual evidence (DDV comparisons, convergence, reference data) |
| `reconsiderations/*.md` | Disagreements between DDV tests and implementation, pending adjudication |
| `code-diff.patch` | Code changes vs. previous pass |
| `PASS_COMPLETE.md` | Marker indicating this pass finished |

### Output

| File | Purpose |
|------|---------|
| `output/audit-summary.md` | Final audit: deliverables produced, validation status, evidence trail |

### References

| Directory | Purpose |
|-----------|---------|
| `references/core/` | User-supplied reference papers and data |
| `references/retrieved/` | Agent-retrieved reference summaries |

## How to Read Results

1. Check `state.toml` to see which pass completed last
2. Read `spiral/pass-N/review-package.md` for the high-level summary
3. Read `spiral/pass-N/system-validation.md` for detailed test results
4. Check `spiral/pass-N/plots/REVIEW.md` for visual evidence
5. Read `spiral/pass-N/progress-tracking.md` for convergence across passes
6. Read `methodology/assumptions-register.md` for caveats and limitations

## How to Interrogate Methods

- The governing equations and their sources are in `methodology/methodology.md`
- Each non-trivial derivation is documented in `methodology/derivations/`
- DDV scenarios in `ddv/scenarios.md` trace back to literature with full citations
- The `literature-survey.md` in pass-0 shows what alternatives were considered and why they were rejected

## How to Assess Trustworthiness

- Check DDV scenario pass rates in `ddv/scenarios.md` (Manifest table)
- Review sanity checks in `validation/sanity-checks.md`
- Look for reconsiderations in `spiral/pass-N/reconsiderations/` — these flag unresolved disagreements
- Compare convergence across passes in `progress-tracking.md`
- Read `methodology/assumptions-register.md` for known limitations
