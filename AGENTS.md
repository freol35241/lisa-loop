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

     You may also specify preferred test frameworks, linters, or build tools.
     The scope agent will respect your preferences when selecting the stack. -->

## Resolved Technology Stack (Scope-Agent-Populated)

<!-- This section is populated by the scope agent during Pass 0 after probing
     the local environment. Do not edit manually unless you know what you're doing.
     If you pre-filled the command sections below before running scope, the scope
     agent will verify they work rather than overwriting them. -->

### Language & Runtime

<!-- Filled by scope agent. Example: Python 3.11.5 (verified), Rust 1.75.0 (verified) -->

[To be resolved by scope agent]

### Key Dependencies

<!-- Filled by scope agent with installed packages and verified versions.
     Example:
       - numpy 1.26.2 (installed via pip)
       - scipy 1.11.4 (installed via pip)
       - pytest 7.4.3 (installed via pip) -->

[To be resolved by scope agent]

### Test Framework

<!-- Filled by scope agent. Example: pytest 7.4.3 with pytest-mark for test levels -->

[To be resolved by scope agent]

### Stack Justification

<!-- Filled by scope agent with reasoning for the technology choices. -->

[To be resolved by scope agent]

## Setup

```bash
# Install dependencies
# e.g., pip install -e ".[dev]" or cargo build
```

## Build

```bash
# Build command (if applicable)
```

## Test

```bash
# Run full test suite
# e.g., pytest tests/ -v
```

## Run DDV Tests

```bash
# Domain-Driven Verification tests (written by DDV Red phase)
# e.g., pytest tests/ddv/ -v -m "ddv"
```

## Run Software Tests

```bash
# Software quality tests (written by build phase)
# e.g., pytest tests/software/ -v -m "software"
```

## Run Integration Tests

```bash
# End-to-end integration tests (written by execute phase)
# e.g., pytest tests/integration/ -v
```

## Generate Plots

```bash
# Generate all verification plots
# e.g., python -m plots.generate_all
```

## Lint / Format

```bash
# e.g., ruff check . && ruff format --check .
```

## Project-Specific Notes

<!-- Any domain-specific setup, data files needed, special hardware, etc. -->
