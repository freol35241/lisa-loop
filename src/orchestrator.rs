use anyhow::Result;
use crossterm::style::Color;
use std::path::Path;

use crate::agent::{self, AgentResult};
use crate::config::Config;

use crate::git;
use crate::prompt::{self, Phase};
use crate::review::{self, BlockDecision, RefineDecision, ReviewDecision, ScopeDecision};
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
    ensure_ddv_complete(&config, project_root)?;

    run_pass_range(&config, project_root, 1, max)
}

/// Run only the scope phase
pub fn run_scope_only(config: &Config, project_root: &Path) -> Result<()> {
    run_scope(config, project_root)
}

/// Resume from saved state
pub fn resume(config: &Config, project_root: &Path, no_pause: bool) -> Result<()> {
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
            run(config, project_root, None, no_pause)
        }
        SpiralState::Scoping | SpiralState::ScopeReview => {
            terminal::log_info("Resuming: scope was incomplete.");
            run_scope(config, project_root)?;
            run(config, project_root, None, no_pause)
        }
        SpiralState::ScopeComplete => {
            terminal::log_info("Scope already complete. Running DDV Agent and spiral passes.");
            run(config, project_root, None, no_pause)
        }
        SpiralState::DdvAgent | SpiralState::DdvAgentReview => {
            terminal::log_info("Resuming: DDV Agent was incomplete.");
            run_ddv_agent(config, project_root)?;
            run(config, project_root, None, no_pause)
        }
        SpiralState::DdvAgentComplete => {
            terminal::log_info("DDV scenarios already complete. Running spiral passes.");
            run_pass_range(config, project_root, 1, config.limits.max_spiral_passes)
        }
        SpiralState::InPass { pass, phase } => {
            resume_from_phase(config, project_root, pass, &phase)
        }
        SpiralState::RefineComplete { pass } => {
            terminal::log_info(&format!(
                "Resuming: refine complete for pass {}, proceeding to refine review.",
                pass
            ));
            state::save_state(&lisa_root, &SpiralState::RefineReview { pass })?;
            match refine_gate_loop(config, project_root, pass)? {
                RefineDecision::Approve | RefineDecision::Edit => {}
                RefineDecision::Quit => return Ok(()),
                RefineDecision::Refine => {
                    unreachable!("refine_gate_loop handles Refine internally")
                }
            }
            if !run_build_loop(config, project_root, pass, 1)? {
                return Ok(());
            }
            run_validate(config, project_root, pass)?;
            git::push(config)?;
            git::create_tag(&format!("lisa/pass-{}", pass))?;
            state::save_state(&lisa_root, &SpiralState::PassReview { pass })?;
            match review::review_gate(config, pass, &lisa_root)? {
                ReviewDecision::Finalize => finalize(config, project_root, pass),
                ReviewDecision::Quit => {
                    terminal::log_warn("Stopping after pass review.");
                    Ok(())
                }
                ReviewDecision::Continue | ReviewDecision::Redirect => run_pass_range(
                    config,
                    project_root,
                    pass + 1,
                    config.limits.max_spiral_passes,
                ),
            }
        }
        SpiralState::RefineReview { pass } => {
            terminal::log_info(&format!("Resuming: refine review gate of pass {}.", pass));
            match refine_gate_loop(config, project_root, pass)? {
                RefineDecision::Approve | RefineDecision::Edit => {}
                RefineDecision::Quit => return Ok(()),
                RefineDecision::Refine => {
                    unreachable!("refine_gate_loop handles Refine internally")
                }
            }
            if !run_build_loop(config, project_root, pass, 1)? {
                return Ok(());
            }
            run_validate(config, project_root, pass)?;
            git::push(config)?;
            git::create_tag(&format!("lisa/pass-{}", pass))?;
            state::save_state(&lisa_root, &SpiralState::PassReview { pass })?;
            match review::review_gate(config, pass, &lisa_root)? {
                ReviewDecision::Finalize => finalize(config, project_root, pass),
                ReviewDecision::Quit => {
                    terminal::log_warn("Stopping after pass review.");
                    Ok(())
                }
                ReviewDecision::Continue | ReviewDecision::Redirect => run_pass_range(
                    config,
                    project_root,
                    pass + 1,
                    config.limits.max_spiral_passes,
                ),
            }
        }
        SpiralState::BuildComplete { pass } => {
            terminal::log_info(&format!(
                "Resuming: build complete for pass {}, proceeding to validate.",
                pass
            ));
            resume_from_phase(config, project_root, pass, &PassPhase::Validate)
        }
        SpiralState::ValidateComplete { pass } => {
            terminal::log_info(&format!(
                "Resuming: validate complete for pass {}, proceeding to review.",
                pass
            ));
            git::push(config)?;
            git::create_tag(&format!("lisa/pass-{}", pass))?;
            state::save_state(&lisa_root, &SpiralState::PassReview { pass })?;
            match review::review_gate(config, pass, &lisa_root)? {
                ReviewDecision::Finalize => finalize(config, project_root, pass),
                ReviewDecision::Quit => {
                    terminal::log_warn("Stopping after pass review.");
                    Ok(())
                }
                ReviewDecision::Continue | ReviewDecision::Redirect => run_pass_range(
                    config,
                    project_root,
                    pass + 1,
                    config.limits.max_spiral_passes,
                ),
            }
        }
        SpiralState::PassReview { pass } => {
            terminal::log_info(&format!("Resuming: review gate of pass {}.", pass));
            match review::review_gate(config, pass, &lisa_root)? {
                ReviewDecision::Finalize => finalize(config, project_root, pass),
                ReviewDecision::Quit => {
                    terminal::log_warn("Stopping after pass review.");
                    Ok(())
                }
                ReviewDecision::Continue | ReviewDecision::Redirect => run_pass_range(
                    config,
                    project_root,
                    pass + 1,
                    config.limits.max_spiral_passes,
                ),
            }
        }
        SpiralState::Complete { final_pass } => {
            terminal::log_success(&format!(
                "Spiral already complete at pass {}. Use `lisa continue \"<question>\"` to start a follow-up.",
                final_pass
            ));
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
            match refine_gate_loop(config, project_root, pass)? {
                RefineDecision::Approve | RefineDecision::Edit => {}
                RefineDecision::Quit => return Ok(()),
                RefineDecision::Refine => {
                    unreachable!("refine_gate_loop handles Refine internally")
                }
            }
            if !run_build_loop(config, project_root, pass, 1)? {
                return Ok(());
            }
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
        ReviewDecision::Finalize => finalize(config, project_root, pass),
        ReviewDecision::Quit => {
            terminal::log_warn("Stopping after pass review.");
            Ok(())
        }
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
        match refine_gate_loop(config, project_root, pass)? {
            RefineDecision::Approve | RefineDecision::Edit => {}
            RefineDecision::Quit => return Ok(()),
            RefineDecision::Refine => unreachable!("refine_gate_loop handles Refine internally"),
        }
        if !run_build_loop(config, project_root, pass, 1)? {
            terminal::log_error(&format!(
                "Build aborted at pass {}. Run `lisa resume` to retry from the build phase.",
                pass
            ));
            return Ok(());
        }
        run_validate(config, project_root, pass)?;
        git::push(config)?;
        git::create_tag(&format!("lisa/pass-{}", pass))?;

        state::save_state(&lisa_root, &SpiralState::PassReview { pass })?;
        match review::review_gate(config, pass, &lisa_root)? {
            ReviewDecision::Finalize => return finalize(config, project_root, pass),
            ReviewDecision::Quit => {
                terminal::log_warn("Stopping after pass review.");
                return Ok(());
            }
            ReviewDecision::Continue | ReviewDecision::Redirect => continue,
        }
    }

    terminal::log_warn(&format!(
        "Reached max spiral passes ({}) without finalization. \
         Run `lisa run --max-passes N` with a higher limit, or `lisa finalize` to finalize current results.",
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
        &config.agent.extra_args,
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

    match usage::check_budget(
        cumulative,
        config.limits.budget_usd,
        config.limits.budget_warn_pct,
    ) {
        usage::BudgetStatus::Ok => {}
        usage::BudgetStatus::Warning => {
            terminal::log_warn(&format!(
                "Budget warning: ${:.4} spent of ${:.2} limit ({}% threshold).",
                cumulative, config.limits.budget_usd, config.limits.budget_warn_pct
            ));
        }
        usage::BudgetStatus::Exceeded => {
            match review::budget_gate(config, cumulative, config.limits.budget_usd)? {
                review::BudgetDecision::Continue => {
                    terminal::log_warn("Budget override — continuing despite exceeded budget.");
                }
                review::BudgetDecision::Stop => {
                    anyhow::bail!(
                        "Budget exceeded: ${:.4} spent of ${:.2} limit. Halting.",
                        cumulative,
                        config.limits.budget_usd
                    );
                }
            }
        }
    }

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

fn run_ddv_agent(config: &Config, project_root: &Path) -> Result<()> {
    let lisa_root = config.lisa_root(project_root);

    terminal::log_phase("DDV AGENT — Writing verification scenarios");

    if lisa_root.join("ddv/DDV_COMPLETE.md").exists() {
        terminal::log_success("DDV scenarios already complete.");
        return Ok(());
    }

    state::save_state(&lisa_root, &SpiralState::DdvAgent)?;
    std::fs::create_dir_all(lisa_root.join("ddv"))?;

    let input = prompt::build_agent_input(Phase::DdvAgent, config, &lisa_root, 0, None);
    let model = Phase::DdvAgent.model_key(config);

    run_agent_with_tracking(
        config,
        &lisa_root,
        &input,
        &model,
        "DDV Agent",
        "ddv_agent",
        0,
    )?;
    git::commit_all("ddv-agent: verification scenarios written", config)?;

    // DDV review gate — always shown (even when pause = false)
    state::save_state(&lisa_root, &SpiralState::DdvAgentReview)?;
    loop {
        match review::ddv_review_gate(config, &lisa_root)? {
            review::DdvDecision::Approve => {
                terminal::log_success("DDV scenarios approved. Proceeding to Pass 1.");
                break;
            }
            review::DdvDecision::Refine => {
                // Create feedback template if it doesn't exist
                let feedback_path = lisa_root.join("ddv/ddv-feedback.md");
                if !feedback_path.exists() {
                    std::fs::write(
                        &feedback_path,
                        "# DDV Feedback\n\n## Coverage Gaps\n-\n\n## Scenario Issues\n-\n\n## Missing Sources\n-\n\n## Other\n-\n",
                    )?;
                }

                review::wait_for_edit(
                    "Write your DDV feedback in the file below. Describe coverage gaps, scenario issues, or missing sources.",
                    &feedback_path,
                );

                // Remove completion marker so agent re-runs
                let complete_marker = lisa_root.join("ddv/DDV_COMPLETE.md");
                if complete_marker.exists() {
                    std::fs::remove_file(&complete_marker)?;
                }

                terminal::log_info("Re-running DDV Agent with feedback...");
                state::save_state(&lisa_root, &SpiralState::DdvAgent)?;

                let refine_ctx = "DDV REFINEMENT: The human has reviewed your scenarios and provided feedback.\n\
                                  Read ddv/ddv-feedback.md carefully and update affected scenarios.\n\
                                  Do not discard previous work — refine it based on the feedback.";

                let input = prompt::build_agent_input(
                    Phase::DdvAgent,
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
                    "DDV Agent: refinement",
                    "ddv_agent",
                    0,
                )?;
                git::commit_all("ddv-agent: scenarios refined after human feedback", config)?;
                terminal::log_info("DDV scenarios refined. Reviewing again...");
                state::save_state(&lisa_root, &SpiralState::DdvAgentReview)?;
            }
            review::DdvDecision::Edit => {
                let scenarios_path = lisa_root.join("ddv/scenarios.md");
                let manifest_path = lisa_root.join("ddv/manifest.md");
                terminal::log_info("Edit the DDV scenario files directly with any editor.");
                println!();
                terminal::print_colored("  Scenarios: ", Color::Cyan);
                println!("{}", scenarios_path.display());
                terminal::print_colored("  Manifest:  ", Color::Cyan);
                println!("{}", manifest_path.display());
                println!();
                print!("  Press Enter when you are done editing...");
                let _ = std::io::Write::flush(&mut std::io::stdout());
                let mut _buf = String::new();
                let _ = std::io::stdin().read_line(&mut _buf);

                // Re-display summary after edit
                display_ddv_edit_summary(&lisa_root);

                terminal::log_success(
                    "DDV scenarios approved (manually edited). Proceeding to Pass 1.",
                );
                break;
            }
            review::DdvDecision::Quit => {
                terminal::log_warn("Stopping after DDV Agent.");
                return Ok(());
            }
        }
    }

    state::save_state(&lisa_root, &SpiralState::DdvAgentComplete)?;
    terminal::log_success("DDV Agent complete.");
    Ok(())
}

fn ensure_ddv_complete(config: &Config, project_root: &Path) -> Result<()> {
    let lisa_root = config.lisa_root(project_root);
    if !lisa_root.join("ddv/DDV_COMPLETE.md").exists() {
        terminal::log_info("DDV scenarios not complete. Running DDV Agent first.");
        run_ddv_agent(config, project_root)?;
    } else {
        terminal::log_info("DDV scenarios already complete.");
    }
    Ok(())
}

/// Public entry point for `lisa ddv` command
pub fn run_ddv_agent_only(config: &Config, project_root: &Path) -> Result<()> {
    ensure_scope_complete(config, project_root)?;
    run_ddv_agent(config, project_root)
}

fn run_scope(config: &Config, project_root: &Path) -> Result<()> {
    let lisa_root = config.lisa_root(project_root);

    terminal::log_phase("PASS 0 — SCOPING");

    if lisa_root.join("spiral/pass-0/PASS_COMPLETE.md").exists() {
        terminal::log_success("Pass 0 already complete.");
        return Ok(());
    }

    state::save_state(&lisa_root, &SpiralState::Scoping)?;
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

                review::wait_for_edit(
                    "Write your scope feedback in the file below. Describe issues with acceptance criteria, methodology, scope progression, or validation.",
                    &feedback_path,
                );

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
                terminal::log_info("Edit the scope files directly with any editor.");
                println!();
                terminal::print_colored("  Methodology:  ", Color::Cyan);
                println!("{}", lisa_root.join("methodology/methodology.md").display());
                terminal::print_colored("  Plan:         ", Color::Cyan);
                println!("{}", lisa_root.join("methodology/plan.md").display());
                terminal::print_colored("  Criteria:     ", Color::Cyan);
                println!(
                    "{}",
                    lisa_root
                        .join("spiral/pass-0/acceptance-criteria.md")
                        .display()
                );
                terminal::print_colored("  Spiral plan:  ", Color::Cyan);
                println!(
                    "{}",
                    lisa_root.join("spiral/pass-0/spiral-plan.md").display()
                );
                println!();
                print!("  Press Enter when you are done editing...");
                let _ = std::io::Write::flush(&mut std::io::stdout());
                let mut _buf = String::new();
                let _ = std::io::stdin().read_line(&mut _buf);

                // Re-display summary after edit
                display_scope_edit_summary(&lisa_root);

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

/// Post-refine review gate loop. Returns the terminal decision (Approve, Edit, or Quit).
/// Refine is handled internally by re-running the refine agent with feedback.
fn refine_gate_loop(config: &Config, project_root: &Path, pass: u32) -> Result<RefineDecision> {
    let lisa_root = config.lisa_root(project_root);

    state::save_state(&lisa_root, &SpiralState::RefineReview { pass })?;
    loop {
        match review::refine_review_gate(config, pass, &lisa_root)? {
            RefineDecision::Approve => {
                terminal::log_success("Refine approved. Proceeding to build.");
                return Ok(RefineDecision::Approve);
            }
            RefineDecision::Refine => {
                // Create feedback template
                let feedback_path =
                    lisa_root.join(format!("spiral/pass-{}/refine-feedback.md", pass));
                if !feedback_path.exists() {
                    std::fs::create_dir_all(feedback_path.parent().unwrap())?;
                    std::fs::write(
                        &feedback_path,
                        format!(
                            "# Refine Feedback — Pass {}\n\n\
                             ## Methodology Issues\n-\n\n\
                             ## Task Plan Issues\n-\n\n\
                             ## Scope Issues\n-\n\n\
                             ## Other\n-\n",
                            pass
                        ),
                    )?;
                }

                review::wait_for_edit(
                    "Write your refine feedback in the file below. Describe issues with methodology, task plan, or scope.",
                    &feedback_path,
                );

                terminal::log_info("Re-running refine agent with feedback...");
                let refine_ctx = format!(
                    "REFINE FEEDBACK: The human has reviewed your refine artifacts and provided feedback.\n\
                     Read spiral/pass-{}/refine-feedback.md carefully and update all affected artifacts.\n\
                     Do not discard previous work — refine it based on the feedback.",
                    pass
                );

                let mut extra = format!("Current spiral pass: {}\n", pass);
                let prev_pass = pass - 1;
                extra.push_str(&format!(
                    "Previous pass results: {}/spiral/pass-{}/\n",
                    config.paths.lisa_root, prev_pass
                ));
                extra.push_str(&refine_ctx);

                state::save_state(
                    &lisa_root,
                    &SpiralState::InPass {
                        pass,
                        phase: PassPhase::Refine,
                    },
                )?;

                let input = prompt::build_agent_input(
                    Phase::Refine,
                    config,
                    &lisa_root,
                    pass,
                    Some(&extra),
                );
                let model = Phase::Refine.model_key(config);
                run_agent_with_tracking(
                    config,
                    &lisa_root,
                    &input,
                    &model,
                    &format!("Refine: pass {} (feedback)", pass),
                    "refine",
                    pass,
                )?;
                git::commit_all(
                    &format!("refine: pass {} — refined after human feedback", pass),
                    config,
                )?;
                terminal::log_info("Refine updated. Reviewing again...");
                state::save_state(&lisa_root, &SpiralState::RefineReview { pass })?;
            }
            RefineDecision::Edit => {
                terminal::log_info("Edit the methodology/plan files directly with any editor.");
                println!();
                terminal::print_colored("  Methodology: ", Color::Cyan);
                println!("{}", lisa_root.join("methodology/methodology.md").display());
                terminal::print_colored("  Plan:        ", Color::Cyan);
                println!("{}", lisa_root.join("methodology/plan.md").display());
                println!();
                print!("  Press Enter when you are done editing...");
                let _ = std::io::Write::flush(&mut std::io::stdout());
                let mut _buf = String::new();
                let _ = std::io::stdin().read_line(&mut _buf);

                // Re-display summary after edit
                display_refine_edit_summary(&lisa_root);

                terminal::log_success("Refine approved (manually edited). Proceeding to build.");
                return Ok(RefineDecision::Edit);
            }
            RefineDecision::Quit => {
                terminal::log_warn("Stopping after refine.");
                return Ok(RefineDecision::Quit);
            }
        }
    }
}

/// Display a brief summary of scope artifacts after manual editing.
fn display_scope_edit_summary(lisa_root: &Path) {
    let plan_path = lisa_root.join("methodology/plan.md");
    if let Ok(counts) = crate::tasks::count_tasks_by_status(&plan_path) {
        terminal::print_colored("  Tasks: ", Color::Cyan);
        println!("{} total", counts.total);
    }
    let method_path = lisa_root.join("methodology/methodology.md");
    if method_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&method_path) {
            let section_count = content.lines().filter(|l| l.starts_with("## ")).count();
            terminal::print_colored("  Methodology: ", Color::Cyan);
            println!("{} sections", section_count);
        }
    }
}

/// Display a brief summary of DDV artifacts after manual editing.
fn display_ddv_edit_summary(lisa_root: &Path) {
    let scenarios_path = lisa_root.join("ddv/scenarios.md");
    if scenarios_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&scenarios_path) {
            let count = content.lines().filter(|l| l.starts_with("## DDV-")).count();
            terminal::print_colored("  Scenarios: ", Color::Cyan);
            println!("{}", count);
        }
    }
    let manifest_path = lisa_root.join("ddv/manifest.md");
    if manifest_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&manifest_path) {
            let count = content.lines().filter(|l| l.starts_with("| DDV-")).count();
            terminal::print_colored("  Manifest: ", Color::Cyan);
            println!("{} entries", count);
        }
    }
}

/// Display a brief summary of key artifacts after manual editing.
fn display_refine_edit_summary(lisa_root: &Path) {
    let plan_path = lisa_root.join("methodology/plan.md");
    if let Ok(counts) = crate::tasks::count_tasks_by_status(&plan_path) {
        terminal::print_colored("  Tasks: ", Color::Cyan);
        println!(
            "{} total ({} TODO, {} DONE, {} BLOCKED)",
            counts.total, counts.todo, counts.done, counts.blocked
        );
    }
    let method_path = lisa_root.join("methodology/methodology.md");
    if method_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&method_path) {
            let section_count = content.lines().filter(|l| l.starts_with("## ")).count();
            terminal::print_colored("  Methodology: ", Color::Cyan);
            println!("{} sections", section_count);
        }
    }
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
    state::save_state(&lisa_root, &SpiralState::RefineComplete { pass })?;
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

        git::commit_all(&format!("build: pass {} iteration {}", pass, iter), config)?;

        // Check completion
        if tasks::all_tasks_done(&plan_path, pass)? {
            if tasks::has_blocked_tasks(&plan_path, pass)? {
                terminal::log_warn("All non-blocked tasks complete. Some tasks are BLOCKED.");
                match review::block_gate(config, pass, &plan_path, &lisa_root)? {
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
                match review::block_gate(config, pass, &plan_path, &lisa_root)? {
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

    state::save_state(&lisa_root, &SpiralState::BuildComplete { pass })?;
    Ok(true)
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
    state::save_state(&lisa_root, &SpiralState::ValidateComplete { pass })?;
    Ok(())
}

pub fn finalize(config: &Config, project_root: &Path, pass: u32) -> Result<()> {
    let lisa_root = config.lisa_root(project_root);
    terminal::log_phase("FINALIZING — Producing deliverables");

    // Run finalization agent
    let extra = format!(
        "Current spiral pass: {}\n\
         FINALIZATION MODE: The human has FINALIZED the results.\n\
         Read the review package at {}/spiral/pass-{}/review-package.md for the current answer.\n\
         Read all {}/spiral/pass-*/progress-tracking.md files for the progress history.\n\
         Read {}/methodology/methodology.md for the methodology.\n\
         Produce the deliverables specified in ASSIGNMENT.md.",
        pass, config.paths.lisa_root, pass, config.paths.lisa_root, config.paths.lisa_root,
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

    // Post-finalize confirmation gate
    match review::finalize_gate(config, &lisa_root, pass)? {
        review::FinalizeDecision::Accept => {
            terminal::log_success("Finalization accepted.");
        }
        review::FinalizeDecision::Rollback => {
            terminal::log_warn("Rolling back finalization.");
            if git::has_uncommitted_changes()? {
                terminal::log_error(
                    "Uncommitted changes detected. Commit or stash them before rolling back.",
                );
                anyhow::bail!(
                    "Cannot rollback finalization: uncommitted changes would be lost by git reset."
                );
            }
            state::save_state(&lisa_root, &SpiralState::PassReview { pass })?;
            git::reset_hard("HEAD~1")?;
            terminal::log_info(
                "Finalize commit undone. Run `lisa resume` to return to pass review.",
            );
            return Ok(());
        }
    }

    // Create SPIRAL_COMPLETE.md
    let complete_content = format!(
        "# Spiral Complete\n\n\
         The human has finalized the results.\n\n\
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
        &format!("final: spiral complete — finalized at pass {}", pass),
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

/// Continue with a follow-up question after a completed spiral.
pub fn continue_spiral(
    config: &Config,
    project_root: &Path,
    question: &str,
    max_passes: Option<u32>,
    no_pause: bool,
) -> Result<()> {
    let lisa_root = config.lisa_root(project_root);
    let state = state::load_state(&lisa_root)?;

    let final_pass = match state {
        SpiralState::Complete { final_pass } => final_pass,
        _ => {
            anyhow::bail!(
                "Cannot continue: spiral is not complete (current state: {}). \
                 Use `lisa run` or `lisa resume` instead.",
                state
            );
        }
    };

    // Determine follow-up number
    let assignment_path = project_root.join("ASSIGNMENT.md");
    let assignment_content = std::fs::read_to_string(&assignment_path).unwrap_or_default();
    let follow_up_num = count_follow_ups(&assignment_content) + 1;

    terminal::log_phase(&format!(
        "CONTINUE — Follow-up {} (after pass {})",
        follow_up_num, final_pass
    ));

    // Append follow-up to ASSIGNMENT.md
    let appendix = format!("\n\n## Follow-up {}\n\n{}\n", follow_up_num, question);
    let updated = format!("{}{}", assignment_content, appendix);
    std::fs::write(&assignment_path, &updated)?;
    terminal::log_success(&format!(
        "Appended follow-up {} to ASSIGNMENT.md.",
        follow_up_num
    ));

    // Remove SPIRAL_COMPLETE.md
    let complete_marker = lisa_root.join("spiral/SPIRAL_COMPLETE.md");
    if complete_marker.exists() {
        std::fs::remove_file(&complete_marker)?;
    }

    // Reset state to DdvAgentComplete (scope + DDV scenarios are still valid)
    state::save_state(&lisa_root, &SpiralState::DdvAgentComplete)?;

    git::commit_all(
        &format!(
            "continue: follow-up {} after pass {}",
            follow_up_num, final_pass
        ),
        config,
    )?;

    // Calculate effective max: prior passes + new allowance
    let mut config = config.clone();
    if no_pause {
        config.review.pause = false;
    }
    let effective_max = final_pass + max_passes.unwrap_or(config.limits.max_spiral_passes);

    terminal::log_info(&format!(
        "Starting new passes {}-{} (prior passes 1-{} will be skipped).",
        final_pass + 1,
        effective_max,
        final_pass
    ));

    run(&config, project_root, Some(effective_max), no_pause)
}

/// Count the number of `## Follow-up` sections in ASSIGNMENT.md content.
fn count_follow_ups(content: &str) -> u32 {
    content
        .lines()
        .filter(|line| line.starts_with("## Follow-up "))
        .count() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_follow_ups_none() {
        let content = "# Assignment\n\n## Assignment\nSolve X.\n";
        assert_eq!(count_follow_ups(content), 0);
    }

    #[test]
    fn test_count_follow_ups_multiple() {
        let content = "# Assignment\n\n## Assignment\nSolve X.\n\n\
                        ## Follow-up 1\n\nWhat about Y?\n\n\
                        ## Follow-up 2\n\nAlso Z?\n";
        assert_eq!(count_follow_ups(content), 2);
    }

    #[test]
    fn test_count_follow_ups_ignores_other_headings() {
        let content = "## Assignment\nFoo\n\n## Deliverables\nBar\n\n## Follow-up 1\nBaz\n";
        assert_eq!(count_follow_ups(content), 1);
    }

    #[test]
    fn test_append_follow_up_format() {
        let original = "# Assignment\n\n## Assignment\nSolve X.\n";
        let follow_up_num = 1u32;
        let question = "What about edge case Y?";
        let appendix = format!("\n\n## Follow-up {}\n\n{}\n", follow_up_num, question);
        let result = format!("{}{}", original, appendix);

        assert!(result.starts_with("# Assignment"));
        assert!(result.contains("## Follow-up 1"));
        assert!(result.contains("What about edge case Y?"));
        assert!(result.contains("## Assignment\nSolve X."));
    }
}
