# Operational Guide

<!-- This file tells the AI agent how to build, test, and run the project.
     It is read by ALL phase agents (scope, refine, ddv, build, execute, validate).
     Update it as your project evolves. -->

## Technology Preferences (Human-Provided, Optional)

<!-- State your technology preferences here BEFORE running scope.
     If left blank, the scope agent will choose freely based on the problem requirements.

     Examples:
       - "Use Python 3.11+ with NumPy/SciPy"
       - "Must be Rust for performance"
       - "Python with PyO3 for hot loops"
       - "Julia 1.10+ for numerical work"

     You may also specify preferred test frameworks, linters, or build tools. -->

## Resolved Technology Stack (Scope-Agent-Populated)

<!-- This section is populated by the scope agent during Pass 0 after probing
     the local environment. Do not edit manually unless you know what you're doing. -->

### Language & Runtime

[To be resolved by scope agent]

### Key Dependencies

[To be resolved by scope agent]

### Test Framework

[To be resolved by scope agent]

### Stack Justification

[To be resolved by scope agent]

## Setup

```bash
# Install dependencies
```

## Build

```bash
# Build command (if applicable)
```

## Test

```bash
# Run full test suite
```

## Run DDV Tests

```bash
# Run domain verification tests independently
```

## Run Software Tests

```bash
# Run software quality tests independently
```

## Run Integration Tests

```bash
# Run end-to-end integration tests independently
```

## Generate Plots

```bash
# Generate all verification plots
```

## Lint / Format

```bash
# e.g., ruff check . && ruff format --check .
```

## Project-Specific Notes

<!-- Any domain-specific setup, data files needed, special hardware, etc. -->
