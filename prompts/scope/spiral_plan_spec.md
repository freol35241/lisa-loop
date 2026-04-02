# Spiral Plan Spec — `{{lisa_root}}/spiral/pass-0/spiral-plan.md`

The spiral plan defines how scope and fidelity increase per pass using **diagonal scoping**:
each pass targets one phenomenon at one fidelity level — narrow on both dimensions simultaneously.
This produces small, independently checkable passes of {{max_tasks_per_pass}} tasks or fewer.
If a pass would require more than {{max_tasks_per_pass}} tasks, it is too broad, too deep, or both —
split it.

**Calibrate the spiral plan to the human's approach preference** from `ASSIGNMENT.md`:
- If they want simplicity/minimum viable: plan fewer passes, stay with one method, widen
  tolerances. The spiral may converge in 1-2 passes.
- If they want state of the art: plan for progressive method upgrades across passes,
  tighter final tolerances, more validation.
- If they specified a particular method: build the spiral around that method's scope
  progression (narrow to broad), not around method upgrades.
- If no preference stated: use balanced judgment.

## Template

```markdown
# Spiral Plan

## Approach Philosophy
[Summarize the human's approach preference from ASSIGNMENT.md, or state "balanced (default)"
if none was given. Note any tension between the requested approach and the acceptance criteria.]

## Scope Progression

| Pass | Scope subset | Fidelity | Acceptance (this pass) | Key question |
|------|-------------|----------|----------------------|--------------|
| 1    | [subset]    | [level]  | [+/-X%]              | Does the approach work at all? |
| 2    | [broader]   | [level]  | [+/-X%]              | Does it generalize across range? |
| 3    | [full]      | [level]  | [+/-X%]              | Does coupling work? |
| 4    | [full]      | [refined]| [+/-X%]              | Converged? |

## Progress Tracking Expectations
[What quantities to track across passes, expected rate of change]

## Risk Areas
[Where methodology might need reconsideration, known difficult aspects]
```

## Example: Ship resistance (5-25 kn, sea states 1-6)

**Balanced approach:**
- Pass 1: 12 kn, calm water, simplest method -> +/-50%
- Pass 2: 5-25 kn, calm water, add corrections -> +/-20%
- Pass 3: Full range + sea states 1-3 -> +/-10%
- Pass 4: Full scope, refined methods -> +/-5%

**Minimum viable approach:**
- Pass 1: Full speed range, calm water, Holtrop-Mennen -> +/-15%
- Pass 2: Add sea state corrections -> +/-10%
(Fewer passes, simpler method, acceptance tolerances widened to match method capability)

## How the spiral plan is used

The refine phase reads this plan to scope tasks for the current pass.
The Audit phase checks bounding test discipline and coverage for the current pass's scope subset.
Acceptance criteria are staged — early passes have wider tolerances.
