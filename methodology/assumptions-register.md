# Assumptions Register

<!-- Central registry of all assumptions made across all subsystems.
     Updated by the scope phase (Pass 0) and subsystem refine phases
     whenever cross-cutting assumptions are added or modified. -->

## Format

Each assumption entry follows this format:

```
### A-[NNN]: [Short description]
- **Subsystems:** [Which subsystems rely on this assumption]
- **Statement:** [Precise statement of the assumption]
- **Justification:** [Why this assumption is reasonable]
- **Valid when:** [Conditions under which this holds]
- **Breaks when:** [Conditions under which this fails]
- **Source:** [Paper or reference supporting this]
- **Conflicts:** [Any tension with other assumptions — or "None"]
```

## Assumptions

<!-- Populated during scoping and subsystem refine phases -->
