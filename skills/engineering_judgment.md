# Engineering Judgment

Engineering judgment means verifying results through first-principles reasoning before trusting them. This skill encodes a three-level bounding methodology analogous to the unit → integration → end-to-end testing pyramid in software.

The core discipline: **derive bounds before implementing, verify after implementing, check composition when integrating.**

---

## Level 1 — Phenomenon Bounds

Each individual physical phenomenon gets first-principles bounds derived from dimensional analysis, known coefficient ranges, and scaling laws.

**Catches:** wrong equations, unit errors, wrong coefficient values, misapplied correlations, wrong regime selection.

### When implementing a physical phenomenon

Before writing implementation code:
1. Identify the governing dimensional groups
2. Establish coefficient ranges from known physics
3. Compute an order-of-magnitude expected output using simple arithmetic with known constants
4. Write a bounding test that:
   - Documents the first-principles derivation as a comment
   - Computes the bounds from the stated constants
   - Asserts the implementation output falls within bounds
5. The derivation IS the test documentation

After implementation:
6. Run the bounding test
7. If it fails, your implementation is wrong — not the bound
8. Generate a visual: bar showing computed value within its first-principles bounds

### Example

Frictional resistance on a 200m ship at 15 knots. Re ≈ 1.5×10⁹. At this Re, flat plate Cf ≈ 0.0015. RF = ½ρV²SCf ≈ 365 kN. Bound: [100, 1000] kN. Below 100 kN suggests missing wetted surface or unit error. Above 1000 kN suggests double-counting.

---

## Level 2 — Composition Bounds

When phenomena are combined, their relationships get checked: additive totals must equal the sum of components (conservation), component ratios must match physical expectations, trends must be monotonic where physics demands it.

**Catches:** double-counting, sign errors in coupling, missing interaction terms, violated conservation laws, physically impossible component ratios.

### When composing phenomena

After integrating multiple phenomena into a combined model:
1. Derive composition bounds from phenomenon-level bounds
   - Additive composition: sum the bound ranges
   - Multiplicative: multiply the ranges
   - Check that component ratios match physical expectations
2. Verify conservation laws hold across the composition
3. Write composition-level bounding tests
4. Generate a visual: waterfall/stacked bar showing how components sum to total, with ratio annotations

### Example

Frictional resistance should dominate at Fn < 0.2. Wave resistance fraction should increase with Froude number. Air resistance should be less than 5% of total below 20 knots.

---

## Level 3 — System Bounds

The top-level output gets bounded by an independent back-of-envelope calculation using completely different reasoning from the detailed model.

**Catches:** systematic bias across all components, missing phenomena, wrong problem formulation, errors that look locally reasonable but produce globally wrong answers.

### When producing a system-level answer

Before reporting any result:
1. Derive an independent estimate using completely different reasoning from the detailed model
   - Empirical correlations (e.g., admiralty coefficient)
   - Scaling from similar known cases
   - Back-of-envelope from first principles
2. Compare against the detailed model output
3. If disagreement exceeds a factor of 2, investigate before reporting
4. Document the independent estimate alongside results
5. Generate a visual: detailed model output plotted against independent estimate with bounds

### Example

Admiralty coefficient for this hull type is typically 400-600, giving estimated power of 5-15 MW at 15 knots. If the detailed model produces 2 MW or 50 MW, something is fundamentally wrong.

---

## How bounds compose

Level 2 bounds can be derived from level 1 bounds. If frictional resistance is bounded to [300, 500] kN and wave resistance to [50, 200] kN, then calm water resistance must be in [350, 700] kN. The composition bound comes for free from the phenomenon bounds.

If the composed result falls outside this derived bound, one of the components is wrong. If it falls inside and the components individually satisfy their bounds, you have reasonable confidence in the decomposition.

---

## Bounding Test Structure

Each bounding test follows this pattern:

```
# Level: [phenomenon | composition | system]
# Phenomenon: [what physical quantity is being bounded]
#
# Derivation:
#   [Physical reasoning — dimensional groups, known
#    coefficient ranges, scaling laws used]
#   [Arithmetic — show the computation step by step]
#
# Bound: [lower] to [upper] [units]
# Confidence: [why this range is appropriate]

def test_<phenomenon>_bounds():
    result = compute_<phenomenon>(inputs)
    lower = <derived_lower_bound>
    upper = <derived_upper_bound>
    assert lower <= result <= upper
```

The derivation comment is mandatory. A bounding test without a derivation is not a bounding test — it's an arbitrary assertion.

---

## Test Directory Structure

```
tests/
  bounds/
    phenomenon/   # Level 1: individual phenomena
    composition/  # Level 2: composed phenomena
    system/       # Level 3: system-level cross-checks
```

Place bounding tests in the appropriate subdirectory based on their level.

---

## Visual Evidence

**Level 1 — Phenomenon bounds plot.** For each phenomenon: a horizontal bar showing the computed value positioned within its first-principles bounds. Green if inside bounds, red if outside. The derivation is summarised alongside.

**Level 2 — Composition waterfall.** A waterfall or stacked bar chart showing how individual components sum to the total. Annotated with component ratios. Lines or bands showing the derived composition bounds.

**Level 3 — System cross-check.** The detailed model output plotted against the independent back-of-envelope estimate. A single chart showing agreement/disagreement.
