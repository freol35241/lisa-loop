use anyhow::{Context, Result};
use std::process::Command;

use crate::config::Config;
use crate::terminal;

pub fn commit_all(msg: &str, config: &Config) -> Result<bool> {
    if !config.git.auto_commit {
        terminal::log_info("Skipping commit (auto_commit = false)");
        return Ok(false);
    }

    terminal::log_info("Staging all changes...");

    let status = Command::new("git")
        .args(["add", "-A"])
        .status()
        .context("Failed to run git add")?;

    if !status.success() {
        terminal::log_warn("git add failed");
        return Ok(false);
    }

    // Check if there are staged changes
    let diff = Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .status()
        .context("Failed to run git diff")?;

    if diff.success() {
        terminal::log_info("No changes to commit.");
        return Ok(false);
    }

    terminal::log_info(&format!("Committing: {}", msg));

    let status = Command::new("git")
        .args(["commit", "-m", msg])
        .status()
        .context("Failed to run git commit")?;

    if status.success() {
        terminal::log_success("Commit created.");
        Ok(true)
    } else {
        terminal::log_warn("git commit failed");
        Ok(false)
    }
}

pub fn push(config: &Config) -> Result<()> {
    if !config.git.auto_push {
        terminal::log_info("Skipping push (auto_push = false)");
        return Ok(());
    }

    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .context("Failed to get current branch")?;

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    terminal::log_info(&format!("Pushing to origin/{}...", branch));

    let status = Command::new("git")
        .args(["push", "-u", "origin", &branch])
        .status()
        .context("Failed to run git push")?;

    if status.success() {
        terminal::log_success("Push complete.");
    } else {
        terminal::log_warn("git push failed");
    }

    Ok(())
}

pub fn is_git_repo() -> bool {
    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if files in a path have been modified (unstaged changes)
pub fn has_modifications(path: &str) -> Result<bool> {
    let output = Command::new("git")
        .args(["diff", "--name-only", path])
        .output()
        .context("Failed to run git diff")?;

    let changed = String::from_utf8_lossy(&output.stdout);
    Ok(!changed.trim().is_empty())
}

/// Revert changes to a specific path
pub fn checkout_path(path: &str) -> Result<()> {
    Command::new("git")
        .args(["checkout", "--", path])
        .status()
        .context("Failed to run git checkout")?;
    Ok(())
}
