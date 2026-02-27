# Lisa Loop v3 — Implementation Review

Comprehensive review of methodological coherence, implementation correctness, and UX.

**Reviewed:** All 14 Rust source files, 7 prompt templates, 12 init templates, Cargo.toml, README.md.
**Build status:** Compiles cleanly; 16/16 tests pass.

---

## I. Methodological Coherence

The V-Model + Design Spiral fusion is well-conceived. The five-phase spiral (Refine → DDV Red → Build → Execute → Validate) with staged acceptance and engineering judgment as a named standard is methodologically sound. The following issues affect coherence:

### M1. Scope completion state is semantically broken (Critical)

**Files:** `state.rs:9`, `orchestrator.rs:357-360`

After scope approval, state is saved as `Scoping { attempt: 0 }` with `// 0 = complete`. This is a semantic hack — `attempt: 0` has no "complete" meaning in the type system. The enum lacks a variant representing "scope finished, ready for pass 1."

The `PASS_COMPLETE.md` file existence check (`orchestrator.rs:241,255`) is the *actual* completion gate, making `state.toml` unreliable as the authoritative state machine.

Consequence: `lisa resume` from `Scoping { attempt: 0 }` matches `SpiralState::Scoping { .. }` at `orchestrator.rs:106`, re-entering the scope path. The `PASS_COMPLETE.md` guard inside `run_scope` would catch it, but the user sees the misleading message "Resuming: scope was incomplete."

**Fix:** Add a `ScopeComplete` variant to `SpiralState`, or transition directly to `InPass { pass: 1, phase: Refine }` after scope approval.

### M2. DDV isolation violations are advisory, not enforced (Critical)

**File:** `enforcement.rs:22-27`

When the DDV Red agent reads source files (an isolation violation), the system only logs a warning. It does not abort, roll back, or re-run the phase. The entire methodological value of DDV rests on the independence of the test author from the implementation. A warning that scrolls past — or is hidden in collapse mode — is insufficient.

**Fix:** Make DDV isolation violations a hard error that aborts the phase and requires human intervention.

### M3. DDV test modification detection misses staged changes (Critical)

**Files:** `enforcement.rs:31-38`, `git.rs:89-97`

`verify_ddv_tests_unmodified` uses `git diff --name-only` which only detects *unstaged* changes. The build agent runs with `--dangerously-skip-permissions` and could stage changes via `git add`. Staged modifications to DDV tests would pass undetected.

**Fix:** Check both staged and unstaged diffs: `git diff --name-only` + `git diff --cached --name-only`, or use `git diff HEAD --name-only`.

### M4. Validate prompt creates deliverables that finalize also creates

**Files:** `PROMPT_validate.md:264-305`, `PROMPT_finalize.md`

The validate prompt instructs the agent to draft `output/answer.md` and `output/report.md` if convergence is achieved. The finalize phase's sole job is producing these same deliverables. This creates ambiguity: the validate agent might produce draft deliverables that the finalize agent overwrites, or the validate agent might skip them assuming finalize handles it.

**Fix:** Remove deliverable drafting from the validate prompt. Validate should only *recommend* acceptance; finalize should produce deliverables.

### M5. `run` vs `run_remaining_passes` duplicated control flow

**File:** `orchestrator.rs:38-86` and `orchestrator.rs:197-235`

The main `run` function and `run_remaining_passes` contain nearly identical spiral loop logic (phase execution, pass-skipping, review gates). A bug fix in one must be replicated in the other.

**Fix:** Extract a shared `run_pass_range(config, project_root, start_pass, max_pass)` function.

### M6. Build stall detection has edge cases

**File:** `orchestrator.rs:508-535`

Stall detection compares `cur_remaining == prev_remaining`. If the agent completes task A but creates a new task B (common during reconsiderations), the count stays the same, triggering a false stall. A `Fix` that doesn't actually change the plan will immediately re-stall (2 more iterations) and hit the block gate again — a user-hostile loop.

**Fix:** Track task IDs/names rather than counts, or include a hash of the plan content.

---

## II. Implementation Correctness

### I1. Agent process exit status is silently discarded (Critical)

**File:** `agent.rs:176`

```rust
let _ = child.wait();
```

The exit code of the Claude CLI subprocess is discarded. If `claude` crashes, exits with error, or returns non-zero, the orchestrator continues as if everything worked. This could lead to proceeding with an incomplete scope, missing DDV tests, or broken builds.

**Fix:** Check the exit status and return an error for non-zero exit codes.

### I2. Empty result_text fallback is dead code

**File:** `agent.rs:180-182`

The comment says "Try to extract from the tool log as fallback" but the block is empty. The `decode_b64_result` function (`agent.rs:342`) exists but is `#[allow(dead_code)]`. This is an incomplete feature.

**Fix:** Either implement the fallback or remove the dead code.

### I3. git commit/push failures are swallowed

**File:** `git.rs:7-50`, `git.rs:52-78`

`commit_all` returns `Ok(false)` when git operations fail, and `push` returns `Ok(())` on failure. The orchestrator never knows a commit or push failed. Push failures during finalize are particularly problematic since the user expects their deliverables to be pushed.

**Fix:** At minimum, push failures should propagate as errors. Commit failures should at least be logged more prominently.

### I4. Task parsing assumes exact markdown format

**File:** `tasks.rs:68-111`

The regex-based markdown parser assumes `### Task` headings, `**Status:** XXX`, and `**Pass:** N` format. Agent-produced markdown may vary (extra spaces, different heading levels, bold syntax variations). Missing `**Pass:**` defaults to 9999, meaning such tasks pass the `t.pass <= max_pass` filter for any reasonable `max_pass`.

**Fix:** Make parsing more tolerant, or validate format strictly during the refine phase.

### I5. Block gate uses different parsing logic than tasks module

**File:** `review.rs:262-270`

The block gate counts tasks via `content.matches("### Task").count()` and `content.matches("**Status:** DONE").count()`, while the task module uses a regex parser. These could disagree on edge cases.

**Fix:** Share the `tasks::parse_tasks()` function in the block gate.

### I6. `max_passes=0` is a sentinel value, not "zero passes"

**Files:** `cli.rs:22`, `orchestrator.rs:26-30`

`max_passes` defaults to 0, which is then mapped to the config default (5). `lisa run --max-passes 0` silently runs 5 passes. Zero is a valid count that should mean "zero passes."

**Fix:** Use `Option<u32>` instead of 0-as-sentinel. `None` → config default; `Some(0)` → zero passes.

### I7. `lisa status` ignores configured `lisa_root`

**File:** `main.rs:84`

```rust
let lisa_root = root.join(".lisa");
```

This hardcodes `.lisa` instead of using `config.lisa_root()`. The loaded config (`_config`, line 91) is never used. If a user configures a different `lisa_root`, `lisa status` looks in the wrong place. Compare with `finalize` (`main.rs:56`) which correctly uses `config.lisa_root()`.

**Fix:** Use `config.lisa_root(&root)` consistently.

---

## III. UX Issues

### U1. No progress indication during long agent runs

**File:** `agent.rs:56-59`

In collapse mode (the default), the only output is a static `▸ Scope ...` line. For agents that can run 10+ minutes, there's no spinner, elapsed time ticker, or heartbeat. The user cannot distinguish "working" from "stuck."

**Fix:** Add an elapsed time counter or periodic heartbeat in collapse mode.

### U2. Default `collapse_output = true` hides agent work

**File:** `config.rs:128`

The default hides all tool calls and thinking behind a summary line. For a methodology emphasizing auditability, hiding agent work by default seems contradictory. New users won't know what the agent is doing.

**Fix:** Consider defaulting to `false`, or provide a prominent `--verbose` / `--quiet` flag.

### U3. No `--dry-run` mode

There's no way to preview what `lisa run` will do before committing to expensive Claude CLI calls. A dry-run showing planned phases and the prompts that would be sent (without invoking agents) would improve trust and onboarding.

### U4. No per-phase replay commands

If a user wants to re-run just the DDV Red phase after manually editing verification cases, they must manipulate `state.toml` directly. There's no `lisa run-phase ddv-red --pass 2` command. Only `scope` has its own command.

### U5. Redirect file emptiness check doesn't account for template

**File:** `review.rs:235-246`

After the user edits the redirect file, the check is `m.len() > 0`. But the file was pre-populated with a template (lines 218-224) containing 145+ bytes of comments. It will always be > 0. The redirect is then treated as "guidance saved" even if the user saved without writing anything.

**Fix:** Check if content beyond the template comments was added, or start with an empty file and only create the template as a hint.

### U6. Error messages lack recovery suggestions

Throughout the codebase, messages like "git commit failed" or "Build aborted at pass N" don't tell the user what to do next. Adding "Run `lisa resume` to retry" or "Check `lisa status` for current state" would reduce confusion.

### U7. `--no-pause` is unguarded

**File:** `cli.rs:24-25`

`--no-pause` skips all human review gates including scope review. Combined with the `max_passes=0` sentinel (which means "use default 5"), `lisa run --no-pause` triggers 5 fully autonomous spiral passes with no confirmation. Consider at least a startup warning.

### U8. `lisa init` subcommand nesting adds friction

**File:** `cli.rs:42-52`

Init is modeled as a subcommand enum with only one variant (`ResolveAssignment`). Running `lisa init` without the subcommand produces a clap error, not a helpful message. For a single init mode, this nesting is unnecessary.

### U9. `$EDITOR` assumption may fail for GUI editors

**Files:** `orchestrator.rs:318-322`, `review.rs:226-232`

The code launches `$EDITOR` with `.status()`, which works for terminal editors (vim, nano). GUI editors (VS Code, Sublime) may return immediately before the user finishes editing.

---

## Summary

### Strengths

- **Architecture:** The V-Model + Design Spiral fusion with five-phase spiral and staged acceptance is well-designed
- **DDV concept:** Two-agent separation for independent domain verification is a strong methodological innovation
- **Prompts:** Detailed, prescriptive, and well-structured phase prompts
- **State management:** TOML-based state machine with roundtrip serialization tests
- **Code quality:** Clean Rust, proper error handling with `anyhow`, good module separation
- **Traceability:** Every phase produces explicit artifacts; filesystem is the audit trail
- **Extensibility:** Prompt ejection allows customization without recompilation

### Priorities

| Priority | Count | Items |
|----------|-------|-------|
| Critical | 4 | M1, M2, M3, I1 |
| High | 5 | M4, I3, I5, I6, I7 |
| Medium | 6 | M5, M6, I2, I4, U1, U5 |
| Low | 6 | U2, U3, U4, U6, U7, U8 |
