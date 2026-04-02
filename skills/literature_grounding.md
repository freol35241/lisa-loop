# Literature Grounding

When reference data comparison is needed, this skill provides a methodology for rigorous comparison with published experimental or computational data. This is an optional refinement on top of the engineering judgment bounding hierarchy — not the foundation of verification.

---

## When to Use

Use literature grounding when:
- Published experimental data exists for the specific case being modelled
- The project requires tighter verification than first-principles bounds alone
- Calibrating empirical coefficients against measured data
- Validating against benchmark problems with known solutions

Do NOT use literature grounding as the primary verification method. First-principles bounds (engineering judgment skill) should always be established first.

---

## Methodology

### Step 1: Source Verification

Before using any reference value:
1. **Identify the primary source.** Is this the original paper, or a secondary citation? Trace to the original.
2. **Check the measurement conditions.** Do they match your problem setup? (Reynolds number range, geometry, boundary conditions, fluid properties)
3. **Understand the measurement uncertainty.** Published data has error bars. If the paper doesn't report uncertainty, treat the values with caution.
4. **Check for known corrections.** Some older datasets have known systematic errors or have been superseded.

### Step 2: Unit Verification

1. Verify the units of every reference value
2. Check for implicit unit conventions (e.g., "resistance in pounds" vs "resistance in Newtons")
3. Convert all reference values to your working unit system before comparison
4. Document the conversion explicitly

### Step 3: Comparison Metrics

When comparing model output to reference data:
1. **Absolute error:** `|model - reference|` — meaningful only when the scale is known
2. **Relative error:** `|model - reference| / |reference|` — use for non-zero quantities
3. **Correlation coefficient:** For datasets with multiple points, R² indicates trend agreement
4. **Bias:** Systematic over- or under-prediction across all data points suggests a systematic error

### Step 4: Interpretation

- Agreement within measurement uncertainty → model is consistent with data
- Agreement within first-principles bounds but outside measurement uncertainty → model captures the right physics but may have calibration issues
- Disagreement outside first-principles bounds → fundamental model error (investigate using Level 1 bounds to locate the faulty phenomenon)

---

## Integration with the Bounding Hierarchy

Literature grounding sits on top of the three-level bounding hierarchy:

1. **Level 1 bounds** establish physically plausible ranges
2. **Level 2 bounds** verify composition is correct
3. **Level 3 bounds** provide independent cross-checks
4. **Literature grounding** (this skill) provides tighter comparison where data is available

A result that passes all three bounding levels but disagrees with reference data may indicate:
- The reference data has different conditions than assumed
- A phenomenon is modelled at the right order of magnitude but with insufficient fidelity
- The reference data itself has issues (measurement error, different geometry, etc.)

Never adjust an implementation to match reference data without first understanding why it disagrees. Matching reference data by tuning coefficients without physical justification creates a calibrated-but-wrong model.
