# Operational Guide

<!-- This file tells the AI agent how to build, test, and run the project.
     It is read by ALL four phase agents (scope, refine, build, validate).
     Update it as your project evolves. -->

## Language & Runtime

<!-- e.g., Python 3.11+, Rust 1.75+, Julia 1.10+ -->

[Specify language and minimum version]

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

## Run Subsystem-Level Tests

```bash
# Level 0 — Unit tests (individual equations)
# e.g., pytest tests/ -v -m "level0"

# Level 1 — Subsystem tests
# e.g., pytest tests/ -v -m "level1"

# Per-subsystem filtering:
# e.g., pytest tests/ -v -m "[subsystem_name]"
# e.g., pytest tests/ -v -m "level0 and [subsystem_name]"
```

## Run Integration and System Tests

```bash
# Level 2 — Coupled subsystem pair tests
# e.g., pytest tests/ -v -m "level2"

# Level 3 — Full system tests
# e.g., pytest tests/ -v -m "level3"
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
