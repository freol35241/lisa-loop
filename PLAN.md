# Implementation Plan — Resolving Review Items

Organized into 6 sequential phases. Each phase is an atomic commit that compiles and passes tests.

---

## Phase 1: State Machine & Agent Reliability (Critical)

Fixes M1, I1, I6 — the foundation everything else depends on.

### 1a. Add `ScopeComplete` variant to `SpiralState` (M1)

**Files:** `state.rs`, `orchestrator.rs`, `main.rs`

**state.rs:**
- Add variant `ScopeComplete` to `SpiralState` enum (no fields needed)
- Add `Display` impl: `"Scope complete"`
- Add a roundtrip test for the new variant

**orchestrator.rs:357-360:**
- Change `save_state(&lisa_root, &SpiralState::Scoping { attempt: 0 })` → `save_state(&lisa_root, &SpiralState::ScopeComplete)`

**orchestrator.rs:106 (resume match):**
- Change:
  ```rust
  SpiralState::Scoping { .. } | SpiralState::ScopeReview => {
      terminal::log_info("Resuming: scope was incomplete.");
      run_scope(config, project_root)?;
      run(config, project_root, 0, false)
  }
  ```
  to:
  ```rust
  SpiralState::Scoping { .. } | SpiralState::ScopeReview => {
      terminal::log_info("Resuming: scope was incomplete.");
      run_scope(config, project_root)?;
      run(config, project_root, None, false)
  }
  SpiralState::ScopeComplete => {
      terminal::log_info("Scope already complete. Running spiral passes.");
      run(config, project_root, None, false)
  }
  ```

### 1b. Check agent exit status (I1)

**File:** `agent.rs`

- Change `let _ = child.wait();` to:
  ```rust
  let status = child.wait().context("Failed to wait for claude process")?;
  if !status.success() {
      let code = status.code().unwrap_or(-1);
      anyhow::bail!(
          "Agent '{}' exited with code {}. Check the output above for errors.",
          label, code
      );
  }
  ```
- Remove the dead `decode_b64_result` function and its `#[allow(dead_code)]`
- Remove the empty fallback block (lines 180-182) and its misleading comment

### 1c. Change `max_passes` from sentinel to `Option<u32>` (I6)

**File:** `cli.rs`
- Change `max_passes: u32` with `default_value_t = 0` to `max_passes: Option<u32>`

**File:** `orchestrator.rs`
- Change signature: `max_passes: u32` → `max_passes: Option<u32>`
- Change the resolution logic:
  ```rust
  let max = max_passes.unwrap_or(config.limits.max_spiral_passes);
  ```
- Update `resume` to pass `None` instead of `0`

**File:** `main.rs`
- Thread the `Option<u32>` through to `orchestrator::run`

---

## Phase 2: DDV Enforcement Hardening (Critical)

Fixes M2, M3 — the DDV methodology integrity.

### 2a. Make DDV isolation violations a hard error (M2)

**File:** `enforcement.rs`

- Change `verify_ddv_isolation` return type from `()` to `Result<()>`
- When violations are detected, return `Err(anyhow!(...))` instead of just logging
- Include the specific violation paths in the error message
- Add a count: "N source file access violations detected"

**File:** `orchestrator.rs` (run_ddv_red):
- The call is already `enforcement::verify_ddv_isolation(...)`. Change it to:
  ```rust
  enforcement::verify_ddv_isolation(&result.tool_log, config, project_root)?;
  ```
  Since it now returns `Result`, the `?` propagates the error and aborts the phase.

### 2b. Fix DDV test modification detection to include staged changes (M3)

**File:** `git.rs`

- Rename `has_modifications` to `has_any_modifications` and check both staged and unstaged:
  ```rust
  pub fn has_any_modifications(path: &str) -> Result<bool> {
      // Check unstaged
      let unstaged = Command::new("git")
          .args(["diff", "--name-only", path])
          .output()
          .context("Failed to run git diff")?;
      let unstaged_files = String::from_utf8_lossy(&unstaged.stdout);
      if !unstaged_files.trim().is_empty() {
          return Ok(true);
      }
      // Check staged
      let staged = Command::new("git")
          .args(["diff", "--cached", "--name-only", path])
          .output()
          .context("Failed to run git diff --cached")?;
      let staged_files = String::from_utf8_lossy(&staged.stdout);
      Ok(!staged_files.trim().is_empty())
  }
  ```

**File:** `enforcement.rs`
- Update `verify_ddv_tests_unmodified` to call `git::has_any_modifications`
- Also unstage any staged DDV test changes before reverting:
  ```rust
  if git::has_any_modifications(tests_ddv)? {
      terminal::log_warn("Build agent modified DDV tests — reverting!");
      git::reset_path(tests_ddv)?;    // unstage
      git::checkout_path(tests_ddv)?;  // revert working tree
  }
  ```

**File:** `git.rs`
- Add `reset_path` function:
  ```rust
  pub fn reset_path(path: &str) -> Result<()> {
      Command::new("git")
          .args(["reset", "HEAD", "--", path])
          .status()
          .context("Failed to run git reset")?;
      Ok(())
  }
  ```

---

## Phase 3: Error Propagation & Consistency (High)

Fixes I3, I5, I7 — correctness issues that silently swallow errors or use inconsistent logic.

### 3a. Propagate git push failures as errors (I3)

**File:** `git.rs`

- Change `push` to return `Result<()>` with an actual error on failure:
  ```rust
  if status.success() {
      terminal::log_success("Push complete.");
      Ok(())
  } else {
      anyhow::bail!("git push to origin/{} failed. Check remote access and try `lisa resume`.", branch)
  }
  ```
- For `commit_all`, change from `Ok(false)` on failure to `Err(...)`:
  - `git add` failure → `anyhow::bail!("git add failed")`
  - `git commit` failure → `anyhow::bail!("git commit failed")`
  - No changes to commit → keep `Ok(false)` (this is not an error)

### 3b. Share task parsing in block gate (I5)

**File:** `review.rs`

- Import `crate::tasks`
- Replace the block gate's manual counting:
  ```rust
  // Before (lines 262-270):
  let (total, done, blocked) = if plan_path.exists() {
      let content = std::fs::read_to_string(plan_path)?;
      let total = content.matches("### Task").count();
      // ...
  ```
  with:
  ```rust
  let counts = tasks::count_tasks_by_status(plan_path)?;
  let total = counts.total;
  let done = counts.done;
  let blocked = counts.blocked;
  ```

### 3c. Fix `lisa status` to use config lisa_root (I7)

**File:** `main.rs`

- In `cmd_status()`, replace:
  ```rust
  let root = project_root();
  let lisa_root = root.join(".lisa");
  // ...
  let _config = load_config()?;
  ```
  with:
  ```rust
  let root = project_root();
  let config = load_config()?;
  let lisa_root = config.lisa_root(&root);
  ```
- Handle the case where config can't be loaded (no `.lisa/lisa.toml`) by falling back to `.lisa`:
  ```rust
  let lisa_root = match load_config() {
      Ok(config) => config.lisa_root(&root),
      Err(_) => root.join(".lisa"),
  };
  ```

- Apply the same fix to `cmd_doctor()` where `project_root().join(".lisa")` is hardcoded (line 185).

---

## Phase 4: Prompt & Methodology Fixes (High + Medium)

Fixes M4, U5, I2 — prompt conflicts, dead code, template-aware checks.

### 4a. Remove deliverable drafting from validate prompt (M4)

**File:** `prompts/PROMPT_validate.md`

- Remove section "#### Final Output (if convergence achieved)" (lines 262-306)
- Replace with a note:
  ```markdown
  #### Note on Final Output

  If you recommend ACCEPT, do NOT draft deliverables. The finalize phase handles
  deliverable production after human acceptance. Your job is to provide the
  recommendation and evidence — not the deliverables themselves.
  ```

### 4b. Fix redirect file emptiness check (U5)

**File:** `review.rs`

- Replace the current check (lines 235-246):
  ```rust
  if redirect_path.exists()
      && std::fs::metadata(&redirect_path)
          .map(|m| m.len() > 0)
          .unwrap_or(false)
  ```
  with a content-aware check that strips HTML comments:
  ```rust
  if redirect_path.exists() {
      let content = std::fs::read_to_string(&redirect_path).unwrap_or_default();
      let has_real_content = content
          .lines()
          .any(|l| {
              let trimmed = l.trim();
              !trimmed.is_empty()
                  && !trimmed.starts_with('#')
                  && !trimmed.starts_with("<!--")
                  && !trimmed.starts_with("-->")
                  && !trimmed.contains("<!--")
          });
      if has_real_content {
          terminal::log_info(&format!(
              "REDIRECT — guidance saved to {}",
              redirect_path.display()
          ));
      } else {
          terminal::log_warn("Redirect file contains only template comments. Treating as CONTINUE.");
      }
  }
  ```

### 4c. Remove dead code in agent.rs (I2)

**File:** `agent.rs`

- Remove the `decode_b64_result` function (lines 341-348) and its `#[allow(dead_code)]`
- Remove the empty fallback block (lines 179-182) and its comment
- Remove `use base64::Engine;` import if no longer used
- Remove `base64` from `Cargo.toml` dependencies if no longer used anywhere

---

## Phase 5: Structural Improvements (Medium)

Fixes M5, M6, I4 — deduplication, stall detection, parsing robustness.

### 5a. Deduplicate spiral loop logic (M5)

**File:** `orchestrator.rs`

- Extract a shared function:
  ```rust
  fn run_pass_range(
      config: &Config,
      project_root: &Path,
      start_pass: u32,
      max_pass: u32,
  ) -> Result<()>
  ```
  This contains the shared loop body: pass-skipping check, run all 5 phases, commit/push, review gate, finalize-on-accept.

- Rewrite `run()` to call `ensure_scope_complete` then `run_pass_range(config, project_root, 1, max)`.
- Rewrite `run_remaining_passes()` to call `run_pass_range(config, project_root, start_pass, max)`.
- Rewrite `resume_from_phase()` to run the remaining phases of the current pass, then call `run_pass_range` for subsequent passes.

### 5b. Improve build stall detection (M6)

**File:** `orchestrator.rs` (run_build_loop)

- Replace count-based stall detection with content-hash-based detection:
  ```rust
  use std::collections::hash_map::DefaultHasher;
  use std::hash::{Hash, Hasher};

  fn hash_plan_content(plan_path: &Path) -> u64 {
      let content = std::fs::read_to_string(plan_path).unwrap_or_default();
      let mut hasher = DefaultHasher::new();
      content.hash(&mut hasher);
      hasher.finish()
  }
  ```
- Track `prev_plan_hash` instead of `prev_remaining`. If the plan content hash is unchanged after an iteration, increment stall counter. This catches the case where task A is completed but task B is created (hash changes, no stall) while also catching true stalls (no changes at all).

### 5c. Make task parsing more tolerant (I4)

**File:** `tasks.rs`

- Make the heading regex case-insensitive and allow variable `#` depth:
  ```rust
  let task_re = Regex::new(r"(?i)^#{2,4}\s+Task").unwrap();
  ```
- Make the status regex tolerant of extra whitespace:
  ```rust
  let status_re = Regex::new(r"\*\*Status:\*\*\s+(\w+)").unwrap();
  ```
- Change the missing-pass default from 9999 to the `max_pass` parameter (or 0 for "current pass"):
  - Actually, the better fix is: when `current_pass` is missing, default to 1 (the earliest meaningful pass) rather than 9999. This way, passless tasks are included in all pass filters rather than excluded.
- Add a test for heading variations (## Task, #### Task, extra spaces)

---

## Phase 6: UX Polish (Low)

Fixes U1, U2, U6, U7, U8 — user experience improvements. U3 (dry-run), U4 (per-phase replay), and U9 (GUI editors) are deferred as they require more design discussion.

### 6a. Add elapsed time ticker in collapse mode (U1)

**File:** `agent.rs`

- Replace static `▸ label ...` line with a background thread that updates the line every 5 seconds:
  ```rust
  // In collapse mode, spawn a thread that updates the elapsed time
  let label_clone = label.to_string();
  let running = Arc::new(AtomicBool::new(true));
  let running_clone = running.clone();

  let ticker = if collapse_output && std::io::stdout().is_terminal() {
      Some(std::thread::spawn(move || {
          let start = Instant::now();
          while running_clone.load(Ordering::Relaxed) {
              std::thread::sleep(Duration::from_secs(5));
              if !running_clone.load(Ordering::Relaxed) {
                  break;
              }
              let elapsed = start.elapsed().as_secs();
              eprint!("\x1b[1A\x1b[2K");
              eprintln!("  ▸ {} ... ({}s)", label_clone, elapsed);
          }
      }))
  } else {
      None
  };
  // ... at the end, before printing summary:
  running.store(false, Ordering::Relaxed);
  if let Some(handle) = ticker {
      let _ = handle.join();
  }
  ```

**Cargo.toml:** No new dependencies needed — `std::sync::atomic` and `std::thread` are in std.

### 6b. Add `--verbose` flag to override collapse_output (U2)

**File:** `cli.rs`

- Add to `Commands::Run`:
  ```rust
  /// Show full agent output (overrides collapse_output config)
  #[arg(long, short)]
  verbose: bool,
  ```

**File:** `orchestrator.rs`

- Accept `verbose: bool` parameter
- When `verbose` is true, set `config.terminal.collapse_output = false`

### 6c. Add recovery suggestions to error messages (U6)

**Files:** `orchestrator.rs`, `git.rs`

- After "Build aborted at pass N" → append "Run `lisa resume` to retry from the build phase."
- After "git push failed" → append "Check your remote access. Run `lisa resume` to retry."
- After "Reached max spiral passes" → append "Run `lisa run --max-passes N` with a higher limit, or `lisa finalize` to accept current results."
- After agent crash (new I1 error) → append "Run `lisa resume` to retry this phase."

### 6d. Add `--no-pause` startup warning (U7)

**File:** `orchestrator.rs` (top of `run`)

```rust
if no_pause {
    terminal::log_warn(
        "Running with --no-pause: all human review gates will be skipped."
    );
    terminal::log_warn(
        &format!("This will run up to {} spiral passes autonomously.", max)
    );
}
```

### 6e. Flatten `lisa init` subcommand (U8)

**File:** `cli.rs`

- Remove the `InitMode` enum
- Change `Init` to accept the arguments directly with a `mode` field that defaults to `resolve-assignment`:
  ```rust
  /// Initialize a new Lisa Loop project
  Init {
      /// Assignment name (defaults to directory name)
      #[arg(long)]
      name: Option<String>,
      /// Technology preference (e.g., "Python 3.11+ with NumPy/SciPy")
      #[arg(long)]
      tech: Option<String>,
  },
  ```
- This makes `lisa init` work directly (no subcommand needed) while remaining extensible (add `--mode` later if more modes are needed)

**File:** `main.rs`
- Update the match arm for `Commands::Init`

---

## Deferred Items

These require more design discussion and are out of scope for this pass:

| Item | Reason |
|------|--------|
| U3: `--dry-run` mode | Needs design: what exactly to print, how to format prompts without sending them |
| U4: Per-phase replay (`lisa run-phase`) | Needs design: state manipulation semantics, guard rails for skipping phases |
| U9: GUI `$EDITOR` handling | Platform-dependent; `--wait` flag varies by editor. Could add `LISA_EDITOR` override or `editor_wait = true` config, but this is a niche issue |

---

## Execution Order

```
Phase 1 → Phase 2 → Phase 3 → Phase 4 → Phase 5 → Phase 6
```

Each phase is one commit. Dependencies:
- Phase 2 depends on Phase 1 (agent error propagation)
- Phase 3 depends on Phase 1 (Option<u32> max_passes)
- Phases 4, 5, 6 are independent of each other but depend on 1-3

Estimated scope: ~400 lines changed across 10 files.
