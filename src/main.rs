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
            init::resolve_assignment::run(&project_root(), name, tech)
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
                    terminal::log_info(&format!(
                        "Spiral already complete at pass {}.",
                        final_pass
                    ));
                    Ok(())
                }
                _ => {
                    terminal::log_error(
                        "Cannot finalize: no pass has been completed yet.",
                    );
                    Ok(())
                }
            }
        }
        cli::Commands::EjectPrompts => cmd_eject_prompts(),
    }
}

fn cmd_status() -> Result<()> {
    let root = project_root();
    let lisa_root = match load_config() {
        Ok(config) => config.lisa_root(&root),
        Err(_) => root.join(".lisa"),
    };

    if !lisa_root.exists() {
        terminal::log_error("No .lisa/ directory found. Run `lisa init resolve-assignment` first.");
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
                terminal::println_colored(
                    "  Spiral COMPLETE — answer accepted.",
                    Color::Green,
                );
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

        // Check BRIEF.md
        let brief = lisa_root.join("BRIEF.md");
        if brief.exists() {
            terminal::print_colored("  ✓", Color::Green);
            println!(" BRIEF.md exists");
        } else {
            terminal::print_colored("  ✗", Color::Red);
            println!(" BRIEF.md missing");
        }
    } else {
        terminal::print_colored("  ○", Color::Yellow);
        println!(" .lisa/ not found (run: lisa init resolve-assignment)");
    }

    println!();
    Ok(())
}

fn cmd_eject_prompts() -> Result<()> {
    let root = project_root();
    let lisa_root = root.join(".lisa");

    if !lisa_root.exists() {
        terminal::log_error("No .lisa/ directory found. Run `lisa init resolve-assignment` first.");
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
