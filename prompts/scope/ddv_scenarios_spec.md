# DDV Scenario Sketches Spec — `{{lisa_root}}/ddv/scenarios.md`

Write initial DDV scenario sketches directly into `{{lisa_root}}/ddv/scenarios.md`. These are preliminary
verification scenarios that the DDV Agent will later refine and expand with full literature grounding.
Place scenarios after the `## Scenarios` heading (the `## Manifest` section at the top is managed by later phases).

## Format for each scenario

```markdown
## DDV-001: [Short descriptive title]

**Physical behavior:** [What physical/domain behavior this tests]
**Level:** L0 | L1
**Conditions:** [Input parameters with units]
**Expected output:** [Expected result with units and tolerance]
**Source:** [Citation or reasoning for expected value]
**Category:** [unit-function | model-behavior | system-integration | limiting-case | reference-data]
**Visual:** [What plot to generate, or "None" for simple spot-checks]
```

## Level definitions

- **L0** = individual function tests (known input -> known output)
- **L1** = model-level tests (behavior over valid range)

These sketches do not need the full rigor of DDV Agent scenarios — they establish the verification
intent that the DDV Agent will ground in authoritative literature.
