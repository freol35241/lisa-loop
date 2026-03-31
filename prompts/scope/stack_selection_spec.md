# Stack Selection Spec — `{{lisa_root}}/STACK.md` + Environment Probing

**This artifact ensures that all subsequent agents use a concrete, verified technology stack rather than making implicit choices.**

## Step 1: Reason About Stack Selection

Before probing the environment, reason about the best technology stack for this project:

- **Computational requirements:** Is the problem compute-bound (favoring a compiled language) or I/O-bound / prototyping-oriented (where a scripting language suffices)?
- **Ecosystem:** Are there domain-specific libraries that favor a particular language?
- **Human preferences:** Read the "Technology Preferences" section of `ASSIGNMENT.md`. If the human stated preferences, respect them. If blank, choose freely.

## Step 2: Probe the Local Environment

The Environment Probe subagent has already checked what runtimes and tools are available.
Synthesize its report: verify it covers all runtimes needed for your chosen stack,
and note any gaps that require human resolution.

## Step 3: Handle Two Categories of Dependencies

**1. Runtimes and toolchains** (language interpreters, compilers, system-level libraries, etc.):

Check if these are present by running version commands. If a required runtime is **not available**:
- Do **NOT** attempt to install it
- Create `{{lisa_root}}/spiral/pass-0/environment-resolution.md` listing what is missing:

```markdown
# Environment Resolution Required

## Missing Runtimes / Toolchains

### [Tool Name]
- **What:** [e.g., Python 3.10+]
- **Why needed:** [e.g., Primary implementation language]
- **Suggested install:** [e.g., `apt install python3` or `pyenv install 3.11`]
- **Alternative:** [Could a different stack choice avoid this? If so, describe.]

## Status
Waiting for human resolution before proceeding.
```

If all required runtimes are present, do **NOT** create this file (or create it empty).

**2. Package-level dependencies** (packages, crates, modules, etc.):

Install these directly using the appropriate package manager. These are routine development dependencies:
- Run the install command using the appropriate package manager
- Verify each install succeeded
- Record installed versions in {{lisa_root}}/STACK.md

## Step 4: Populate {{lisa_root}}/STACK.md

Update the "Resolved Technology Stack" section of `{{lisa_root}}/STACK.md`:

- **Language & Runtime:** Fill with verified language and version (e.g., "Python 3.11.5 (verified present)")
- **Key Dependencies:** List all installed packages with versions
- **Test Framework:** Specify the chosen test framework and version
- **Stack Justification:** Brief reasoning for the technology choices

Fill in all command sections (Setup, Build, Test, Lint, etc.) with **concrete, tested commands** — no more placeholders. If the human pre-filled any command sections before running scope, verify those commands work (run them) rather than overwriting them.

**Backward compatibility:** If {{lisa_root}}/STACK.md already has concrete (non-placeholder) commands filled in by the user, verify they work and keep them. Only populate sections that contain placeholders or template text.
