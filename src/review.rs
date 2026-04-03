use anyhow::Result;
use crossterm::style::Color;
use std::io::{self, Write};
use std::path::Path;

use crate::config::Config;
use crate::terminal;

/// Read a line from stdin, returning an error on EOF.
/// Prevents infinite loops when stdin is closed or piped from an empty source.
fn read_stdin_line(buf: &mut String) -> Result<()> {
    let n = io::stdin().read_line(buf)?;
    if n == 0 {
        anyhow::bail!(
            "EOF on stdin — cannot read interactive input (is stdin connected to a terminal?)"
        );
    }
    Ok(())
}

/// Show a file path to the user and wait for them to press Enter after editing.
///
/// This replaces the old $EDITOR spawning pattern — users can edit with whatever
/// tool they prefer (VS Code, vim, nano, a file manager, etc.) and press Enter
/// when done.
pub fn wait_for_edit(label: &str, file_path: &Path) {
    println!();
    terminal::log_info(label);
    println!();
    terminal::print_colored("  File: ", Color::Cyan);
    println!("{}", file_path.display());
    println!();
    print!("  Press Enter when you are done editing...");
    let _ = io::stdout().flush();
    let mut _buf = String::new();
    let _ = read_stdin_line(&mut _buf);
}

#[derive(Debug, PartialEq)]
pub enum ReviewDecision {
    Finalize,
    Continue,
    Redirect,
    Explore,
    Quit,
}

#[derive(Debug, PartialEq)]
pub enum ExploreDecision {
    Merge,
    Discard,
}

#[derive(Debug, PartialEq)]
pub enum ScopeDecision {
    Approve,
    Refine,
    Edit,
    Quit,
}

#[derive(Debug, PartialEq)]
pub enum BlockDecision {
    Fix,
    Skip,
    Abort,
}

#[derive(Debug, PartialEq)]
pub enum RefineDecision {
    Approve,
    Refine,
    Edit,
    Quit,
}

#[derive(Debug, PartialEq)]
pub enum FinalizeDecision {
    Accept,
    Rollback,
}

#[derive(Debug, PartialEq)]
pub enum MethodologyDecision {
    Approve,
    Refine,
    Edit,
    Quit,
}

#[derive(Debug, PartialEq)]
pub enum BudgetDecision {
    Continue,
    Stop,
}

/// Methodology review gate — after Research phase, before Validation Design.
/// Reviews methodology choice, acceptance criteria, and stack selection.
pub fn methodology_review_gate(config: &Config, lisa_root: &Path) -> Result<MethodologyDecision> {
    if !config.review.pause {
        terminal::log_warn("Methodology review skipped (pause = false)");
        return Ok(MethodologyDecision::Approve);
    }

    println!();
    terminal::print_separator();
    terminal::println_bold("  RESEARCH COMPLETE — METHODOLOGY REVIEW");
    terminal::print_separator();
    println!();

    // Approach (from methodology.md)
    let method_path = lisa_root.join("methodology/methodology.md");
    if method_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&method_path) {
            if let Some(approach) = extract_methodology_approach_from(&content) {
                terminal::print_colored("  Approach: ", Color::Cyan);
                println!("{}", approach);
            }
            let sections: Vec<&str> = content
                .lines()
                .filter(|l| l.starts_with("## "))
                .take(8)
                .collect();
            if !sections.is_empty() {
                println!();
                terminal::print_colored("  Methodology sections:\n", Color::Cyan);
                for s in sections {
                    println!("    {}", s);
                }
            }
        }
    }

    // Question (from acceptance-criteria.md)
    let acceptance_path = lisa_root.join("spiral/pass-0/acceptance-criteria.md");
    if acceptance_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&acceptance_path) {
            if let Some(question) = extract_primary_question_from(&content) {
                terminal::print_colored("  Question: ", Color::Cyan);
                println!("{}", question);
            }
            let criteria = extract_acceptance_lines(&content, 5);
            if !criteria.is_empty() {
                println!();
                terminal::print_colored("  Acceptance criteria:\n", Color::Cyan);
                for line in &criteria {
                    println!("    {}", line);
                }
            }
        }
    }

    // Stack (from STACK.md)
    let stack_path = lisa_root.join("STACK.md");
    if stack_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&stack_path) {
            if let Some(stack) = extract_stack_info(&content) {
                terminal::print_colored("  Stack:    ", Color::Cyan);
                println!("{}", stack);
            }
        }
    }

    // File paths
    println!();
    terminal::print_colored("  Files: ", Color::DarkGrey);
    println!("methodology.md, acceptance-criteria.md, STACK.md");

    println!();
    terminal::print_colored("  [A]", Color::Green);
    println!(" APPROVE  — methodology is sound, proceed to validation design and planning");
    terminal::print_colored("  [R]", Color::Yellow);
    println!(" REFINE   — write feedback, re-run the research agent");
    terminal::print_colored("  [E]", Color::Cyan);
    println!(" EDIT     — edit the methodology/criteria files yourself");
    terminal::print_colored("  [Q]", Color::Red);
    println!(" QUIT     — stop the spiral here");
    println!();
    terminal::print_separator();
    println!();

    loop {
        print!("  Your choice [A/R/E/Q]: ");
        io::stdout().flush()?;
        let mut choice = String::new();
        read_stdin_line(&mut choice)?;
        match choice.trim().to_lowercase().as_str() {
            "a" => return Ok(MethodologyDecision::Approve),
            "r" => return Ok(MethodologyDecision::Refine),
            "e" => return Ok(MethodologyDecision::Edit),
            "q" => return Ok(MethodologyDecision::Quit),
            _ => println!("  Invalid choice. Enter A, R, E, or Q."),
        }
    }
}

/// Scope review gate — after Pass 0
pub fn scope_review_gate(config: &Config, lisa_root: &Path) -> Result<ScopeDecision> {
    if !config.review.pause {
        terminal::log_warn("Scope review skipped (pause = false)");
        return Ok(ScopeDecision::Approve);
    }

    println!();
    terminal::print_separator();
    terminal::println_bold("  PASS 0 (SCOPING) COMPLETE — REVIEW REQUIRED");
    terminal::print_separator();
    println!();

    // Question (from acceptance-criteria.md)
    let acceptance_path = lisa_root.join("spiral/pass-0/acceptance-criteria.md");
    if acceptance_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&acceptance_path) {
            if let Some(question) = extract_primary_question_from(&content) {
                terminal::print_colored("  Question: ", Color::Cyan);
                println!("{}", question);
            }
        }
    }

    // Approach (from methodology.md)
    let method_path = lisa_root.join("methodology/methodology.md");
    if method_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&method_path) {
            if let Some(approach) = extract_methodology_approach_from(&content) {
                terminal::print_colored("  Approach: ", Color::Cyan);
                println!("{}", approach);
            }
        }
    }

    // Approach philosophy (from spiral-plan.md)
    let spiral_plan_path = lisa_root.join("spiral/pass-0/spiral-plan.md");
    if spiral_plan_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&spiral_plan_path) {
            if let Some(philosophy) = extract_section_first_line(&content, "## Approach Philosophy")
            {
                terminal::print_colored("  Ambition: ", Color::Cyan);
                println!("{}", philosophy);
            }
        }
    }

    // Stack (from STACK.md)
    let agents_path = lisa_root.join("STACK.md");
    if agents_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&agents_path) {
            if let Some(stack) = extract_stack_info(&content) {
                terminal::print_colored("  Stack:    ", Color::Cyan);
                println!("{}", stack);
            }
        }
    }

    // Tasks (from plan.md)
    let plan_path = lisa_root.join("methodology/plan.md");
    if plan_path.exists() {
        if let Ok(counts) = crate::tasks::count_tasks_by_status(&plan_path) {
            if counts.total > 0 {
                terminal::print_colored("  Tasks:    ", Color::Cyan);
                println!(
                    "{} total ({} TODO, {} BLOCKED)",
                    counts.total, counts.todo, counts.blocked
                );
            }
        }
    }

    // Acceptance criteria lines
    if acceptance_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&acceptance_path) {
            let criteria = extract_acceptance_lines(&content, 5);
            if !criteria.is_empty() {
                println!();
                terminal::print_colored("  Acceptance criteria:\n", Color::Cyan);
                for line in &criteria {
                    println!("    {}", line);
                }
            }
        }
    }

    // Spiral plan summary
    let spiral_plan = lisa_root.join("spiral/pass-0/spiral-plan.md");
    if spiral_plan.exists() {
        if let Ok(content) = std::fs::read_to_string(&spiral_plan) {
            let pass_lines: Vec<&str> = content
                .lines()
                .filter(|l| {
                    (l.starts_with("| ") && l.contains("Pass"))
                        || (l.starts_with("| ")
                            && l.chars().nth(2).is_some_and(|c| c.is_ascii_digit()))
                })
                .take(5)
                .collect();
            if !pass_lines.is_empty() {
                println!();
                terminal::print_colored("  Scope progression:\n", Color::Cyan);
                for line in pass_lines {
                    println!("    {}", line);
                }
            }
        }
    }

    // Methodology sections
    if method_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&method_path) {
            let sections: Vec<&str> = content
                .lines()
                .filter(|l| l.starts_with("## ") && !l.contains("Phenomenon"))
                .take(8)
                .collect();
            if !sections.is_empty() {
                println!();
                terminal::print_colored("  Methodology sections:\n", Color::Cyan);
                for s in sections {
                    println!("    {}", s);
                }
            }
        }
    }

    // File paths (compact, at bottom)
    println!();
    terminal::print_colored("  Files: ", Color::DarkGrey);
    println!("methodology.md, plan.md, acceptance-criteria.md, spiral-plan.md");

    println!();
    terminal::print_colored("  [A]", Color::Green);
    println!(" APPROVE  — accept scope artifacts and proceed to Pass 1");
    terminal::print_colored("  [R]", Color::Yellow);
    println!(" REFINE   — write feedback to a file, then the scope agent re-runs");
    terminal::print_colored("  [E]", Color::Cyan);
    println!(" EDIT     — edit the scope files yourself with any editor, then approve");
    terminal::print_colored("  [Q]", Color::Red);
    println!(" QUIT     — stop the spiral here (resume later with `lisa resume`)");
    println!();
    terminal::print_separator();
    println!();

    loop {
        print!("  Your choice [A/R/E/Q]: ");
        io::stdout().flush()?;
        let mut choice = String::new();
        read_stdin_line(&mut choice)?;
        match choice.trim().to_lowercase().as_str() {
            "a" => return Ok(ScopeDecision::Approve),
            "r" => return Ok(ScopeDecision::Refine),
            "e" => return Ok(ScopeDecision::Edit),
            "q" => return Ok(ScopeDecision::Quit),
            _ => println!("  Invalid choice. Enter A, R, E, or Q."),
        }
    }
}

/// Refine review gate — after each pass's refine phase
pub fn refine_review_gate(config: &Config, pass: u32, lisa_root: &Path) -> Result<RefineDecision> {
    if !config.review.pause {
        terminal::log_warn("Refine review skipped (pause = false)");
        return Ok(RefineDecision::Approve);
    }

    println!();
    terminal::print_separator();
    terminal::println_bold(&format!(
        "  PASS {} REFINE COMPLETE — REVIEW REQUIRED",
        pass
    ));
    terminal::print_separator();
    println!();

    // Show refine summary path
    let summary_path = lisa_root.join(format!("spiral/pass-{}/refine-summary.md", pass));
    if summary_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&summary_path) {
            let line_count = content.lines().filter(|l| !l.trim().is_empty()).count();
            terminal::print_colored("  Refine summary: ", Color::Cyan);
            println!("{} lines", line_count);
        }
    }

    // Methodology change count
    let method_path = lisa_root.join("methodology/methodology.md");
    if method_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&method_path) {
            let section_count = content.lines().filter(|l| l.starts_with("## ")).count();
            terminal::print_colored("  Methodology:    ", Color::Cyan);
            println!("{} sections", section_count);
        }
    }

    // Plan task delta
    let plan_path = lisa_root.join("methodology/plan.md");
    if plan_path.exists() {
        if let Ok(counts) = crate::tasks::count_tasks_by_status(&plan_path) {
            if counts.total > 0 {
                terminal::print_colored("  Tasks:          ", Color::Cyan);
                println!(
                    "{} total ({} TODO, {} DONE, {} BLOCKED)",
                    counts.total, counts.todo, counts.done, counts.blocked
                );
            }
        }
    }

    // Reconsideration resolutions
    let recon_dir = lisa_root.join(format!("spiral/pass-{}/reconsiderations", pass));
    if recon_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&recon_dir) {
            let files: Vec<String> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_file())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect();
            if !files.is_empty() {
                terminal::print_colored("  Reconsiderations: ", Color::Cyan);
                println!("{} resolved", files.len());
            }
        }
    }

    println!();
    terminal::print_colored("  [A]", Color::Green);
    println!(" APPROVE  — accept refine artifacts and proceed to build phase");
    terminal::print_colored("  [R]", Color::Yellow);
    println!(" REFINE   — write feedback to a file, then the refine agent re-runs");
    terminal::print_colored("  [E]", Color::Cyan);
    println!(" EDIT     — edit the methodology/plan files yourself, then approve");
    terminal::print_colored("  [Q]", Color::Red);
    println!(" QUIT     — stop the spiral here (resume later with `lisa resume`)");
    println!();
    terminal::print_separator();
    println!();

    loop {
        print!("  Your choice [A/R/E/Q]: ");
        io::stdout().flush()?;
        let mut choice = String::new();
        read_stdin_line(&mut choice)?;
        match choice.trim().to_lowercase().as_str() {
            "a" => return Ok(RefineDecision::Approve),
            "r" => return Ok(RefineDecision::Refine),
            "e" => return Ok(RefineDecision::Edit),
            "q" => return Ok(RefineDecision::Quit),
            _ => println!("  Invalid choice. Enter A, R, E, or Q."),
        }
    }
}

/// Pass review gate — after each pass's audit phase
pub fn review_gate(config: &Config, pass: u32, lisa_root: &Path) -> Result<ReviewDecision> {
    if !config.review.pause {
        terminal::log_warn("Review gate skipped (pause = false) — defaulting to CONTINUE");
        return Ok(ReviewDecision::Continue);
    }

    println!();
    terminal::print_separator();
    terminal::println_bold(&format!(
        "  SPIRAL PASS {} COMPLETE — REVIEW REQUIRED",
        pass
    ));
    terminal::print_separator();
    println!();

    // Parse and display review package
    let review_path = lisa_root.join(format!("spiral/pass-{}/review-package.md", pass));
    if review_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&review_path) {
            display_review_summary(&content, pass);
        }
    } else {
        terminal::print_colored(
            &format!("  Review package not found at {}\n", review_path.display()),
            Color::Yellow,
        );
    }

    println!();
    terminal::print_colored("  Files:\n", Color::Cyan);
    println!(
        "    Review:     {}/spiral/pass-{}/review-package.md",
        lisa_root.display(),
        pass
    );
    println!(
        "    Execution:  {}/spiral/pass-{}/execution-report.md",
        lisa_root.display(),
        pass
    );
    println!(
        "    Plots:      {}/spiral/pass-{}/plots/REVIEW.md",
        lisa_root.display(),
        pass
    );
    println!();

    terminal::print_colored("  [F]", Color::Green);
    println!(" FINALIZE — results are satisfactory, produce the final report");
    terminal::print_colored("  [C]", Color::Yellow);
    println!(" CONTINUE — run another spiral pass to improve results");
    terminal::print_colored("  [R]", Color::Cyan);
    println!(" REDIRECT — write guidance to a file to steer the next pass");
    terminal::print_colored("  [E]", Color::Magenta);
    println!(" EXPLORE  — create a side-branch to investigate an alternative approach");
    terminal::print_colored("  [Q]", Color::Red);
    println!(" QUIT     — stop the spiral here (resume later with `lisa resume`)");
    println!();
    terminal::print_separator();
    println!();

    loop {
        print!("  Your choice [F/C/R/E/Q]: ");
        io::stdout().flush()?;
        let mut choice = String::new();
        read_stdin_line(&mut choice)?;
        match choice.trim().to_uppercase().as_str() {
            "F" => {
                terminal::log_success("FINALIZED — producing final output.");
                return Ok(ReviewDecision::Finalize);
            }
            "C" => {
                terminal::log_info("CONTINUE — proceeding to next pass.");
                return Ok(ReviewDecision::Continue);
            }
            "Q" => {
                terminal::log_warn("QUIT — stopping after this pass.");
                return Ok(ReviewDecision::Quit);
            }
            "R" => {
                // Create redirect file and open editor
                let redirect_path =
                    lisa_root.join(format!("spiral/pass-{}/human-redirect.md", pass));
                std::fs::create_dir_all(redirect_path.parent().unwrap())?;

                let template = format!(
                    "# Human Redirect — Pass {}\n\n\
                     <!-- Write your guidance for the next pass below. Save and close when done. -->\n\
                     <!-- Delete this comment block. -->\n\n",
                    pass
                );
                std::fs::write(&redirect_path, &template)?;

                wait_for_edit(
                    "Write your guidance for the next spiral pass in the file below.",
                    &redirect_path,
                );

                if redirect_path.exists() {
                    let content = std::fs::read_to_string(&redirect_path).unwrap_or_default();
                    let has_real_content = content.lines().any(|l| {
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
                        return Ok(ReviewDecision::Redirect);
                    } else {
                        terminal::log_warn(
                            "Redirect file contains only template comments. Treating as CONTINUE.",
                        );
                    }
                } else {
                    terminal::log_warn("Redirect file is empty. Treating as CONTINUE.");
                }
                return Ok(ReviewDecision::Continue);
            }
            "E" => {
                terminal::log_info("EXPLORE — creating a side-branch for investigation.");
                return Ok(ReviewDecision::Explore);
            }
            _ => println!("  Please enter F, C, R, E, or Q."),
        }
    }
}

/// Gate shown after an exploration completes. User decides to merge findings or discard.
pub fn explore_review_gate(
    pass: u32,
    explore_id: u32,
    lisa_root: &Path,
) -> Result<ExploreDecision> {
    let findings_path = lisa_root.join(format!(
        "spiral/pass-{}/explore-{}/findings.md",
        pass, explore_id
    ));

    println!();
    terminal::print_separator();
    terminal::println_bold("  EXPLORATION REVIEW");
    println!();

    if findings_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&findings_path) {
            let lines: Vec<&str> = content.lines().take(15).collect();
            for line in &lines {
                println!("    {}", line);
            }
            if content.lines().count() > 15 {
                println!("    ...");
            }
        }
    } else {
        println!("  No findings file produced.");
    }

    println!();
    println!("  Findings: {}", findings_path.display());
    println!();

    terminal::print_colored("  [M]", Color::Green);
    println!(" MERGE   — merge exploration findings back into the main branch");
    terminal::print_colored("  [D]", Color::Red);
    println!(" DISCARD — discard the exploration branch");
    println!();

    loop {
        print!("  Your choice [M/D]: ");
        io::stdout().flush()?;
        let mut choice = String::new();
        read_stdin_line(&mut choice)?;
        match choice.trim().to_uppercase().as_str() {
            "M" => {
                terminal::log_success("MERGE — folding exploration into main branch.");
                return Ok(ExploreDecision::Merge);
            }
            "D" => {
                terminal::log_warn("DISCARD — abandoning exploration branch.");
                return Ok(ExploreDecision::Discard);
            }
            _ => println!("  Please enter M or D."),
        }
    }
}

/// Block gate — when build loop stalls or all remaining tasks are blocked
pub fn block_gate(
    config: &Config,
    pass: u32,
    plan_path: &Path,
    lisa_root: &Path,
) -> Result<BlockDecision> {
    if !config.review.pause {
        terminal::log_warn("Block gate skipped (pause = false) — defaulting to ABORT");
        return Ok(BlockDecision::Abort);
    }

    // Gather counts
    let counts = crate::tasks::count_tasks_by_status(plan_path)?;
    let total = counts.total;
    let done = counts.done;
    let blocked = counts.blocked;

    println!();
    terminal::print_separator();
    terminal::print_colored("  ", Color::Red);
    terminal::println_bold("BUILD BLOCKED");
    terminal::print_separator();
    println!();
    terminal::print_colored("  Completed: ", Color::White);
    terminal::print_colored(&format!("{}", done), Color::Green);
    println!(" / {} tasks", total);
    terminal::print_colored("  Blocked:   ", Color::White);
    terminal::print_colored(&format!("{}", blocked), Color::Red);
    println!(" tasks");
    println!();

    // Show blocked task names
    if plan_path.exists() {
        if let Ok(content) = std::fs::read_to_string(plan_path) {
            println!("  Blocked tasks:");
            let mut current_task = String::new();
            let mut is_blocked = false;

            for line in content.lines() {
                if line.starts_with("### Task") {
                    if is_blocked {
                        println!("    • {}", current_task);
                    }
                    current_task = line.trim_start_matches("### ").to_string();
                    is_blocked = false;
                }
                if line.contains("**Status:** BLOCKED") {
                    is_blocked = true;
                }
            }
            if is_blocked {
                println!("    • {}", current_task);
            }
            println!();
        }
    }

    // Show reconsideration files if any exist
    let recon_dir = lisa_root.join(format!("spiral/pass-{}/reconsiderations", pass));
    if recon_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&recon_dir) {
            let files: Vec<String> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_file())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect();
            if !files.is_empty() {
                terminal::print_colored("  Reconsiderations:\n", Color::Cyan);
                for f in &files {
                    println!("    {}", f);
                }
                println!();
            }
        }
    }

    terminal::print_colored("  [F]", Color::Green);
    println!(" FIX   — edit the plan to resolve blocked tasks, then resume build");
    terminal::print_colored("  [S]", Color::Yellow);
    println!(" SKIP  — skip blocked tasks and continue to the next phase");
    terminal::print_colored("  [X]", Color::Red);
    println!(" ABORT — stop this spiral pass (resume later with `lisa resume`)");
    println!();
    terminal::print_separator();
    println!();

    loop {
        print!("  Your choice [F/S/X]: ");
        io::stdout().flush()?;
        let mut choice = String::new();
        read_stdin_line(&mut choice)?;
        match choice.trim().to_uppercase().as_str() {
            "F" => {
                wait_for_edit(
                    "FIX — Edit the plan to resolve blocked tasks, then press Enter.",
                    plan_path,
                );

                // Re-read and display updated task counts
                if let Ok(updated) = crate::tasks::count_tasks_by_status(plan_path) {
                    println!();
                    terminal::print_colored("  Updated: ", Color::Green);
                    println!(
                        "{} done / {} todo / {} blocked (of {} total)",
                        updated.done, updated.todo, updated.blocked, updated.total
                    );
                }

                terminal::log_info("Resuming build with updated plan.");
                return Ok(BlockDecision::Fix);
            }
            "S" => {
                terminal::log_info("SKIP — continuing to next phase.");
                return Ok(BlockDecision::Skip);
            }
            "X" => {
                terminal::log_error("ABORT — stopping spiral pass.");
                return Ok(BlockDecision::Abort);
            }
            _ => println!("  Please enter F, S, or X."),
        }
    }
}

/// Post-finalize confirmation gate
pub fn finalize_gate(config: &Config, lisa_root: &Path, pass: u32) -> Result<FinalizeDecision> {
    if !config.review.pause {
        terminal::log_warn("Finalize gate skipped (pause = false) — auto-accepting");
        return Ok(FinalizeDecision::Accept);
    }

    println!();
    terminal::print_separator();
    terminal::println_bold("  FINALIZATION COMPLETE — CONFIRM RESULTS");
    terminal::print_separator();
    println!();

    // Show output files
    let output_dir = lisa_root.join("output");
    if output_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&output_dir) {
            let files: Vec<String> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_file())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect();
            if !files.is_empty() {
                terminal::print_colored("  Output files:\n", Color::Cyan);
                for f in &files {
                    println!("    {}", f);
                }
                println!();
            }
        }
    }

    // Show audit summary preview
    let audit_path = lisa_root.join("output/audit-summary.md");
    if audit_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&audit_path) {
            terminal::print_colored("  Audit summary (first 10 lines):\n", Color::Cyan);
            for line in content.lines().take(10) {
                println!("    {}", line);
            }
            println!();
        }
    }

    terminal::print_colored("  [A]", Color::Green);
    println!(" ACCEPT   — confirm results and complete the spiral");
    terminal::print_colored("  [R]", Color::Red);
    println!(
        " ROLLBACK — undo the finalize and return to pass {} review",
        pass
    );
    println!();
    terminal::print_separator();
    println!();

    loop {
        print!("  Choice: ");
        io::stdout().flush()?;
        let mut choice = String::new();
        read_stdin_line(&mut choice)?;
        match choice.trim().to_lowercase().as_str() {
            "a" => return Ok(FinalizeDecision::Accept),
            "r" => return Ok(FinalizeDecision::Rollback),
            _ => println!("  Invalid choice. Enter A or R."),
        }
    }
}

/// Budget exhaustion gate
pub fn budget_gate(config: &Config, cumulative: f64, budget: f64) -> Result<BudgetDecision> {
    if !config.review.pause {
        terminal::log_warn("Budget gate skipped (pause = false) — defaulting to STOP");
        return Ok(BudgetDecision::Stop);
    }

    println!();
    terminal::print_separator();
    terminal::print_colored("  ", Color::Red);
    terminal::println_bold("BUDGET EXCEEDED");
    terminal::print_separator();
    println!();
    terminal::print_colored("  Spent:  ", Color::White);
    terminal::println_colored(&format!("${:.4}", cumulative), Color::Red);
    terminal::print_colored("  Budget: ", Color::White);
    println!("${:.2}", budget);
    println!();

    terminal::print_colored("  [C]", Color::Yellow);
    println!(" CONTINUE — override the budget limit and keep going");
    terminal::print_colored("  [S]", Color::Red);
    println!(" STOP     — halt the spiral (resume later with `lisa resume`)");
    println!();
    terminal::print_separator();
    println!();

    loop {
        print!("  Choice: ");
        io::stdout().flush()?;
        let mut choice = String::new();
        read_stdin_line(&mut choice)?;
        match choice.trim().to_lowercase().as_str() {
            "c" => return Ok(BudgetDecision::Continue),
            "s" => return Ok(BudgetDecision::Stop),
            _ => println!("  Invalid choice. Enter C or S."),
        }
    }
}

/// Environment gate — check for missing runtimes after scope
pub fn environment_gate(config: &Config, lisa_root: &Path) -> Result<bool> {
    let env_file = lisa_root.join("spiral/pass-0/environment-resolution.md");

    if !env_file.exists()
        || std::fs::metadata(&env_file)
            .map(|m| m.len() == 0)
            .unwrap_or(true)
    {
        return Ok(true); // No issues
    }

    if !config.review.pause {
        terminal::log_warn(
            "Environment gate skipped (pause = false) — proceeding with possible missing tooling",
        );
        return Ok(true);
    }

    println!();
    terminal::print_separator();
    terminal::println_bold("  ENVIRONMENT RESOLUTION REQUIRED");
    terminal::print_separator();
    println!();
    println!("  The scope agent detected missing runtimes or toolchains.");
    println!("  Details: {}", env_file.display());
    println!();

    if let Ok(content) = std::fs::read_to_string(&env_file) {
        terminal::println_colored(&content, Color::Yellow);
    }

    println!();
    println!("  [F] FIX  — install the missing runtimes/tooling yourself, then press Enter");
    println!("  [S] SKIP — proceed without fixing (risk of build failures)");
    println!();
    terminal::print_separator();
    println!();

    loop {
        print!("  Your choice [F/S]: ");
        io::stdout().flush()?;
        let mut choice = String::new();
        read_stdin_line(&mut choice)?;
        match choice.trim().to_uppercase().as_str() {
            "F" => {
                terminal::log_info("FIX — install the missing tooling, then press Enter.");
                print!("  Press ENTER when you've installed the missing tooling... ");
                io::stdout().flush()?;
                let mut _buf = String::new();
                read_stdin_line(&mut _buf)?;
                println!();
                return Ok(true);
            }
            "S" => {
                terminal::log_warn("SKIP — proceeding with possible missing tooling.");
                return Ok(true);
            }
            _ => println!("  Please enter F or S."),
        }
    }
}

// --- Extraction helpers ---

/// Extract the primary question/problem statement from acceptance-criteria.md.
pub fn extract_primary_question_from(content: &str) -> Option<String> {
    let headings = ["## Primary Question", "## Problem Statement", "## Question"];
    for heading in &headings {
        if let Some(line) = extract_section_first_line(content, heading) {
            return Some(line);
        }
    }
    None
}

/// Extract first N non-empty lines after ## Success Criteria or ## Acceptance Criteria.
pub fn extract_acceptance_lines(content: &str, max: usize) -> Vec<String> {
    let headings = [
        "## Success Criteria",
        "## Acceptance Criteria",
        "## Criteria",
    ];
    for heading in &headings {
        let mut found = false;
        let mut lines = Vec::new();
        for line in content.lines() {
            if line.starts_with(heading) {
                found = true;
                continue;
            }
            if found {
                // Stop at next heading
                if line.starts_with("## ") {
                    break;
                }
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    lines.push(trimmed.to_string());
                    if lines.len() >= max {
                        break;
                    }
                }
            }
        }
        if !lines.is_empty() {
            return lines;
        }
    }
    Vec::new()
}

/// Extract the first line after ## Recommended Approach / ## Selected Approach / ## Approach.
pub fn extract_methodology_approach_from(content: &str) -> Option<String> {
    let headings = [
        "## Recommended Approach",
        "## Selected Approach",
        "## Approach",
    ];
    for heading in &headings {
        if let Some(line) = extract_section_first_line(content, heading) {
            return Some(line);
        }
    }
    None
}

fn extract_stack_info(agents_content: &str) -> Option<String> {
    let mut found = false;
    for line in agents_content.lines() {
        if line.contains("Language & Runtime") {
            found = true;
            continue;
        }
        if found && !line.trim().is_empty() && !line.starts_with('#') {
            let text = line.trim();
            if !text.contains("To be resolved") {
                return Some(text.to_string());
            }
            return None;
        }
    }
    None
}

fn display_review_summary(content: &str, _pass: u32) {
    // Extract current answer
    if let Some(answer) = extract_section_first_line(content, "## Current Answer") {
        terminal::print_bold("  Answer: ");
        println!("{}", answer);
    }

    // Extract progress
    if let Some(progress) = extract_section_first_line(content, "## Progress") {
        terminal::print_bold("  Progress: ");
        println!("{}", progress);
    }

    // Extract test summary
    for line in content.lines() {
        if line.starts_with("Bounds:") {
            terminal::print_bold("  Tests: ");
            println!("{}", line);
            break;
        }
    }

    // Extract sanity checks
    for line in content.lines() {
        if line.to_lowercase().contains("sanity checks:") && !line.starts_with('#') {
            let info = line.split(':').next_back().unwrap_or("").trim();
            terminal::print_bold("  Sanity: ");
            println!("{}", info);
            break;
        }
    }

    // Engineering Judgment (HUMAN REVIEW)
    let judgment_lines = extract_section_lines(content, "## Engineering Judgment", 5);
    if !judgment_lines.is_empty() {
        println!();
        terminal::print_bold("  Engineering Judgment (HUMAN REVIEW):\n");
        for line in &judgment_lines {
            println!("    {}", line);
        }
    }

    // Status Assessment
    if let Some(status) = extract_section_first_line(content, "## Status Assessment") {
        println!();
        terminal::print_bold("  Status: ");
        println!("{}", status);
    }
}

/// Extract up to `max_lines` non-empty lines from a section, stopping at next `##` heading.
fn extract_section_lines(content: &str, heading: &str, max_lines: usize) -> Vec<String> {
    let mut found = false;
    let mut lines = Vec::new();
    for line in content.lines() {
        if line.starts_with(heading) {
            found = true;
            continue;
        }
        if found {
            if line.starts_with("## ") {
                break;
            }
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                lines.push(trimmed.to_string());
                if lines.len() >= max_lines {
                    break;
                }
            }
        }
    }
    lines
}

/// Extract the first non-empty line after a given heading.
pub fn extract_section_first_line(content: &str, heading: &str) -> Option<String> {
    let mut found = false;
    for line in content.lines() {
        if line.starts_with(heading) {
            found = true;
            continue;
        }
        if found && !line.trim().is_empty() {
            return Some(line.trim().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_primary_question_from() {
        let content = "# Acceptance Criteria\n\n## Primary Question\n\nWhat is the Reynolds number?\n\n## Success Criteria\n";
        assert_eq!(
            extract_primary_question_from(content),
            Some("What is the Reynolds number?".to_string())
        );
    }

    #[test]
    fn test_extract_primary_question_problem_statement() {
        let content = "# Criteria\n\n## Problem Statement\n\nCalculate drag coefficient.\n";
        assert_eq!(
            extract_primary_question_from(content),
            Some("Calculate drag coefficient.".to_string())
        );
    }

    #[test]
    fn test_extract_primary_question_missing() {
        let content = "# Criteria\n\n## Other Section\n\nSome text.\n";
        assert_eq!(extract_primary_question_from(content), None);
    }

    #[test]
    fn test_extract_acceptance_lines() {
        let content = "# Doc\n\n## Success Criteria\n\n- Accuracy within 1%\n- Validated against reference\n- Clean convergence\n\n## Other\n";
        let lines = extract_acceptance_lines(content, 5);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "- Accuracy within 1%");
        assert_eq!(lines[1], "- Validated against reference");
    }

    #[test]
    fn test_extract_acceptance_lines_limited() {
        let content = "## Acceptance Criteria\n\n- A\n- B\n- C\n- D\n- E\n- F\n";
        let lines = extract_acceptance_lines(content, 3);
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_extract_acceptance_lines_stops_at_heading() {
        let content = "## Success Criteria\n\n- A\n- B\n\n## Next Section\n\n- C\n";
        let lines = extract_acceptance_lines(content, 10);
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_extract_methodology_approach_from() {
        let content = "# Methodology\n\n## Recommended Approach\n\nFinite element analysis with mesh refinement.\n\n## Details\n";
        assert_eq!(
            extract_methodology_approach_from(content),
            Some("Finite element analysis with mesh refinement.".to_string())
        );
    }

    #[test]
    fn test_extract_methodology_approach_fallback() {
        let content = "# Methodology\n\n## Approach\n\nDirect numerical simulation.\n";
        assert_eq!(
            extract_methodology_approach_from(content),
            Some("Direct numerical simulation.".to_string())
        );
    }

    #[test]
    fn test_extract_section_first_line() {
        let content = "## Current Answer\n\nRe = 1.23e5 +/- 2.1%\n\n## Progress\n";
        assert_eq!(
            extract_section_first_line(content, "## Current Answer"),
            Some("Re = 1.23e5 +/- 2.1%".to_string())
        );
    }

    #[test]
    fn test_extract_section_first_line_missing() {
        let content = "## Other\n\nSome text.\n";
        assert_eq!(
            extract_section_first_line(content, "## Current Answer"),
            None
        );
    }

    #[test]
    fn test_extract_stack_info() {
        let content = "# Agents\n\n## Language & Runtime\n\nPython 3.11 with NumPy/SciPy\n";
        assert_eq!(
            extract_stack_info(content),
            Some("Python 3.11 with NumPy/SciPy".to_string())
        );
    }

    #[test]
    fn test_extract_stack_info_unresolved() {
        let content = "# Agents\n\n## Language & Runtime\n\nTo be resolved during scoping\n";
        assert_eq!(extract_stack_info(content), None);
    }

    #[test]
    fn test_extract_section_lines() {
        let content = "## Engineering Judgment\n\n1. Check A\n2. Check B\n3. Check C\n\n## Status Assessment\n\nAll good.\n";
        let lines = extract_section_lines(content, "## Engineering Judgment", 5);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "1. Check A");
        assert_eq!(lines[2], "3. Check C");
    }

    #[test]
    fn test_extract_section_lines_max_limit() {
        let content = "## Engineering Judgment\n\n1. A\n2. B\n3. C\n4. D\n5. E\n6. F\n";
        let lines = extract_section_lines(content, "## Engineering Judgment", 3);
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_extract_section_lines_stops_at_heading() {
        let content = "## Engineering Judgment\n\n1. A\n2. B\n\n## Next Section\n\n3. C\n";
        let lines = extract_section_lines(content, "## Engineering Judgment", 10);
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_extract_section_lines_missing() {
        let content = "## Other\n\nSome text.\n";
        let lines = extract_section_lines(content, "## Engineering Judgment", 5);
        assert!(lines.is_empty());
    }
}
