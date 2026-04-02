# Dimensional Analysis

Systematic unit tracking through computation chains. Every physical quantity has dimensions; every equation must balance dimensionally. This skill is applied always, at every level of the engineering judgment hierarchy.

---

## Core Principle

If an equation doesn't balance dimensionally, it is wrong — regardless of how reasonable the numerical output looks. Dimensional analysis catches errors that numerical testing might miss (e.g., a factor that happens to be close to 1 in the test case but is dimensionally incorrect).

---

## When to Apply

1. **Before implementing any equation:** Write down the dimensions of every term. Verify the equation balances.
2. **At every interface:** When one function passes a result to another, verify the units match what the receiver expects.
3. **When combining quantities:** Addition and subtraction require identical dimensions. Multiplication and division combine dimensions algebraically.
4. **When using empirical correlations:** Check that coefficient dimensions make the equation balance. Empirical coefficients often have implicit units.

---

## Methodology

### Step 1: Identify Base Dimensions

Use the standard set: [M] mass, [L] length, [T] time, [Θ] temperature, or domain-appropriate extensions.

Common derived dimensions:
- Force: [M L T⁻²]
- Pressure: [M L⁻¹ T⁻²]
- Energy: [M L² T⁻²]
- Power: [M L² T⁻³]
- Velocity: [L T⁻¹]
- Density: [M L⁻³]

### Step 2: Track Through Computation

For each intermediate result, annotate its dimensions. When you see:
- `a + b` → dimensions of a must equal dimensions of b
- `a * b` → result dimensions = dim(a) × dim(b)
- `f(x)` where f is transcendental (exp, log, sin) → x must be dimensionless

### Step 3: Verify Final Output

The final result must have the expected physical dimensions. If computing force, the result must have dimensions [M L T⁻²].

---

## Common Traps

- **Implicit unit conversions:** Mixing meters and millimeters, or degrees and radians.
- **Empirical formulas with hidden units:** Correlations from textbooks where coefficients absorb unit conversions (e.g., "speed in knots" baked into a coefficient).
- **Dimensionless groups assembled incorrectly:** Reynolds number, Froude number, etc. must be truly dimensionless.
- **Gravitational constant confusion:** g vs gc, weight vs mass.

---

## Integration with Bounding Tests

Every Level 1 bounding test should include a dimensional analysis check as part of its derivation comment. If the derivation shows the bound has dimensions [kN] and the implementation returns a value with dimensions [kN], the dimensional check passes implicitly. If there's any ambiguity about units in the implementation, add an explicit assertion.
