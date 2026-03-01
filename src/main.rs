mod agent;
mod cli;
mod config;
mod enforcement;
mod git;
mod init;
mod orchestrator;
mod prompt;
mod review;
mod state;
mod tasks;
mod terminal;
mod usage;

use anyhow::Result;
use clap::Parser;
use crossterm::style::Color;
use std::path::PathBuf;

fn project_root() -> PathBuf {
    std::env::current_dir().expect("Failed to get current directory")
}

fn load_config() -> Result<config::Config> {
    let root = project_root();
    config::Config::load(&root)
}

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::Init { name, tech } => {
            init::scaffold::run(&project_root(), name, tech)
        }
        cli::Commands::Run {
            max_passes,
            no_pause,
            verbose,
        } => {
            let mut config = load_config()?;
            if verbose {
                config.terminal.collapse_output = false;
            }
            orchestrator::run(&config, &project_root(), max_passes, no_pause)
        }
        cli::Commands::Resume => {
            let config = load_config()?;
            orchestrator::resume(&config, &project_root())
        }
        cli::Commands::Scope => {
            let config = load_config()?;
            orchestrator::run_scope_only(&config, &project_root())
        }
        cli::Commands::Status => cmd_status(),
        cli::Commands::Doctor => cmd_doctor(),
        cli::Commands::Finalize => {
            let config = load_config()?;
            let lisa_root = config.lisa_root(&project_root());
            let state = state::load_state(&lisa_root)?;
            match state {
                state::SpiralState::PassReview { pass }
                | state::SpiralState::InPass { pass, .. } => {
                    orchestrator::finalize(&config, &project_root(), pass)
                }
                state::SpiralState::Complete { final_pass } => {
                    terminal::log_info(&format!("Spiral already complete at pass {}.", final_pass));
                    Ok(())
                }
                _ => {
                    terminal::log_error("Cannot finalize: no pass has been completed yet.");
                    Ok(())
                }
            }
        }
        cli::Commands::EjectPrompts => cmd_eject_prompts(),
        cli::Commands::History => cmd_history(),
        cli::Commands::Rollback { pass, force } => {
            let config = load_config()?;
            orchestrator::rollback(&config, &project_root(), pass, force)
        }
        cli::Commands::Continue {
            question,
            max_passes,
            no_pause,
            verbose,
        } => {
            let mut config = load_config()?;
            if verbose {
                config.terminal.collapse_output = false;
            }
            orchestrator::continue_spiral(&config, &project_root(), &question, max_passes, no_pause)
        }
    }
}

fn cmd_status() -> Result<()> {
    let root = project_root();
    let lisa_root = match load_config() {
        Ok(config) => config.lisa_root(&root),
        Err(_) => root.join(".lisa"),
    };

    if !lisa_root.exists() {
        terminal::log_error("No .lisa/ directory found. Run `lisa init` first.");
        return Ok(());
    }

    let state = state::load_state(&lisa_root)?;

    println!();
    terminal::println_bold("Lisa Loop — Current Status");
    println!();

    match &state {
        state::SpiralState::NotStarted => {
            println!("  State: Not started");
            println!("  Next:  lisa scope   (or lisa run)");
        }
        _ => {
            println!("  State: {}", state);

            // Check if spiral is complete
            if lisa_root.join("spiral/SPIRAL_COMPLETE.md").exists() {
                println!();
                terminal::println_colored("  Spiral COMPLETE — answer accepted.", Color::Green);

                // Show follow-up count
                let assignment_path = root.join("ASSIGNMENT.md");
                if let Ok(assignment_content) = std::fs::read_to_string(&assignment_path) {
                    let follow_ups = assignment_content
                        .lines()
                        .filter(|l| l.starts_with("## Follow-up "))
                        .count();
                    if follow_ups > 0 {
                        println!("  Follow-ups: {}", follow_ups);
                    }
                }
            }

            // Show pass artifacts
            println!();
            println!("  Pass artifacts:");
            let spiral_dir = lisa_root.join("spiral");
            if spiral_dir.exists() {
                let mut entries: Vec<_> = std::fs::read_dir(&spiral_dir)?
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.path().is_dir()
                            && e.file_name()
                                .to_str()
                                .map(|n| n.starts_with("pass-"))
                                .unwrap_or(false)
                    })
                    .collect();
                entries.sort_by_key(|e| e.file_name());
                for entry in entries {
                    let name = entry.file_name();
                    let complete = entry.path().join("PASS_COMPLETE.md").exists();
                    let marker = if complete { " ✓" } else { "" };
                    println!("    {}{}", name.to_string_lossy(), marker);
                }
            }

            // Show task status
            let plan_path = lisa_root.join("methodology/plan.md");
            if plan_path.exists() {
                let counts = tasks::count_tasks_by_status(&plan_path)?;
                println!();
                println!(
                    "  Task status: TODO={} IN_PROGRESS={} DONE={} BLOCKED={}",
                    counts.todo, counts.in_progress, counts.done, counts.blocked
                );
            }

            // Show usage summary
            let ledger = usage::load_usage(&lisa_root)?;
            if ledger.invocation_count() > 0 {
                println!();
                println!(
                    "  Cost: ${:.4} ({} invocations, {} input + {} output tokens)",
                    ledger.total_cost(),
                    ledger.invocation_count(),
                    ledger.total_input_tokens(),
                    ledger.total_output_tokens(),
                );
            }

            // Show rollback points
            let tags = git::list_pass_tags();
            if !tags.is_empty() {
                let tag_strs: Vec<String> = tags.iter().map(|t| t.to_string()).collect();
                println!("  Rollback points: pass {}", tag_strs.join(", "));
            }
        }
    }
    println!();
    Ok(())
}

fn cmd_doctor() -> Result<()> {
    println!();
    terminal::println_bold("Lisa Loop — Environment Check");
    println!();

    // Check git
    let git_ok = git::is_git_repo();
    if git_ok {
        terminal::print_colored("  ✓", Color::Green);
        println!(" Git repository detected");
    } else {
        terminal::print_colored("  ✗", Color::Red);
        println!(" Not a git repository (run: git init)");
    }

    // Check git user.name
    let git_name = std::process::Command::new("git")
        .args(["config", "user.name"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty());
    if let Some(name) = &git_name {
        terminal::print_colored("  ✓", Color::Green);
        println!(" Git user.name: {}", name);
    } else {
        terminal::print_colored("  ✗", Color::Red);
        println!(" Git user.name not set (run: git config --global user.name \"Your Name\")");
    }

    // Check git user.email
    let git_email = std::process::Command::new("git")
        .args(["config", "user.email"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty());
    if let Some(email) = &git_email {
        terminal::print_colored("  ✓", Color::Green);
        println!(" Git user.email: {}", email);
    } else {
        terminal::print_colored("  ✗", Color::Red);
        println!(" Git user.email not set (run: git config --global user.email \"you@example.com\")");
    }

    // Check claude CLI
    let claude_ok = std::process::Command::new("claude")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if claude_ok {
        terminal::print_colored("  ✓", Color::Green);
        println!(" Claude CLI found");
    } else {
        terminal::print_colored("  ✗", Color::Red);
        println!(" Claude CLI not found (install: npm install -g @anthropic-ai/claude-code)");
    }

    // Check claude authentication
    if claude_ok {
        let auth_ok = std::process::Command::new("claude")
            .args(["auth", "status"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .and_then(|o| {
                let json: serde_json::Value =
                    serde_json::from_slice(&o.stdout).ok()?;
                json.get("loggedIn")?.as_bool()
            })
            .unwrap_or(false);
        if auth_ok {
            terminal::print_colored("  ✓", Color::Green);
            println!(" Claude CLI authenticated");
        } else {
            terminal::print_colored("  ✗", Color::Red);
            println!(" Claude CLI not authenticated (run: claude auth login)");
        }
    }

    // Check .lisa directory
    let root = project_root();
    let lisa_root = match load_config() {
        Ok(ref config) => config.lisa_root(&root),
        Err(_) => root.join(".lisa"),
    };
    let lisa_exists = lisa_root.exists();
    if lisa_exists {
        terminal::print_colored("  ✓", Color::Green);
        println!(" {} directory exists", lisa_root.display());

        // Check config
        match load_config() {
            Ok(_) => {
                terminal::print_colored("  ✓", Color::Green);
                println!(" lisa.toml is valid");
            }
            Err(e) => {
                terminal::print_colored("  ✗", Color::Red);
                println!(" lisa.toml error: {}", e);
            }
        }

        // Check ASSIGNMENT.md (lives in project root, not .lisa/)
        let assignment = root.join("ASSIGNMENT.md");
        if assignment.exists() {
            terminal::print_colored("  ✓", Color::Green);
            println!(" ASSIGNMENT.md exists");
        } else {
            terminal::print_colored("  ✗", Color::Red);
            println!(" ASSIGNMENT.md missing");
        }
    } else {
        terminal::print_colored("  ○", Color::Yellow);
        println!(" .lisa/ not found (run: lisa init)");
    }

    println!();
    Ok(())
}

fn cmd_eject_prompts() -> Result<()> {
    let root = project_root();
    let lisa_root = root.join(".lisa");

    if !lisa_root.exists() {
        terminal::log_error("No .lisa/ directory found. Run `lisa init` first.");
        return Ok(());
    }

    let prompts_dir = lisa_root.join("prompts");
    std::fs::create_dir_all(&prompts_dir)?;

    let prompts = [
        ("scope.md", prompt::PROMPT_SCOPE),
        ("refine.md", prompt::PROMPT_REFINE),
        ("ddv_red.md", prompt::PROMPT_DDV_RED),
        ("build.md", prompt::PROMPT_BUILD),
        ("execute.md", prompt::PROMPT_EXECUTE),
        ("validate.md", prompt::PROMPT_VALIDATE),
        ("finalize.md", prompt::PROMPT_FINALIZE),
    ];

    for (filename, content) in &prompts {
        let path = prompts_dir.join(filename);
        if path.exists() {
            terminal::log_warn(&format!("  Skipping {} (already exists)", filename));
        } else {
            std::fs::write(&path, content)?;
            terminal::log_success(&format!("  Written {}", filename));
        }
    }

    println!();
    terminal::log_info("Prompts ejected to .lisa/prompts/");
    terminal::log_info("Edit them freely — the CLI will use local prompts when present.");
    println!();

    Ok(())
}

fn cmd_history() -> Result<()> {
    let root = project_root();
    let lisa_root = match load_config() {
        Ok(config) => config.lisa_root(&root),
        Err(_) => root.join(".lisa"),
    };

    if !lisa_root.exists() {
        terminal::log_error("No .lisa/ directory found. Run `lisa init` first.");
        return Ok(());
    }

    let spiral_dir = lisa_root.join("spiral");
    if !spiral_dir.exists() {
        terminal::log_error("No spiral directory found. Run `lisa run` first.");
        return Ok(());
    }

    // Collect pass directories (skip pass-0)
    let mut passes: Vec<u32> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&spiral_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if let Some(num_str) = name_str.strip_prefix("pass-") {
                if let Ok(num) = num_str.parse::<u32>() {
                    if num > 0 {
                        passes.push(num);
                    }
                }
            }
        }
    }

    passes.sort();

    if passes.is_empty() {
        terminal::log_info("No completed passes found (only pass-0 exists).");
        return Ok(());
    }

    let ledger = usage::load_usage(&lisa_root).unwrap_or_default();

    println!();
    terminal::println_bold("Lisa Loop — Pass History");
    println!();

    // Header
    println!(
        "  {:>4}  {:<30}  {:<8}  {:<7}  {:<8}  Recommendation",
        "Pass", "Answer", "DDV", "Sanity", "Cost"
    );
    println!(
        "  {:>4}  {:<30}  {:<8}  {:<7}  {:<8}  --------------",
        "----", "------------------------------", "--------", "-------", "--------"
    );

    for pass in &passes {
        let review_path = lisa_root.join(format!("spiral/pass-{}/review-package.md", pass));
        if !review_path.exists() {
            continue;
        }
        let content = match std::fs::read_to_string(&review_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let answer = review::extract_section_first_line(&content, "## Current Answer")
            .unwrap_or_else(|| "-".to_string());
        let answer_trunc = truncate_str(&answer, 30);

        let ddv = extract_ddv_summary(&content).unwrap_or_else(|| "-".to_string());
        let ddv_trunc = truncate_str(&ddv, 8);

        let sanity = extract_sanity_summary(&content).unwrap_or_else(|| "-".to_string());
        let sanity_trunc = truncate_str(&sanity, 7);

        let rec = review::extract_section_first_line(&content, "## Recommendation")
            .unwrap_or_else(|| "-".to_string());

        let cost = ledger.pass_cost(*pass);
        let cost_str = if cost > 0.0 {
            format!("${:.4}", cost)
        } else {
            "-".to_string()
        };
        let cost_trunc = truncate_str(&cost_str, 8);

        println!(
            "  {:>4}  {:<30}  {:<8}  {:<7}  {:<8}  {}",
            pass, answer_trunc, ddv_trunc, sanity_trunc, cost_trunc, rec
        );
    }

    println!();

    Ok(())
}

/// Extract DDV test result summary (e.g., "3/4") from review content.
fn extract_ddv_summary(content: &str) -> Option<String> {
    for line in content.lines() {
        if line.starts_with("DDV:") {
            // Try to extract a fraction like "3/4" or "passed: 3/4"
            let text = line.trim_start_matches("DDV:").trim();
            // Look for N/M pattern
            if let Some(frac) = extract_fraction(text) {
                return Some(frac);
            }
            return Some(truncate_str(text, 8));
        }
    }
    None
}

/// Extract sanity check summary from review content.
fn extract_sanity_summary(content: &str) -> Option<String> {
    for line in content.lines() {
        if line.to_lowercase().contains("sanity checks:") && !line.starts_with('#') {
            let info = line.split(':').next_back().unwrap_or("").trim();
            if let Some(frac) = extract_fraction(info) {
                return Some(frac);
            }
            return Some(truncate_str(info, 7));
        }
    }
    None
}

/// Find a "N/M" fraction pattern in text.
fn extract_fraction(text: &str) -> Option<String> {
    let re = regex::Regex::new(r"(\d+)\s*/\s*(\d+)").unwrap();
    re.captures(text)
        .map(|caps| format!("{}/{}", &caps[1], &caps[2]))
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else if max > 3 {
        format!("{}...", &s[..max - 3])
    } else {
        s[..max].to_string()
    }
}
