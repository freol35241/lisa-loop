use anyhow::Result;
use crossterm::style::Color;
use std::path::Path;

use crate::agent::{self, AgentResult};
use crate::config::Config;
use crate::enforcement;
use crate::git;
use crate::prompt::{self, Phase};
use crate::review::{self, BlockDecision, ReviewDecision, ScopeDecision};
use crate::state::{self, PassPhase, SpiralState};
use crate::tasks;
use crate::terminal;
use crate::usage;

/// Run the full spiral: scope if needed, then iterate passes
pub fn run(
    config: &Config,
    project_root: &Path,
    max_passes: Option<u32>,
    no_pause: bool,
) -> Result<()> {
    let mut config = config.clone();
    if no_pause {
        config.review.pause = false;
    }

    let max = max_passes.unwrap_or(config.limits.max_spiral_passes);

    if no_pause {
        terminal::log_warn("Running with --no-pause: all human review gates will be skipped.");
        terminal::log_warn(&format!(
            "This will run up to {} spiral passes autonomously.",
            max
        ));
    }

    terminal::log_phase(&format!("LISA LOOP — SPIRAL RUN (max {} passes)", max));

    ensure_scope_complete(&config, project_root)?;

    run_pass_range(&config, project_root, 1, max)
}

/// Run only the scope phase
pub fn run_scope_only(config: &Config, project_root: &Path) -> Result<()> {
    run_scope(config, project_root)
}

/// Resume from saved state
pub fn resume(config: &Config, project_root: &Path) -> Result<()> {
    let lisa_root = config.lisa_root(project_root);
    let state = state::load_state(&lisa_root)?;

    terminal::log_phase("RESUMING FROM SAVED STATE");
    terminal::log_info(&format!("Current state: {}", state));

    // Show error context from previous failure
    let error_path = lisa_root.join("last-error.md");
    if error_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&error_path) {
            println!();
            terminal::print_colored("  Previous failure context:\n", Color::Yellow);
            for line in content.lines().take(15) {
                terminal::println_colored(&format!("  {}", line), Color::Yellow);
            }
            println!();
        }
        let _ = std::fs::remove_file(&error_path);
    }

    match state {
        SpiralState::NotStarted => {
            terminal::log_info("No previous run found. Starting fresh.");
            run(config, project_root, None, false)
        }
        SpiralState::Scoping { .. } | SpiralState::ScopeReview => {
            terminal::log_info("Resuming: scope was incomplete.");
            run_scope(config, project_root)?;
            run(config, project_root, None, false)
        }
        SpiralState::ScopeComplete => {
            terminal::log_info("Scope already complete. Running spiral passes.");
            run(config, project_root, None, false)
        }
        SpiralState::InPass { pass, phase } => {
            resume_from_phase(config, project_root, pass, &phase)
        }
        SpiralState::PassReview { pass } => {
            terminal::log_info(&format!("Resuming: review gate of pass {}.", pass));
            match review::review_gate(config, pass, &lisa_root)? {
                ReviewDecision::Accept => finalize(config, project_root, pass),
                ReviewDecision::Continue | ReviewDecision::Redirect => run_pass_range(
                    config,
                    project_root,
                    pass + 1,
                    config.limits.max_spiral_passes,
                ),
            }
        }
        SpiralState::Complete { final_pass } => {
            terminal::log_success(&format!("Spiral already complete at pass {}.", final_pass));
            Ok(())
        }
    }
}

fn resume_from_phase(
    config: &Config,
    project_root: &Path,
    pass: u32,
    phase: &PassPhase,
) -> Result<()> {
    let lisa_root = config.lisa_root(project_root);

    match phase {
        PassPhase::Refine => {
            terminal::log_info(&format!("Resuming: refine phase at pass {}.", pass));
            run_refine(config, project_root, pass)?;
            run_ddv_red(config, project_root, pass)?;
            if !run_build_loop(config, project_root, pass, 1)? {
                return Ok(());
            }
            run_execute(config, project_root, pass)?;
            run_validate(config, project_root, pass)?;
            git::push(config)?;
        }
        PassPhase::DdvRed => {
            terminal::log_info(&format!("Resuming: DDV Red phase at pass {}.", pass));
            run_ddv_red(config, project_root, pass)?;
            if !run_build_loop(config, project_root, pass, 1)? {
                return Ok(());
            }
            run_execute(config, project_root, pass)?;
            run_validate(config, project_root, pass)?;
            git::push(config)?;
        }
        PassPhase::Build { iteration } => {
            terminal::log_info(&format!(
                "Resuming: build phase at pass {} (iteration {}).",
                pass, iteration
            ));
            if !run_build_loop(config, project_root, pass, *iteration)? {
                return Ok(());
            }
            run_execute(config, project_root, pass)?;
            run_validate(config, project_root, pass)?;
            git::push(config)?;
        }
        PassPhase::Execute => {
            terminal::log_info(&format!("Resuming: execute phase at pass {}.", pass));
            run_execute(config, project_root, pass)?;
            run_validate(config, project_root, pass)?;
            git::push(config)?;
        }
        PassPhase::Validate => {
            terminal::log_info(&format!("Resuming: validate phase at pass {}.", pass));
            run_validate(config, project_root, pass)?;
            git::push(config)?;
        }
    }

    git::create_tag(&format!("lisa/pass-{}", pass))?;
    state::save_state(&lisa_root, &SpiralState::PassReview { pass })?;
    match review::review_gate(config, pass, &lisa_root)? {
        ReviewDecision::Accept => finalize(config, project_root, pass),
        ReviewDecision::Continue | ReviewDecision::Redirect => run_pass_range(
            config,
            project_root,
            pass + 1,
            config.limits.max_spiral_passes,
        ),
    }
}

/// Shared loop body: run passes from start_pass to max_pass
fn run_pass_range(
    config: &Config,
    project_root: &Path,
    start_pass: u32,
    max_pass: u32,
) -> Result<()> {
    let lisa_root = config.lisa_root(project_root);

    for pass in start_pass..=max_pass {
        println!();
        terminal::log_phase(&format!("═══ SPIRAL PASS {} / {} ═══", pass, max_pass));

        if lisa_root
            .join(format!("spiral/pass-{}/PASS_COMPLETE.md", pass))
            .exists()
        {
            terminal::log_info(&format!("Pass {} already complete — skipping.", pass));
            continue;
        }

        run_refine(config, project_root, pass)?;
        run_ddv_red(config, project_root, pass)?;
        if !run_build_loop(config, project_root, pass, 1)? {
            terminal::log_error(&format!(
                "Build aborted at pass {}. Run `lisa resume` to retry from the build phase.",
                pass
            ));
            return Ok(());
        }
        run_execute(config, project_root, pass)?;
        run_validate(config, project_root, pass)?;
        git::push(config)?;
        git::create_tag(&format!("lisa/pass-{}", pass))?;

        state::save_state(&lisa_root, &SpiralState::PassReview { pass })?;
        match review::review_gate(config, pass, &lisa_root)? {
            ReviewDecision::Accept => return finalize(config, project_root, pass),
            ReviewDecision::Continue | ReviewDecision::Redirect => continue,
        }
    }

    terminal::log_warn(&format!(
        "Reached max spiral passes ({}) without acceptance. \
         Run `lisa run --max-passes N` with a higher limit, or `lisa finalize` to accept current results.",
        max_pass
    ));
    Ok(())
}

/// Return the path to the error log file for a given lisa root.
fn error_log(lisa_root: &Path) -> std::path::PathBuf {
    lisa_root.join("last-error.md")
}

/// Wrapper: run agent, record usage, check budget.
fn run_agent_with_tracking(
    config: &Config,
    lisa_root: &Path,
    input: &str,
    model: &str,
    label: &str,
    phase: &str,
    pass: u32,
) -> Result<AgentResult> {
    let err_log = error_log(lisa_root);
    let result = agent::run_agent(
        input,
        model,
        label,
        config.terminal.collapse_output,
        Some(&err_log),
    )?;

    let cumulative = usage::record_invocation(
        lisa_root,
        phase,
        pass,
        model,
        &result.usage,
        result.elapsed_secs,
    )?;

    if result.usage.cost_usd > 0.0 {
        terminal::log_info(&format!(
            "Cost: ${:.4} (cumulative: ${:.4})",
            result.usage.cost_usd, cumulative
        ));
    }

    usage::check_budget(
        cumulative,
        config.limits.budget_usd,
        config.limits.budget_warn_pct,
    )?;

    Ok(result)
}

// --- Individual phase runners ---

fn ensure_scope_complete(config: &Config, project_root: &Path) -> Result<()> {
    let lisa_root = config.lisa_root(project_root);
    if !lisa_root.join("spiral/pass-0/PASS_COMPLETE.md").exists() {
        terminal::log_info("Pass 0 (scoping) not complete. Running scope first.");
        run_scope(config, project_root)?;
    } else {
        terminal::log_info("Pass 0 already complete.");
    }
    Ok(())
}

fn run_scope(config: &Config, project_root: &Path) -> Result<()> {
    let lisa_root = config.lisa_root(project_root);

    terminal::log_phase("PASS 0 — SCOPING");

    if lisa_root.join("spiral/pass-0/PASS_COMPLETE.md").exists() {
        terminal::log_success("Pass 0 already complete.");
        return Ok(());
    }

    state::save_state(&lisa_root, &SpiralState::Scoping { attempt: 1 })?;
    std::fs::create_dir_all(lisa_root.join("spiral/pass-0"))?;

    // Check for existing feedback (resume case)
    let feedback_path = lisa_root.join("spiral/pass-0/scope-feedback.md");
    let extra_context = if feedback_path.exists() {
        let content = std::fs::read_to_string(&feedback_path).unwrap_or_default();
        let non_empty_lines = content
            .lines()
            .filter(|l| !l.starts_with('#') && !l.trim().is_empty() && l.trim() != "-")
            .count();
        if non_empty_lines > 0 {
            terminal::log_info("Detected existing scope feedback — running as refinement.");
            Some(
                "SCOPE REFINEMENT: The human has reviewed your scope artifacts and provided feedback.\n\
                 Read spiral/pass-0/scope-feedback.md carefully and update all affected artifacts.\n\
                 Do not discard previous work — refine it based on the feedback.".to_string(),
            )
        } else {
            None
        }
    } else {
        None
    };

    let input = prompt::build_agent_input(
        Phase::Scope,
        config,
        &lisa_root,
        0,
        extra_context.as_deref(),
    );
    let model = Phase::Scope.model_key(config);

    run_agent_with_tracking(config, &lisa_root, &input, &model, "Scope", "scope", 0)?;
    git::commit_all("scope: pass 0 — scoping complete", config)?;

    // Environment gate
    review::environment_gate(config, &lisa_root)?;

    // Scope review gate
    state::save_state(&lisa_root, &SpiralState::ScopeReview)?;
    loop {
        match review::scope_review_gate(config, &lisa_root)? {
            ScopeDecision::Approve => {
                terminal::log_success("Scope approved. Proceeding to Pass 1.");
                break;
            }
            ScopeDecision::Refine => {
                // Create feedback template if it doesn't exist
                let feedback_path = lisa_root.join("spiral/pass-0/scope-feedback.md");
                if !feedback_path.exists() {
                    std::fs::write(
                        &feedback_path,
                        "# Scope Feedback\n\n## Acceptance Criteria Issues\n-\n\n## Methodology Issues\n-\n\n## Scope Progression Issues\n-\n\n## Validation Issues\n-\n\n## Other\n-\n",
                    )?;
                }

                let editor = std::env::var("EDITOR")
                    .unwrap_or_else(|_| std::env::var("VISUAL").unwrap_or_else(|_| "vi".into()));
                let _ = std::process::Command::new(&editor)
                    .arg(&feedback_path)
                    .status();

                terminal::log_info("Re-running scope agent with feedback...");
                let refine_ctx = "SCOPE REFINEMENT: The human has reviewed your scope artifacts and provided feedback.\n\
                                  Read spiral/pass-0/scope-feedback.md carefully and update all affected artifacts.\n\
                                  Do not discard previous work — refine it based on the feedback.";

                let input = prompt::build_agent_input(
                    Phase::Scope,
                    config,
                    &lisa_root,
                    0,
                    Some(refine_ctx),
                );

                run_agent_with_tracking(
                    config,
                    &lisa_root,
                    &input,
                    &model,
                    "Scope: refinement",
                    "scope",
                    0,
                )?;
                git::commit_all("scope: refined after human feedback", config)?;
                terminal::log_info("Scope refined. Reviewing again...");
            }
            ScopeDecision::Edit => {
                terminal::log_info("Edit scope files directly, then press Enter to approve.");
                print!("  Press Enter when done editing...");
                let _ = std::io::Write::flush(&mut std::io::stdout());
                let mut _buf = String::new();
                let _ = std::io::stdin().read_line(&mut _buf);
                terminal::log_success("Scope approved (manually edited). Proceeding to Pass 1.");
                break;
            }
            ScopeDecision::Quit => {
                terminal::log_warn("Stopping after scope.");
                return Ok(());
            }
        }
    }

    state::save_state(&lisa_root, &SpiralState::ScopeComplete)?;
    git::create_tag("lisa/pass-0")?;
    terminal::log_success("Pass 0 (scoping) complete.");
    Ok(())
}

fn run_refine(config: &Config, project_root: &Path, pass: u32) -> Result<()> {
    let lisa_root = config.lisa_root(project_root);
    terminal::log_phase(&format!("PASS {} — REFINE", pass));
    state::save_state(
        &lisa_root,
        &SpiralState::InPass {
            pass,
            phase: PassPhase::Refine,
        },
    )?;

    std::fs::create_dir_all(lisa_root.join(format!("spiral/pass-{}", pass)))?;

    let prev_pass = pass - 1;
    let mut extra = format!("Current spiral pass: {}\n", pass);
    extra.push_str(&format!(
        "Previous pass results: {}/spiral/pass-{}/\n",
        config.paths.lisa_root, prev_pass
    ));
    let redirect_path = lisa_root.join(format!("spiral/pass-{}/human-redirect.md", prev_pass));
    if redirect_path.exists() {
        extra.push_str(&format!(
            "Human redirect file: {}/spiral/pass-{}/human-redirect.md\n",
            config.paths.lisa_root, prev_pass
        ));
    }

    let input = prompt::build_agent_input(Phase::Refine, config, &lisa_root, pass, Some(&extra));
    let model = Phase::Refine.model_key(config);
    run_agent_with_tracking(
        config,
        &lisa_root,
        &input,
        &model,
        &format!("Refine: pass {}", pass),
        "refine",
        pass,
    )?;
    git::commit_all(&format!("refine: pass {}", pass), config)?;
    Ok(())
}

fn run_ddv_red(config: &Config, project_root: &Path, pass: u32) -> Result<()> {
    let lisa_root = config.lisa_root(project_root);
    terminal::log_phase(&format!(
        "PASS {} — DDV RED (domain verification tests)",
        pass
    ));
    state::save_state(
        &lisa_root,
        &SpiralState::InPass {
            pass,
            phase: PassPhase::DdvRed,
        },
    )?;

    std::fs::create_dir_all(lisa_root.join(format!("spiral/pass-{}", pass)))?;

    let extra = format!("Current spiral pass: {}", pass);
    let input = prompt::build_agent_input(Phase::DdvRed, config, &lisa_root, pass, Some(&extra));
    let model = Phase::DdvRed.model_key(config);
    let result = run_agent_with_tracking(
        config,
        &lisa_root,
        &input,
        &model,
        &format!("DDV Red: pass {}", pass),
        "ddv_red",
        pass,
    )?;

    // Verify DDV isolation
    enforcement::verify_ddv_isolation(&result.tool_log, config, project_root)?;

    git::commit_all(
        &format!("ddv-red: pass {} — domain verification tests written", pass),
        config,
    )?;
    Ok(())
}

fn run_build_loop(
    config: &Config,
    project_root: &Path,
    pass: u32,
    start_iter: u32,
) -> Result<bool> {
    let lisa_root = config.lisa_root(project_root);
    terminal::log_phase(&format!("PASS {} — BUILD (Ralph loop)", pass));

    let plan_path = lisa_root.join("methodology/plan.md");
    let extra = format!("Current spiral pass: {}", pass);

    let mut prev_task_hash = tasks::hash_task_statuses(&plan_path)?;
    let mut stall_count: u32 = 0;

    for iter in start_iter..=config.limits.max_ralph_iterations {
        println!();
        terminal::log_phase(&format!(
            "Build iteration {} / {}",
            iter, config.limits.max_ralph_iterations
        ));

        // Display progress
        let counts = tasks::count_tasks_by_status(&plan_path)?;
        let remaining = counts.total - counts.done - counts.blocked;
        println!(
            "  Progress: {} done / {} remaining / {} blocked (of {} total)",
            counts.done, remaining, counts.blocked, counts.total
        );

        state::save_state(
            &lisa_root,
            &SpiralState::InPass {
                pass,
                phase: PassPhase::Build { iteration: iter },
            },
        )?;

        let input = prompt::build_agent_input(Phase::Build, config, &lisa_root, pass, Some(&extra));
        let model = Phase::Build.model_key(config);
        run_agent_with_tracking(
            config,
            &lisa_root,
            &input,
            &model,
            &format!("Build: iter {}", iter),
            "build",
            pass,
        )?;

        // Verify DDV tests weren't modified
        enforcement::verify_ddv_tests_unmodified(config)?;
        git::commit_all(&format!("build: pass {} iteration {}", pass, iter), config)?;

        // Check completion
        if tasks::all_tasks_done(&plan_path, pass)? {
            if tasks::has_blocked_tasks(&plan_path, pass)? {
                terminal::log_warn("All non-blocked tasks complete. Some tasks are BLOCKED.");
                match review::block_gate(config, pass, &plan_path)? {
                    BlockDecision::Fix => {
                        stall_count = 0;
                        continue;
                    }
                    BlockDecision::Abort => return Ok(false),
                    BlockDecision::Skip => {} // Fall through to break
                }
            }
            terminal::log_success(&format!("All tasks for pass {} complete.", pass));
            break;
        }

        // Dual-signal stall detection
        let cur_task_hash = tasks::hash_task_statuses(&plan_path)?;
        let code_changed = git::source_changed_in_last_commit(&config.paths.source)?;

        let tasks_changed = cur_task_hash != prev_task_hash;
        if tasks_changed || code_changed {
            stall_count = 0;
        } else {
            stall_count += 1;
        }
        prev_task_hash = cur_task_hash;

        let task_signal = if tasks_changed {
            "tasks changed"
        } else {
            "tasks unchanged"
        };
        let code_signal = if code_changed {
            "source files modified"
        } else {
            "source files unchanged"
        };
        println!("  Signals: {}, {}", task_signal, code_signal);

        if stall_count > 0 {
            terminal::log_warn(&format!(
                "No progress detected (stall count: {}/{}).",
                stall_count, config.limits.stall_threshold
            ));
        }

        if stall_count >= config.limits.stall_threshold {
            terminal::log_warn(&format!(
                "Build stalled — no progress for {} consecutive iterations.",
                config.limits.stall_threshold
            ));
            if tasks::has_blocked_tasks(&plan_path, pass)? {
                match review::block_gate(config, pass, &plan_path)? {
                    BlockDecision::Fix => {
                        stall_count = 0;
                        continue;
                    }
                    BlockDecision::Abort => return Ok(false),
                    BlockDecision::Skip => {} // Fall through to break
                }
            } else {
                terminal::log_warn("No blocked tasks found — nothing left to do.");
            }
            break;
        }

        terminal::log_info("Tasks remain — continuing Ralph loop.");
    }

    Ok(true)
}

fn run_execute(config: &Config, project_root: &Path, pass: u32) -> Result<()> {
    let lisa_root = config.lisa_root(project_root);
    terminal::log_phase(&format!("PASS {} — EXECUTE", pass));
    state::save_state(
        &lisa_root,
        &SpiralState::InPass {
            pass,
            phase: PassPhase::Execute,
        },
    )?;

    std::fs::create_dir_all(lisa_root.join(format!("spiral/pass-{}", pass)))?;

    let extra = format!("Current spiral pass: {}", pass);
    let input = prompt::build_agent_input(Phase::Execute, config, &lisa_root, pass, Some(&extra));
    let model = Phase::Execute.model_key(config);
    run_agent_with_tracking(
        config,
        &lisa_root,
        &input,
        &model,
        &format!("Execute: pass {}", pass),
        "execute",
        pass,
    )?;
    git::commit_all(&format!("execute: pass {}", pass), config)?;
    Ok(())
}

fn run_validate(config: &Config, project_root: &Path, pass: u32) -> Result<()> {
    let lisa_root = config.lisa_root(project_root);
    terminal::log_phase(&format!("PASS {} — VALIDATE", pass));
    state::save_state(
        &lisa_root,
        &SpiralState::InPass {
            pass,
            phase: PassPhase::Validate,
        },
    )?;

    std::fs::create_dir_all(lisa_root.join(format!("spiral/pass-{}", pass)))?;

    let extra = format!("Current spiral pass: {}", pass);
    let input = prompt::build_agent_input(Phase::Validate, config, &lisa_root, pass, Some(&extra));
    let model = Phase::Validate.model_key(config);
    run_agent_with_tracking(
        config,
        &lisa_root,
        &input,
        &model,
        &format!("Validate: pass {}", pass),
        "validate",
        pass,
    )?;
    git::commit_all(&format!("validate: pass {}", pass), config)?;
    Ok(())
}

pub fn finalize(config: &Config, project_root: &Path, pass: u32) -> Result<()> {
    let lisa_root = config.lisa_root(project_root);
    terminal::log_phase("FINALIZING — Producing deliverables");

    // Run finalization agent
    let extra = format!(
        "Current spiral pass: {}\n\
         FINALIZATION MODE: The human has ACCEPTED the results.\n\
         Read the review package at {}/spiral/pass-{}/review-package.md for the current answer.\n\
         Read all {}/spiral/pass-*/progress-tracking.md files for the progress history.\n\
         Read {}/methodology/methodology.md for the methodology.\n\
         Produce the deliverables specified in {}/BRIEF.md.",
        pass,
        config.paths.lisa_root,
        pass,
        config.paths.lisa_root,
        config.paths.lisa_root,
        config.paths.lisa_root,
    );

    std::fs::create_dir_all(lisa_root.join("output"))?;

    let input = prompt::build_agent_input(Phase::Finalize, config, &lisa_root, pass, Some(&extra));
    let model = Phase::Finalize.model_key(config);
    run_agent_with_tracking(
        config,
        &lisa_root,
        &input,
        &model,
        "Finalize: output",
        "finalize",
        pass,
    )?;
    git::commit_all("final: generate output deliverables", config)?;

    // Create SPIRAL_COMPLETE.md
    let complete_content = format!(
        "# Spiral Complete\n\n\
         The human has accepted the results.\n\n\
         Completed: {}\n\
         Final pass: {}\n",
        chrono::Local::now().to_rfc3339(),
        pass
    );
    std::fs::write(
        lisa_root.join("spiral/SPIRAL_COMPLETE.md"),
        &complete_content,
    )?;

    state::save_state(&lisa_root, &SpiralState::Complete { final_pass: pass })?;
    git::commit_all(
        &format!("final: spiral complete — answer accepted at pass {}", pass),
        config,
    )?;
    git::push(config)?;

    terminal::log_success("Done. Final deliverables produced.");

    // Show audit summary if it exists
    let audit_path = lisa_root.join("output/audit-summary.md");
    if audit_path.exists() {
        println!();
        terminal::log_info(&format!("Audit summary: {}", audit_path.display()));
    }

    Ok(())
}

/// Roll back to a previous pass boundary.
pub fn rollback(config: &Config, project_root: &Path, target_pass: u32, force: bool) -> Result<()> {
    let lisa_root = config.lisa_root(project_root);
    let tag = format!("lisa/pass-{}", target_pass);

    // Verify tag exists
    let available = git::list_pass_tags();
    if !available.contains(&target_pass) {
        let tag_list = if available.is_empty() {
            "none".to_string()
        } else {
            available
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        };
        anyhow::bail!(
            "Tag '{}' not found. Available rollback points: {}",
            tag,
            tag_list
        );
    }

    // Check for uncommitted changes
    if git::has_uncommitted_changes()? {
        anyhow::bail!("Uncommitted changes detected. Commit or stash them before rolling back.");
    }

    // Confirmation prompt
    if !force {
        terminal::log_warn(&format!(
            "This will reset the repository to the state at pass {}.",
            target_pass
        ));
        terminal::log_warn("A backup branch will be created at current HEAD.");
        print!("  Proceed? [y/N] ");
        let _ = std::io::Write::flush(&mut std::io::stdout());
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            terminal::log_info("Rollback cancelled.");
            return Ok(());
        }
    }

    // Create backup branch
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let backup_branch = format!("lisa/backup/rollback-{}", timestamp);
    git::create_branch(&backup_branch)?;
    terminal::log_info(&format!("Backup branch created: {}", backup_branch));

    // Reset to tag
    git::reset_hard(&tag)?;
    terminal::log_success(&format!("Reset to {}", tag));

    // Restore usage.toml from backup branch (cost history should never be lost)
    let usage_rel = format!("{}/usage.toml", config.paths.lisa_root);
    if let Ok(Some(content)) = git::show_file_from_ref(&backup_branch, &usage_rel) {
        let usage_path = lisa_root.join("usage.toml");
        std::fs::write(&usage_path, &content)?;
        git::commit_all("rollback: restore usage ledger", config)?;
        terminal::log_info("Usage ledger preserved from before rollback.");
    }

    terminal::log_success(&format!(
        "Rolled back to pass {}. Run `lisa resume` to continue.",
        target_pass
    ));
    Ok(())
}
