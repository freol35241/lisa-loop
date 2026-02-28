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

/// Check if files in a path have been modified (unstaged or staged changes)
pub fn has_any_modifications(path: &str) -> Result<bool> {
    // Check unstaged
    let unstaged = Command::new("git")
        .args(["diff", "--name-only", path])
        .output()
        .context("Failed to run git diff")?;
    let unstaged_files = String::from_utf8_lossy(&unstaged.stdout);
    if !unstaged_files.trim().is_empty() {
        return Ok(true);
    }
    // Check staged
    let staged = Command::new("git")
        .args(["diff", "--cached", "--name-only", path])
        .output()
        .context("Failed to run git diff --cached")?;
    let staged_files = String::from_utf8_lossy(&staged.stdout);
    Ok(!staged_files.trim().is_empty())
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

/// Unstage changes to a specific path
pub fn reset_path(path: &str) -> Result<()> {
    Command::new("git")
        .args(["reset", "HEAD", "--", path])
        .status()
        .context("Failed to run git reset")?;
    Ok(())
}

/// Revert changes to a specific path
pub fn checkout_path(path: &str) -> Result<()> {
    Command::new("git")
        .args(["checkout", "--", path])
        .status()
        .context("Failed to run git checkout")?;
    Ok(())
}

/// Create a lightweight git tag (delete-then-create for idempotency).
pub fn create_tag(name: &str) -> Result<()> {
    // Delete existing tag if present (ignore errors)
    let _ = Command::new("git").args(["tag", "-d", name]).output();
    let status = Command::new("git")
        .args(["tag", name])
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

/// Retrieve a file from another branch via `git show <branch>:<path>`.
pub fn show_file_from_ref(git_ref: &str, path: &str) -> Result<Option<String>> {
    let output = Command::new("git")
        .args(["show", &format!("{}:{}", git_ref, path)])
        .output()
        .context("Failed to run git show")?;
    if output.status.success() {
        Ok(Some(String::from_utf8_lossy(&output.stdout).to_string()))
    } else {
        Ok(None)
    }
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
