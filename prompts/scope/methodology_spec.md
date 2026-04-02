# Methodology Document Spec — `{{lisa_root}}/methodology/methodology.md`

**This is the central technical artifact.** It identifies the recommended methods, cites source papers, lists key equations by name/number, and documents assumptions and valid ranges.

**Division of labor:** The methodology created here is an *initial specification*. It identifies the recommended method, cites the source paper, lists key equations by name/number, and documents assumptions and valid ranges. It does NOT contain full equation derivations with every variable defined — that level of detail is the refine phase's job in Pass 1. This intentional fidelity gap is what gives the first refine phase meaningful work: transforming a method recommendation into a complete, implementable specification.

## Template

```markdown
# Methodology

## Phenomenon
[What this project models — from ASSIGNMENT.md]

## Candidate Methods

### [Method 1]
- **Source:** [Citation]
- **Approach:** [Description]
- **Fidelity:** [Low / Medium / High]
- **Pros:** [For our problem]
- **Cons:** [Limitations]

### [Method 2]
...

## Recommended Approach
[Which method and why, considering spiral progression and the human's approach preference from ASSIGNMENT.md]
[If the human asked for simplicity, justify why this method is the simplest that can meet the acceptance criteria]
[If the human asked for state of the art, justify why this is the best available method]
[If there is a mismatch between the requested approach and the acceptance criteria, flag it explicitly]

## Key Equations
[Identify by name and equation number from the source paper — e.g., "Eq. 12 in Faltinsen (1990)" or "ITTC-57 friction line." Do NOT write out the full mathematical expressions here — that is the refine phase's job. If a specific paper is needed but not yet available, flag with [NEEDS_PAPER].]

## Assumptions
[List all assumptions]

## Valid Range
[Parameter ranges where the chosen method applies]
```

If the problem has distinct sub-topics (e.g., frictional resistance, wave resistance, added resistance), organize the methodology into clearly separated **sections** within the single document. Each section follows the same structure above.
