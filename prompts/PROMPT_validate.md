# Validation Phase — Lisa Loop v2

You are a senior engineer conducting system-level verification, validation, and convergence
assessment. The system has been executed and produced an answer. Your job is to evaluate that
answer rigorously and determine if it has converged.

You have no memory of previous invocations. The filesystem is your shared state.

Dynamic context is prepended above this prompt by loop.sh. It tells you the current pass number.

## Your Task

### 1. Read Context

Read **all** of the following:

- `BRIEF.md` — project goals
- `AGENTS.md` — build/test/plot commands
- `methodology/methodology.md` — the methodology
- `spiral/pass-0/acceptance-criteria.md` — what success looks like
- `spiral/pass-0/sanity-checks.md` — engineering judgment checks
- `spiral/pass-0/validation-strategy.md` — validation approach
- `spiral/pass-N/execution-report.md` — this pass's execution results and engineering judgment audit
- `spiral/pass-N/ddv-red-manifest.md` — DDV test manifest for this pass
- `validation/sanity-checks.md` — living sanity check document
- `validation/limiting-cases.md` — limiting cases to check
- `validation/reference-data.md` — reference data to compare against
- `validation/convergence-log.md` — convergence history
- `plots/REVIEW.md` — current plot assessments

If this is **Pass N > 1**:
- Read `spiral/pass-{N-1}/convergence.md` — previous convergence assessment
- Read `spiral/pass-{N-1}/system-validation.md` — previous validation report

### 2. Test Results Summary

Collect test results:
- **DDV tests:** Run the DDV test suite. Record pass/fail counts.
- **Software tests:** Run the software test suite. Record pass/fail counts.
- **Integration tests:** Run integration tests. Record pass/fail counts.

### 3. Validation Checks

#### 3a. Sanity Checks

Execute every check in `validation/sanity-checks.md`:

- **Order of magnitude:** Are results in the expected ballpark?
- **Expected trends:** When parameters change, do outputs move in the expected direction?
- **Physical bounds:** Are all outputs within physically possible ranges?
- **Conservation:** Are conserved quantities preserved to within tolerance?
- **Dimensional analysis:** Do all outputs have correct dimensions/units?
- **Red flags:** Are any red-flag conditions triggered?

Record each check as PASS or FAIL with the actual value observed.

#### 3b. Limiting Cases

Check limiting cases from `validation/limiting-cases.md`:
- When parameters go to extreme values, do results match known analytical solutions?

#### 3c. Reference Data

Compare against reference data from `validation/reference-data.md`:
- How do results compare to published experimental or computational data?

#### 3d. Acceptance Criteria

Check each criterion from `spiral/pass-0/acceptance-criteria.md`:
- Is the criterion met? If not, how far off?

### 4. Review Execution Report

Review the engineering judgment audit results from `spiral/pass-N/execution-report.md`:
- Are all checks OK?
- Any flagged items require deeper investigation?

### 5. Methodology Compliance Spot-Check

Sample key equations: does the code match the methodology?
- Are assumptions respected?
- Are valid ranges enforced?
- Are derivation docs present for non-trivial mappings?

### 6. Convergence Assessment

Compare key outputs with the previous spiral pass.

If this is **Pass 1:** No previous pass to compare. Establish baseline values. Note convergence assessment is not possible yet.

If this is **Pass N > 1:**
- Read `spiral/pass-{N-1}/convergence.md` for previous values
- For each key output quantity:
  - Compute absolute and relative change from previous pass
  - Assess whether the change is within the accuracy bounds of the methods used
  - Determine if the quantity has converged

Overall convergence assessment:
- **CONVERGED:** All key quantities have stabilized within method accuracy
- **NOT YET CONVERGED:** Some quantities are still changing significantly
- **DIVERGING:** Quantities are moving further from expected values (indicates a problem)

### 7. Produce Artifacts

Create **all** of the following:

#### `spiral/pass-N/system-validation.md`

Detailed validation report. Be concise: one line per passing check, detailed analysis only for failures.

```markdown
# Spiral Pass N — System Validation Report

## Verification

### Test Results
- DDV tests: [pass/total]
- Software tests: [pass/total]
- Integration tests: [pass/total]

### Failures
[For each failing test:]
- **[Test name]:** Expected [X], got [Y]. [Analysis of why.]

### Methodology Compliance
[Results of spot-check. Issues found, if any.]

### Derivation Completeness
[Gaps found, if any.]

## Validation

### Sanity Checks
| Check | Expected | Actual | Status |
|-------|----------|--------|--------|
| [check] | [value] | [value] | PASS/FAIL |

### Limiting Cases
| Case | Expected | Actual | Status |
|------|----------|--------|--------|
| [case] | [value] | [value] | PASS/FAIL |

### Reference Data Comparison
| Dataset | Source | Our Result | Published | Δ (%) | Status |
|---------|--------|-----------|-----------|-------|--------|
| [data] | [cite] | [value] | [value] | [X.X] | PASS/FAIL |

### Acceptance Criteria
| Criterion | Target | Current | Met? |
|-----------|--------|---------|------|
| [criterion] | [target] | [value] | YES/NO |
```

#### `spiral/pass-N/convergence.md`

```markdown
# Spiral Pass N — Convergence Assessment

## Key Quantities
| Quantity | Pass N-1 | Pass N | Δ (abs) | Δ (%) | Converged? |
|----------|---------|--------|---------|-------|------------|
| [qty 1] | [value] | [value] | [value] | [X.X] | [yes/no] |

## Overall Assessment
[CONVERGED / NOT YET CONVERGED / DIVERGING]

## Analysis
[Why quantities have/haven't converged. What is driving remaining changes.]

## Recommendation
[ACCEPT: answer has converged / CONTINUE: refine X because Y / BLOCKED: need Z]
```

#### `spiral/pass-N/review-package.md`

This is the primary artifact for human review. Use this **exact format**:

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
These checks require domain expertise and first-principles reasoning:
1. [Plot: path] → [what to look for]
2. [Key result] → [is this reasonable? dimensional analysis, conservation, order-of-magnitude]

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

#### Update `validation/convergence-log.md`

Append this pass's convergence data to the cumulative log.

#### Update `plots/REVIEW.md`

Ensure all plots have current assessments reflecting this pass's results.

#### `spiral/pass-N/PASS_COMPLETE.md`

Create this file **last**:

```markdown
# Pass N — Complete

Verification: DDV [pass/total], Software [pass/total], Integration [pass/total]
Validation: [X/Y sanity checks passing]
Convergence: [CONVERGED / NOT YET CONVERGED / DIVERGING]
Agent recommendation: [ACCEPT / CONTINUE / BLOCKED]
```

#### Final Output (if convergence achieved)

If convergence is achieved and you recommend ACCEPT, also draft:

- `output/answer.md` — Direct response to the question in `BRIEF.md`. Brief, specific, quantitative.
- `output/report.md` — Full development report:

```markdown
# Development Report

## Problem Statement
[From BRIEF.md]

## Acceptance Criteria
[From spiral/pass-0/acceptance-criteria.md]

## Methodology
[Method, citation, equations, assumptions, valid range]

## Spiral History
### Pass 1
- Methods, key results
### Pass N (final)
- Final convergence assessment

## Verification Summary
[DDV, software, and integration test results]

## Validation Summary
[Sanity checks, limiting cases, reference data, acceptance criteria]

## Convergence Summary
[Table showing key quantities across all passes]

## Assumptions and Limitations
[From methodology/assumptions-register.md]

## References
[All cited papers]

## Traceability
[Chain from acceptance criterion → methodology → code → V&V → final value]
```

The loop will finalize these upon human acceptance.

## Rules

- **Do NOT modify source code, methodology, or tests.** This is an audit phase. The only files you create or modify are: `spiral/pass-N/` reports, `validation/convergence-log.md`, `plots/REVIEW.md`, and optionally `output/` drafts.
- **Do NOT skip any sanity check.** Execute every check in `validation/sanity-checks.md`.
- **If you cannot verify something** (e.g., paper not available, test infrastructure missing), flag it explicitly — do not silently skip it.

## Output

Provide a brief summary of:
- Test results (DDV, software, integration pass rates)
- Validation results (sanity check results)
- Convergence assessment
- Your recommendation (ACCEPT, CONTINUE, or BLOCKED with reasons)
