# Validation Artifact Specs

This file contains the format specifications for three validation artifacts produced during scoping.

---

## 1. Sanity Checks — `{{lisa_root}}/validation/sanity-checks.md`

```markdown
# Sanity Checks

These are engineering judgment checks to be executed after every spiral pass.
A failure on any check indicates a likely error and should block acceptance.

## Order of Magnitude
- [ ] [Quantity] should be approximately [value] [units] (+/-[order of magnitude])
  - **Reasoning:** [Why this magnitude is expected]

## Expected Trends
- [ ] When [parameter] increases, [quantity] should [increase/decrease/remain constant]
  - **Reasoning:** [Physical justification]

## Physical Bounds
- [ ] [Quantity] must be [positive / in range [a,b] / less than X]
  - **Reasoning:** [Physical constraint]

## Conservation
- [ ] [Conserved quantity] should be preserved to within [tolerance]
  - **Check method:** [How to verify]

## Dimensional Analysis
- [ ] All outputs have correct dimensions/units
  - **Check method:** [How to verify]

## Red Flags
- [ ] [Specific condition that would indicate a clearly wrong answer]
```

---

## 2. Limiting Cases — `{{lisa_root}}/validation/limiting-cases.md`

Extract the limiting cases from your validation research and format them using the `LC-NNN` format (e.g., `LC-001`, `LC-002`). Each entry should include: case description, the condition, expected result, source/reasoning, and a pass/fail status placeholder.

---

## 3. Reference Data — `{{lisa_root}}/validation/reference-data.md`

Extract the reference datasets from your validation research and format them using the `RD-NNN` format (e.g., `RD-001`, `RD-002`). Each entry should include: dataset description, source citation, what it measures, comparison method, and a pass/fail status placeholder.

---

These are the living validation documents that will be checked during every validation phase and refined during methodology refinement phases.
