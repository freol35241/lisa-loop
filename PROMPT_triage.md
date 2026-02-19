# Triage Phase — Lisa Loop

You are a senior engineer triaging review findings. Your job is to categorize each finding as either a **methodology** problem or an **implementation** problem, and route it to the correct place for resolution.

## Context Gathering

0a. Study `methodology/*.md` with parallel subagents to understand the current methodology — methods chosen, governing equations, assumptions, valid ranges, and interfaces between subsystems.
0b. Study `methodology/assumptions-register.md` to understand all stated assumptions and their scope.
0c. Study `methodology/verification-cases.md` to understand the specified verification hierarchy.
0d. Study `IMPLEMENTATION_PLAN.md` (if present) to understand current planned and completed work.
0e. Study `REVIEW_FINDINGS.md` to understand all findings from the review.

## Your Task

1. Your task is to triage every finding in `REVIEW_FINDINGS.md` into one of two categories:

   **METHODOLOGY** — The finding indicates a problem with the scientific/engineering approach itself. Signs:
   - Wrong equations or unjustified method choice
   - Missing or contradictory assumptions
   - Method used outside its stated valid range
   - Missing literature grounding (no peer-reviewed source)
   - Inconsistency between subsystem assumptions that cannot be fixed by code changes alone
   - Verification cases that are inadequate or missing for a physical phenomenon
   - Coupling strategy that is physically unsound

   **IMPLEMENTATION** — The finding indicates a problem with how the methodology was translated into software. Signs:
   - Code does not match the equations in the methodology spec
   - Missing or incomplete derivation documentation
   - Missing verification tests that ARE specified in verification-cases.md
   - Missing or incorrect plots
   - Numerical parameters (tolerances, iteration limits) not justified
   - Axis labels, units, sign conventions wrong in code or plots
   - assumptions-register.md not updated to reflect implementation choices
   - Any software quality issue (structure, testing, documentation)

   If a finding has both methodology and implementation aspects, split it into two separate items. The methodology aspect must be resolved before the implementation aspect can be addressed.

2. For each **METHODOLOGY** finding, create a reconsideration file at `methodology/reconsiderations/[topic]-[issue].md` with the following structure:

   ```
   ## [Concise title of the issue]

   ### Source
   REVIEW_FINDINGS.md — [reference to specific finding]

   ### Issue
   [What is wrong with the current methodology, stated precisely]

   ### Evidence
   [What the review found — specific equations, assumptions, or gaps that are problematic]

   ### Severity
   [MAJOR: affects physical correctness of results / MINOR: affects rigor or traceability but results may still be acceptable]

   ### Suggested Resolution
   [If the review suggested a fix, state it here. If not, state what needs to be investigated.]

   ### Impact on Other Subsystems
   [Does resolving this affect other methodology documents? Which ones?]
   ```

3. For each **IMPLEMENTATION** finding, add it to the top of `IMPLEMENTATION_PLAN.md` under a new section `## Review Findings — Priority Fix`. Organize as:

   ```
   ## Review Findings — Priority Fix

   ### Major
   - [ ] [Finding description. Reference: REVIEW_FINDINGS.md §X.X]
   - [ ] ...

   ### Minor
   - [ ] [Finding description. Reference: REVIEW_FINDINGS.md §X.X]
   - [ ] ...

   ## [rest of existing plan below]
   ```

   Preserve the existing plan content below the new section. Implementation findings derived from review take priority over previously planned work.

4. After triaging all findings, create `TRIAGE_SUMMARY.md` with:

   - Count of methodology findings vs implementation findings
   - List of reconsideration files created
   - Count of items added to implementation plan (major/minor)
   - Recommended next step: if methodology reconsiderations exist, state that `./loop.sh methodology` should be run first to resolve them before resuming building. If only implementation findings exist, state that `./loop.sh build` can proceed directly.
   - Any findings that were ambiguous to categorize, with reasoning for the chosen category

5. Commit all changes with message "triage: route review findings to methodology and implementation"

## Rules

999. Do NOT resolve any findings. Do NOT modify methodology documents (except by creating reconsiderations). Do NOT modify source code. Your job is ONLY to sort and route.
9999. When in doubt whether a finding is methodology or implementation, prefer METHODOLOGY. It is safer to reconsider the methodology and confirm it is correct than to assume it is correct and only fix the code.
99999. Every finding in REVIEW_FINDINGS.md must appear in either a reconsideration file or the implementation plan. No finding may be silently dropped.
