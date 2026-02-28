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

    // Stack (from AGENTS.md)
    let agents_path = lisa_root.join("AGENTS.md");
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

    // DDV cases (from validation-strategy.md)
    let validation_path = lisa_root.join("spiral/pass-0/validation-strategy.md");
    if validation_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&validation_path) {
            let ddv_count = count_verification_cases_from(&content);
            if ddv_count > 0 {
                terminal::print_colored("  DDV cases:", Color::Cyan);
                println!(" {}", ddv_count);
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
    println!(
        "methodology.md, plan.md, acceptance-criteria.md, spiral-plan.md, validation-strategy.md"
    );

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

/// Count `### V0-` and `### V1-` headings in validation-strategy.md.
pub fn count_verification_cases_from(content: &str) -> u32 {
    content
        .lines()
        .filter(|l| {
            let trimmed = l.trim_start_matches('#').trim();
            trimmed.starts_with("V0-") || trimmed.starts_with("V1-")
        })
        .count() as u32
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
            let info = line.split(':').next_back().unwrap_or("").trim();
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
    fn test_count_verification_cases_from() {
        let content = "# Validation\n\n### V0-basic-check\nDetails...\n\n### V0-boundary\nDetails...\n\n### V1-convergence\nDetails...\n\n## Other\n";
        assert_eq!(count_verification_cases_from(content), 3);
    }

    #[test]
    fn test_count_verification_cases_none() {
        let content = "# Validation\n\n## Some Section\n\nNo verification headings.\n";
        assert_eq!(count_verification_cases_from(content), 0);
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
}
