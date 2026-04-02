# Implementation Plan Spec — `{{lisa_root}}/methodology/plan.md`

Initial implementation plan with task structure for Pass 1. At this stage you know *what*
needs to be implemented and in what order, but the equations are not yet fully specified —
that detail comes from the refine phase. Keep this plan at the structural level: task names,
ordering, methodology references, and dependencies. Do NOT write detailed checklists — the
refine phase will flesh those out once the methodology is complete.

## Template

```markdown
# Implementation Plan

## Tasks

### Task 1: [Short name]
- **Status:** TODO
- **Pass:** 1
- **Methodology:** [section ref]
- **Dependencies:** [task refs or "None"]

### Task 2: [Short name]
- **Status:** TODO
- **Pass:** 1
- **Methodology:** [section ref]
- **Dependencies:** [task refs or "None"]
```

## Task Rules

- Order tasks bottom-up: utilities -> core equations -> higher-level models -> integration
- Each task should be scoped for a single Ralph iteration
- Infrastructure tasks (setup, test framework, etc.) come first if needed
- Tag every task with `**Pass:** 1` for Pass 1 tasks
- Pass 2+ tasks can be sketched with TODO placeholders
- Tasks should include bounding check items where applicable (the Build phase derives and writes bounding tests per the engineering judgment skill)
- Do NOT add checklists — the refine phase adds those after completing the methodology
