use anyhow::Result;
use crossterm::style::Color;
use std::io::{self, Write};
use std::path::Path;

use crate::config::Config;
use crate::terminal;

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub enum ReviewDecision {
    Accept,
    Continue,
    Redirect,
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

    // Show file locations
    terminal::print_colored("  Methodology:       ", Color::Cyan);
    println!("{}/methodology/methodology.md", lisa_root.display());
    terminal::print_colored("  Plan:              ", Color::Cyan);
    println!("{}/methodology/plan.md", lisa_root.display());
    terminal::print_colored("  Acceptance:        ", Color::Cyan);
    println!(
        "{}/spiral/pass-0/acceptance-criteria.md",
        lisa_root.display()
    );
    terminal::print_colored("  Scope progression: ", Color::Cyan);
    println!("{}/spiral/pass-0/spiral-plan.md", lisa_root.display());
    terminal::print_colored("  Validation:        ", Color::Cyan);
    println!(
        "{}/spiral/pass-0/validation-strategy.md",
        lisa_root.display()
    );

    // Show technology stack if available
    let agents_path = lisa_root.join("AGENTS.md");
    if agents_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&agents_path) {
            if let Some(stack) = extract_stack_info(&content) {
                println!();
                terminal::print_colored("  Stack: ", Color::Cyan);
                println!("{}", stack);
            }
        }
    }

    // Show spiral plan summary
    let spiral_plan = lisa_root.join("spiral/pass-0/spiral-plan.md");
    if spiral_plan.exists() {
        if let Ok(content) = std::fs::read_to_string(&spiral_plan) {
            let pass_lines: Vec<&str> = content
                .lines()
                .filter(|l| {
                    (l.starts_with("| ") && l.contains("Pass"))
                        || (l.starts_with("| ")
                            && l.chars().nth(2).map_or(false, |c| c.is_ascii_digit()))
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

    // Show methodology sections
    let method_path = lisa_root.join("methodology/methodology.md");
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

    println!();
    terminal::print_colored("  [A]", Color::Green);
    println!(" APPROVE  — proceed to Pass 1");
    terminal::print_colored("  [R]", Color::Yellow);
    println!(" REFINE   — provide feedback, re-run scope agent");
    terminal::print_colored("  [E]", Color::Cyan);
    println!(" EDIT     — I'll edit the files directly, then approve");
    terminal::print_colored("  [Q]", Color::Red);
    println!(" QUIT     — stop here");
    println!();
    terminal::print_separator();
    println!();

    loop {
        print!("  Choice: ");
        io::stdout().flush()?;
        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        match choice.trim().to_lowercase().as_str() {
            "a" => return Ok(ScopeDecision::Approve),
            "r" => return Ok(ScopeDecision::Refine),
            "e" => return Ok(ScopeDecision::Edit),
            "q" => return Ok(ScopeDecision::Quit),
            _ => println!("  Invalid choice. Enter A, R, E, or Q."),
        }
    }
}

/// Pass review gate — after each pass's validate phase
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
    println!("    Plots:      {}/plots/REVIEW.md", lisa_root.display());
    println!();

    terminal::print_colored("  [A]", Color::Green);
    println!(" ACCEPT — produce final report");
    terminal::print_colored("  [C]", Color::Yellow);
    println!(" CONTINUE — next spiral pass");
    terminal::print_colored("  [R]", Color::Cyan);
    println!(" REDIRECT — provide guidance (opens $EDITOR)");
    println!();
    terminal::print_separator();
    println!();

    loop {
        print!("  Your choice [A/C/R]: ");
        io::stdout().flush()?;
        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        match choice.trim().to_uppercase().as_str() {
            "A" => {
                terminal::log_success("ACCEPTED — producing final output.");
                return Ok(ReviewDecision::Accept);
            }
            "C" => {
                terminal::log_info("CONTINUE — proceeding to next pass.");
                return Ok(ReviewDecision::Continue);
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

                let editor = std::env::var("EDITOR").unwrap_or_else(|_| {
                    std::env::var("VISUAL").unwrap_or_else(|_| "vi".to_string())
                });

                let _ = std::process::Command::new(&editor)
                    .arg(&redirect_path)
                    .status();

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
            _ => println!("  Please enter A, C, or R."),
        }
    }
}

/// Block gate — when build loop stalls or all remaining tasks are blocked
pub fn block_gate(config: &Config, _pass: u32, plan_path: &Path) -> Result<BlockDecision> {
    if !config.review.pause {
        terminal::log_warn("Block gate skipped (pause = false) — defaulting to SKIP");
        return Ok(BlockDecision::Skip);
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

    terminal::print_colored("  [F]", Color::Green);
    println!(" FIX — resolve blocks, then resume build");
    terminal::print_colored("  [S]", Color::Yellow);
    println!(" SKIP — continue to next phase");
    terminal::print_colored("  [X]", Color::Red);
    println!(" ABORT — stop this spiral pass");
    println!();
    terminal::print_separator();
    println!();

    loop {
        print!("  Your choice [F/S/X]: ");
        io::stdout().flush()?;
        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        match choice.trim().to_uppercase().as_str() {
            "F" => {
                terminal::log_info(
                    "FIX — resolve blocks in methodology/plan.md, then build resumes.",
                );
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
    println!("  [F] FIX — I'll install the missing runtimes/tooling. Press Enter when ready.");
    println!("  [S] SKIP — Proceed anyway. I accept the risk of build failures.");
    println!();
    terminal::print_separator();
    println!();

    loop {
        print!("  Your choice [F/S]: ");
        io::stdout().flush()?;
        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        match choice.trim().to_uppercase().as_str() {
            "F" => {
                terminal::log_info("FIX — install the missing tooling, then press Enter.");
                print!("  Press ENTER when you've installed the missing tooling... ");
                io::stdout().flush()?;
                let mut _buf = String::new();
                io::stdin().read_line(&mut _buf)?;
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
        if line.starts_with("DDV:") {
            terminal::print_bold("  Tests: ");
            println!("{}", line);
            break;
        }
    }

    // Extract sanity checks
    for line in content.lines() {
        if line.to_lowercase().contains("sanity checks:") && !line.starts_with('#') {
            let info = line.split(':').last().unwrap_or("").trim();
            terminal::print_bold("  Sanity: ");
            println!("{}", info);
            break;
        }
    }

    // Extract recommendation
    if let Some(rec) = extract_section_first_line(content, "## Recommendation") {
        println!();
        terminal::print_bold("  Agent recommends: ");
        println!("{}", rec);
    }
}

fn extract_section_first_line(content: &str, heading: &str) -> Option<String> {
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
