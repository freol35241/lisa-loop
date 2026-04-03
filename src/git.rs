use anyhow::{Context, Result};
use std::process::Command;

use crate::config::Config;
use crate::terminal;

pub fn commit_all(msg: &str, config: &Config) -> Result<bool> {
    if !config.git.auto_commit {
        terminal::log_info("Skipping commit (auto_commit = false)");
        return Ok(false);
    }

    terminal::log_info("Staging changes...");

    // Stage only deliverable paths (source, tests, config files).
    // .lisa/ is gitignored and never committed.
    let mut paths: Vec<String> = Vec::new();
    paths.extend(config.paths.source.iter().cloned());
    for test_dir in [
        &config.paths.tests_bounds,
        &config.paths.tests_software,
        &config.paths.tests_integration,
    ] {
        if !test_dir.is_empty() {
            paths.push(test_dir.clone());
        }
    }
    // Always include project-root files the agents may modify
    paths.extend(
        ["ASSIGNMENT.md", "lisa.toml", ".gitignore"]
            .iter()
            .map(|s| s.to_string()),
    );

    // Deduplicate
    paths.sort();
    paths.dedup();

    if paths.is_empty() {
        terminal::log_info("No paths configured to stage.");
        return Ok(false);
    }

    let mut args = vec!["add".to_string(), "--".to_string()];
    args.extend(paths);

    let status = Command::new("git")
        .args(&args)
        .status()
        .context("Failed to run git add")?;

    if !status.success() {
        anyhow::bail!("git add failed");
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
        // Unstage files to avoid leaving a dirty index for the next resume
        let _ = Command::new("git").args(["reset", "HEAD", "--"]).status();
        anyhow::bail!("git commit failed")
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
        Ok(())
    } else {
        anyhow::bail!(
            "git push to origin/{} failed. Check remote access and try `lisa resume`.",
            branch
        )
    }
}

pub fn is_git_repo() -> bool {
    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if any source files were modified in the most recent commit.
/// Runs `git diff --name-only HEAD~1 HEAD -- <source_dirs...>` and returns
/// true if any files match.
pub fn source_changed_in_last_commit(source_dirs: &[String]) -> Result<bool> {
    let mut args = vec![
        "diff".to_string(),
        "--name-only".to_string(),
        "HEAD~1".to_string(),
        "HEAD".to_string(),
        "--".to_string(),
    ];
    args.extend(source_dirs.iter().cloned());

    let output = Command::new("git")
        .args(&args)
        .output()
        .context("Failed to run git diff HEAD~1 HEAD")?;

    if !output.status.success() {
        // HEAD~1 may not exist (first commit); treat as no change
        return Ok(false);
    }

    let files = String::from_utf8_lossy(&output.stdout);
    Ok(!files.trim().is_empty())
}

/// Create a lightweight git tag (force-replace for idempotency).
pub fn create_tag(name: &str) -> Result<()> {
    let status = Command::new("git")
        .args(["tag", "-f", name])
        .status()
        .context("Failed to create git tag")?;
    if !status.success() {
        anyhow::bail!("git tag {} failed", name);
    }
    terminal::log_info(&format!("Tagged: {}", name));
    Ok(())
}

/// List pass tags (lisa/pass-*) and return sorted pass numbers.
pub fn list_pass_tags() -> Vec<u32> {
    let output = match Command::new("git")
        .args(["tag", "--list", "lisa/pass-*"])
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return Vec::new(),
    };
    parse_pass_tags(&String::from_utf8_lossy(&output.stdout))
}

/// Parse pass numbers from tag list output.
fn parse_pass_tags(output: &str) -> Vec<u32> {
    let mut tags: Vec<u32> = output
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("lisa/pass-")
                .and_then(|n| n.parse::<u32>().ok())
        })
        .collect();
    tags.sort();
    tags
}

/// Create a branch at current HEAD.
pub fn create_branch(name: &str) -> Result<()> {
    let status = Command::new("git")
        .args(["branch", name])
        .status()
        .context("Failed to create git branch")?;
    if !status.success() {
        anyhow::bail!("git branch {} failed", name);
    }
    Ok(())
}

/// git reset --hard to a target ref.
pub fn reset_hard(target: &str) -> Result<()> {
    let status = Command::new("git")
        .args(["reset", "--hard", target])
        .status()
        .context("Failed to run git reset --hard")?;
    if !status.success() {
        anyhow::bail!("git reset --hard {} failed", target);
    }
    Ok(())
}

/// Check for uncommitted changes (staged or unstaged).
pub fn has_uncommitted_changes() -> Result<bool> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .context("Failed to run git status")?;
    Ok(!String::from_utf8_lossy(&output.stdout).trim().is_empty())
}

/// Checkout a branch or ref.
pub fn checkout(target: &str) -> Result<()> {
    let status = Command::new("git")
        .args(["checkout", target])
        .status()
        .context("Failed to run git checkout")?;
    if !status.success() {
        anyhow::bail!("git checkout {} failed", target);
    }
    Ok(())
}

/// Get the current branch name.
pub fn current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .context("Failed to get current branch")?;
    if !output.status.success() {
        anyhow::bail!("git rev-parse --abbrev-ref HEAD failed");
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Merge a branch into the current branch with --no-ff.
pub fn merge_branch(branch: &str) -> Result<()> {
    let status = Command::new("git")
        .args([
            "merge",
            branch,
            "--no-ff",
            "-m",
            &format!("Merge exploration: {}", branch),
        ])
        .status()
        .context("Failed to run git merge")?;
    if !status.success() {
        anyhow::bail!("git merge {} failed — resolve conflicts manually", branch);
    }
    Ok(())
}

/// Delete a local branch.
pub fn delete_branch(name: &str) -> Result<()> {
    let status = Command::new("git")
        .args(["branch", "-D", name])
        .status()
        .context("Failed to delete git branch")?;
    if !status.success() {
        anyhow::bail!("git branch -D {} failed", name);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pass_tags_normal() {
        let output = "lisa/pass-0\nlisa/pass-1\nlisa/pass-2\n";
        assert_eq!(parse_pass_tags(output), vec![0, 1, 2]);
    }

    #[test]
    fn test_parse_pass_tags_empty() {
        assert_eq!(parse_pass_tags(""), Vec::<u32>::new());
    }

    #[test]
    fn test_parse_pass_tags_unordered() {
        let output = "lisa/pass-3\nlisa/pass-1\nlisa/pass-0\n";
        assert_eq!(parse_pass_tags(output), vec![0, 1, 3]);
    }

    #[test]
    fn test_parse_pass_tags_with_noise() {
        let output = "lisa/pass-0\nother-tag\nlisa/pass-abc\nlisa/pass-2\n";
        assert_eq!(parse_pass_tags(output), vec![0, 2]);
    }
}
