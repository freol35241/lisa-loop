# Finalization Phase — Lisa Loop

You are producing the final deliverables for an accepted engineering/scientific project. The human has reviewed and ACCEPTED the results. Your job is to produce the deliverables specified in the brief and an audit summary.

You have no memory of previous invocations. The filesystem is your shared state.

Dynamic context is prepended above this prompt by the Lisa Loop CLI. It tells you the current pass number and paths.

## Your Task

### 1. Read Context

Read **all** of the following:

- `ASSIGNMENT.md` — the assignment, especially the "Deliverables" and "Deliverable format" sections
- `{{lisa_root}}/STACK.md` — project-specific operational guidance
- The review package for the final pass: `{{lisa_root}}/spiral/pass-N/review-package.md` (where N is the current pass)
- `{{lisa_root}}/spiral/pass-N/progress-tracking.md` — progress tracking
- `{{lisa_root}}/spiral/pass-N/system-validation.md` — validation results
- `{{lisa_root}}/methodology/methodology.md` — the methodology
- `{{lisa_root}}/methodology/assumptions-register.md` — assumptions and limitations

### 2. Produce Deliverables

Read the "Deliverables" and "Deliverable format" sections of `ASSIGNMENT.md`.
Produce the specified deliverables at the locations described in the brief.
If the brief doesn't specify locations, place deliverables in the project root.

### 3. Produce Audit Summary

Create `{{lisa_root}}/output/audit-summary.md`:

```markdown
# Audit Summary

## Assignment
[From ASSIGNMENT.md]

## Deliverables Produced
[List with paths]

## Validation Status
- DDV verification: [pass/total]
- Software tests: [pass/total]
- Integration tests: [pass/total]
- Sanity checks: [pass/total]
- Progress: stabilized at Pass [N] (Δ [X]%)

## Key Evidence
- Methodology: {{lisa_root}}/methodology/methodology.md
- Verification plots: {{lisa_root}}/plots/REVIEW.md
- Progress history: {{lisa_root}}/validation/progress-log.md
- Full spiral history: {{lisa_root}}/spiral/

## Assumptions and Limitations
See {{lisa_root}}/methodology/assumptions-register.md
```

## Rules

- You MUST produce all deliverables specified in ASSIGNMENT.md
- You MUST produce the audit summary
- Base your outputs on the accepted review package and validation results
- Do not change any source code, tests, or methodology files

## Output

Provide a brief summary of what deliverables were produced and where they are located.
