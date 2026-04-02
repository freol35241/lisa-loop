# Numerical Stability

When implementing numerical methods, discretisation errors, convergence behaviour, and floating-point accumulation can produce results that look plausible but are wrong. This skill identifies when to worry and what to check.

---

## When to Apply

Apply this skill when implementing:
- Iterative solvers (Newton-Raphson, fixed-point iteration, optimisation)
- Numerical integration (quadrature, ODE solvers, time-stepping)
- Interpolation or curve fitting
- Matrix operations (solving linear systems, eigenvalue problems)
- Summation of many terms (series, discretised integrals)

---

## Convergence Criteria

### For iterative methods:
1. Define a convergence tolerance **before** implementing the solver
2. Check both absolute and relative convergence: `|x_{n+1} - x_n| < atol` AND `|x_{n+1} - x_n| / |x_n| < rtol`
3. Set a maximum iteration count to prevent infinite loops
4. Log the convergence history (residual vs iteration) — if it oscillates or diverges, the method is unsuitable or the initial guess is bad
5. Verify the converged solution satisfies the original equation (substitution check)

### For discretised problems:
1. Run at multiple resolutions (e.g., N, 2N, 4N grid points)
2. Check that the solution converges as resolution increases
3. Estimate the order of convergence: if doubling resolution reduces error by ~4×, you have second-order convergence
4. The production resolution should be in the converged regime — not just "finer than the coarsest attempt"

---

## Floating-Point Awareness

### Catastrophic cancellation:
When subtracting two nearly equal numbers, relative error explodes. Watch for:
- `a - b` where `a ≈ b` and both are large
- Reformulate: use `(a² - b²) = (a+b)(a-b)` instead of computing `a²` and `b²` separately

### Accumulation errors:
When summing many small terms:
- Sum from smallest to largest (not largest to smallest)
- Consider Kahan summation for critical paths
- For N terms of similar magnitude ε, naive summation error grows as O(√N · ε_machine)

### Condition number:
For linear systems Ax = b:
- Check `cond(A)` — if it's large (>10⁶), the solution is sensitive to input perturbations
- Consider preconditioning or reformulation
- Never invert a matrix when you can solve the system directly

---

## Integration with Bounding Tests

Numerical stability issues manifest as:
- **Level 1 bounds failures** when a single phenomenon's computation is numerically unstable
- **Level 2 bounds failures** when composition amplifies numerical errors (e.g., subtracting two large, nearly-equal components)

When a bounding test fails marginally (result just outside bounds), investigate numerical stability before widening the bounds. The bound derivation is physics-based; if the physics says the answer should be in [100, 1000] and you get 1001, the implementation likely has a numerical issue, not a physics issue.
