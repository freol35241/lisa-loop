# Verification Cases

<!-- Hierarchical verification specification. Defined during the methodology phase,
     before any code exists. Each case specifies what to test and what the expected
     result is, with references to the source of expected values. -->

## Level 0 — Individual Functions

<!-- Tests for individual equations/functions.
     Known input → known output, compared against analytical or published values. -->

```
### V0-[NNN]: [Short description]
- **Function:** [What function/equation this tests]
- **Subsystem:** [Which subsystem]
- **Input:** [Specific input values with units]
- **Expected output:** [Expected result with units]
- **Source:** [Where the expected value comes from — paper, analytical derivation]
- **Tolerance:** [Acceptable error and justification]
```

## Level 1 — Subsystem Models

<!-- Tests for complete subsystem models over their valid range. -->

```
### V1-[NNN]: [Short description]
- **Subsystem:** [Which subsystem]
- **Test type:** [Analytical solution / MMS / benchmark / conservation / limiting case / convergence]
- **Description:** [What behavior is being verified]
- **Expected behavior:** [Quantitative or qualitative expected result]
- **Source:** [Reference for expected behavior]
- **Plot:** [What plot to generate for visual verification]
```

## Level 2 — Coupled Subsystem Pairs

<!-- Tests for pairs of coupled subsystems. -->

```
### V2-[NNN]: [Short description]
- **Subsystems:** [Which pair]
- **Test type:** [As above]
- **Description:** [What coupled behavior is being verified]
- **Expected behavior:** [Expected result]
- **Source:** [Reference]
- **Plot:** [What plot to generate]
```

## Level 3 — Full System

<!-- Tests for the complete integrated model. -->

```
### V3-[NNN]: [Short description]
- **Test type:** [Limiting case / conservation / benchmark / symmetry]
- **Description:** [What system behavior is being verified]
- **Expected behavior:** [Expected result]
- **Source:** [Reference]
- **Plot:** [What plot to generate]
```
