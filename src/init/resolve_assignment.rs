use anyhow::{Context, Result};
use crossterm::style::Color;
use std::io::IsTerminal;
use std::path::Path;

use crate::config::default_config_toml;
use crate::terminal;

// Compiled-in templates
const ASSIGNMENT_TEMPLATE: &str = include_str!("../../templates/assignment.md");
const AGENTS_TEMPLATE: &str = include_str!("../../templates/agents.md");
const METHODOLOGY_TEMPLATE: &str = include_str!("../../templates/methodology.md");
const PLAN_TEMPLATE: &str = include_str!("../../templates/plan.md");
const VERIFICATION_CASES_TEMPLATE: &str = include_str!("../../templates/verification_cases.md");
const OVERVIEW_TEMPLATE: &str = include_str!("../../templates/overview.md");
const ASSUMPTIONS_REGISTER_TEMPLATE: &str = include_str!("../../templates/assumptions_register.md");
const SANITY_CHECKS_TEMPLATE: &str = include_str!("../../templates/sanity_checks.md");
const LIMITING_CASES_TEMPLATE: &str = include_str!("../../templates/limiting_cases.md");
const REFERENCE_DATA_TEMPLATE: &str = include_str!("../../templates/reference_data.md");
const PROGRESS_LOG_TEMPLATE: &str = include_str!("../../templates/progress_log.md");
const PLOTS_REVIEW_TEMPLATE: &str = include_str!("../../templates/plots_review.md");

pub fn run(project_root: &Path, name: Option<String>, tech: Option<String>) -> Result<()> {
    let lisa_root = project_root.join(".lisa");

    if lisa_root.exists() {
        anyhow::bail!(
            ".lisa/ directory already exists. This project has already been initialized."
        );
    }

    // Determine project name
    let project_name = if let Some(n) = name {
        n
    } else {
        // Interactive or infer from directory name
        let dir_name = project_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed")
            .to_string();

        if std::io::stdin().is_terminal() {
            println!();
            terminal::println_bold("  Lisa Loop — Resolve Assignment");
            println!();
            let name: String = dialoguer::Input::new()
                .with_prompt("  Assignment name")
                .default(dir_name)
                .interact_text()?;
            name
        } else {
            dir_name
        }
    };

    // Determine tech preference
    let tech_pref = if let Some(t) = tech {
        t
    } else if std::io::stdin().is_terminal() {
        let pref: String = dialoguer::Input::new()
            .with_prompt("  Technology preference (blank for auto)")
            .allow_empty(true)
            .interact_text()?;
        pref
    } else {
        String::new()
    };

    // Create directory structure
    let dirs = [
        ".lisa",
        ".lisa/methodology",
        ".lisa/methodology/derivations",
        ".lisa/spiral",
        ".lisa/spiral/pass-0",
        ".lisa/validation",
        ".lisa/references/core",
        ".lisa/references/retrieved",
        ".lisa/plots",
        ".lisa/output",
        "src",
        "tests/ddv",
        "tests/software",
        "tests/integration",
    ];

    for dir in &dirs {
        std::fs::create_dir_all(project_root.join(dir))
            .with_context(|| format!("Failed to create directory: {}", dir))?;
    }

    // Write config
    let config_content = default_config_toml(&project_name);
    write_file(&lisa_root.join("lisa.toml"), &config_content)?;

    // Write ASSIGNMENT.md (in project root, not inside .lisa/)
    write_file(&project_root.join("ASSIGNMENT.md"), ASSIGNMENT_TEMPLATE)?;

    // Write AGENTS.md with optional tech preference
    let agents_content = if tech_pref.is_empty() {
        AGENTS_TEMPLATE.to_string()
    } else {
        AGENTS_TEMPLATE.replacen(
            "<!-- State your technology preferences here",
            &format!(
                "{}\n\n<!-- State your technology preferences here",
                tech_pref
            ),
            1,
        )
    };
    write_file(&lisa_root.join("AGENTS.md"), &agents_content)?;

    // Write methodology templates
    write_file(
        &lisa_root.join("methodology/methodology.md"),
        METHODOLOGY_TEMPLATE,
    )?;
    write_file(&lisa_root.join("methodology/plan.md"), PLAN_TEMPLATE)?;
    write_file(
        &lisa_root.join("methodology/verification-cases.md"),
        VERIFICATION_CASES_TEMPLATE,
    )?;
    write_file(
        &lisa_root.join("methodology/overview.md"),
        OVERVIEW_TEMPLATE,
    )?;
    write_file(
        &lisa_root.join("methodology/assumptions-register.md"),
        ASSUMPTIONS_REGISTER_TEMPLATE,
    )?;

    // Write validation templates
    write_file(
        &lisa_root.join("validation/sanity-checks.md"),
        SANITY_CHECKS_TEMPLATE,
    )?;
    write_file(
        &lisa_root.join("validation/limiting-cases.md"),
        LIMITING_CASES_TEMPLATE,
    )?;
    write_file(
        &lisa_root.join("validation/reference-data.md"),
        REFERENCE_DATA_TEMPLATE,
    )?;
    write_file(
        &lisa_root.join("validation/progress-log.md"),
        PROGRESS_LOG_TEMPLATE,
    )?;

    // Write plots review
    write_file(&lisa_root.join("plots/REVIEW.md"), PLOTS_REVIEW_TEMPLATE)?;

    // Write .gitkeep files
    for dir in &[
        ".lisa/methodology/derivations",
        ".lisa/references/core",
        ".lisa/references/retrieved",
        ".lisa/output",
        "tests/ddv",
        "tests/software",
        "tests/integration",
        "src",
    ] {
        let keepfile = project_root.join(dir).join(".gitkeep");
        if !keepfile.exists() {
            std::fs::write(&keepfile, "")?;
        }
    }

    // Write initial state
    crate::state::save_state(&lisa_root, &crate::state::SpiralState::NotStarted)?;

    // Print summary
    println!();
    terminal::println_bold("  Created:");
    terminal::print_colored("    .lisa/lisa.toml              ", Color::Cyan);
    println!("Configuration");
    terminal::print_colored("    ASSIGNMENT.md                ", Color::Cyan);
    println!("Edit with your full assignment");
    terminal::print_colored("    .lisa/references/core/       ", Color::Cyan);
    println!("Add reference papers here");
    terminal::print_colored("    .lisa/methodology/           ", Color::Cyan);
    println!("Process artifacts (auto-managed)");
    terminal::print_colored("    .lisa/spiral/                ", Color::Cyan);
    println!("Spiral state (auto-managed)");
    terminal::print_colored("    .lisa/validation/            ", Color::Cyan);
    println!("V&V artifacts (auto-managed)");
    terminal::print_colored("    .lisa/plots/                 ", Color::Cyan);
    println!("Verification plots");
    terminal::print_colored("    src/                         ", Color::Cyan);
    println!("Implementation code");
    terminal::print_colored("    tests/ddv/                   ", Color::Cyan);
    println!("Domain verification tests");
    terminal::print_colored("    tests/software/              ", Color::Cyan);
    println!("Software quality tests");
    terminal::print_colored("    tests/integration/           ", Color::Cyan);
    println!("End-to-end tests");
    println!();

    terminal::println_bold("  Next steps:");
    println!("    1. Edit ASSIGNMENT.md with the full assignment");
    println!("    2. Add reference papers to .lisa/references/core/");
    println!("    3. Run: lisa run");
    println!();

    Ok(())
}

fn write_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}
