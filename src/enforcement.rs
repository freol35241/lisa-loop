//! Post-hoc DDV isolation enforcement.
//!
//! This module is a **drift detector for common mistakes**, not a security
//! boundary.  Agents run with full filesystem access (`--dangerously-skip-
//! permissions`), so a determined or confused agent can always circumvent
//! these checks.  What we *do* catch:
//!
//! - DDV Red agent reading/writing source files via the Read, Write, or Edit tools
//! - DDV Red agent referencing source dirs in Bash commands (best-effort substring match)
//! - Build agent modifying or adding files in the DDV test directory (git diff + untracked)
//!
//! What we *don't* catch:
//!
//! - Bash commands that reach source files indirectly (e.g. `cd src && cat main.rs`)
//! - Agent reading source content that was piped through a subagent (Task tool)
//! - Agent memorising source content from a previous invocation (can't happen —
//!   agents are stateless between invocations, but worth stating)
//!
//! The goal is to flag the 80% case where an agent drifts from its instructions,
//! not to provide airtight isolation.

use anyhow::Result;
use std::path::Path;

use crate::agent::ToolCall;
use crate::config::Config;
use crate::git;
use crate::terminal;

/// Check that the DDV Red agent didn't access source files (isolation violation).
///
/// Flags Read, Write, Edit, and Bash tool calls that reference source directories.
pub fn verify_ddv_isolation(
    tool_log: &[ToolCall],
    config: &Config,
    project_root: &Path,
) -> Result<()> {
    let source_dirs = &config.paths.source;

    let violations: Vec<&ToolCall> = tool_log
        .iter()
        .filter(|call| match call {
            ToolCall::Read { path } | ToolCall::Write { path } | ToolCall::Edit { path } => {
                is_under_source(path, source_dirs, project_root)
            }
            ToolCall::Bash { command } => command_references_source(command, source_dirs),
            _ => false,
        })
        .collect();

    if !violations.is_empty() {
        for v in &violations {
            terminal::log_error(&format!("  DDV isolation violation: {:?}", v));
        }
        anyhow::bail!(
            "{} source file access violations detected during DDV Red phase. \
             The DDV agent must not read or write implementation source code.",
            violations.len()
        );
    }
    Ok(())
}

/// Check that build agent didn't modify or add DDV test files; revert if so.
pub fn verify_ddv_tests_unmodified(config: &Config) -> Result<()> {
    let tests_ddv = &config.paths.tests_ddv;

    if git::has_any_modifications(tests_ddv)? {
        terminal::log_warn("Build agent modified DDV tests — reverting tracked changes!");
        git::reset_path(tests_ddv)?;
        git::checkout_path(tests_ddv)?;
    }

    let untracked = git::has_untracked_files(tests_ddv)?;
    if !untracked.is_empty() {
        terminal::log_warn(&format!(
            "Build agent added {} new file(s) in DDV test dir — removing!",
            untracked.len()
        ));
        for path in &untracked {
            if let Err(e) = std::fs::remove_file(path) {
                terminal::log_warn(&format!("  Failed to remove {}: {}", path, e));
            }
        }
    }

    Ok(())
}

fn is_under_source(path: &str, source_dirs: &[String], project_root: &Path) -> bool {
    for src in source_dirs {
        let abs_src = project_root.join(src);
        let abs_src_str = abs_src.to_string_lossy();

        // Check both relative and absolute paths
        if path.starts_with(&format!("{}/", src))
            || path.starts_with(&format!("./{}/", src))
            || path.starts_with(&format!("{}/", abs_src_str))
            || path == src.as_str()
        {
            return true;
        }
    }
    false
}

fn command_references_source(command: &str, source_dirs: &[String]) -> bool {
    for src in source_dirs {
        // Best-effort: flag commands that mention source dirs by path.
        // This won't catch indirect access (cd src && ...) — see module docs.
        if command.contains(&format!(" {}/", src)) || command.contains(&format!(" ./{}/", src)) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config(source: Vec<&str>) -> Config {
        use crate::config::*;
        Config {
            project: ProjectConfig {
                name: "test".to_string(),
            },
            models: ModelsConfig::default(),
            limits: LimitsConfig::default(),
            review: ReviewConfig::default(),
            git: GitConfig::default(),
            terminal: TerminalConfig::default(),
            paths: PathsConfig {
                source: source.into_iter().map(String::from).collect(),
                ..PathsConfig::default()
            },
            commands: CommandsConfig::default(),
        }
    }

    #[test]
    fn test_ddv_isolation_catches_read() {
        let config = test_config(vec!["src"]);
        let root = Path::new("/project");
        let log = vec![ToolCall::Read {
            path: "src/main.rs".to_string(),
        }];
        assert!(verify_ddv_isolation(&log, &config, root).is_err());
    }

    #[test]
    fn test_ddv_isolation_catches_write() {
        let config = test_config(vec!["src"]);
        let root = Path::new("/project");
        let log = vec![ToolCall::Write {
            path: "src/lib.rs".to_string(),
        }];
        assert!(verify_ddv_isolation(&log, &config, root).is_err());
    }

    #[test]
    fn test_ddv_isolation_catches_edit() {
        let config = test_config(vec!["src"]);
        let root = Path::new("/project");
        let log = vec![ToolCall::Edit {
            path: "src/lib.rs".to_string(),
        }];
        assert!(verify_ddv_isolation(&log, &config, root).is_err());
    }

    #[test]
    fn test_ddv_isolation_allows_test_files() {
        let config = test_config(vec!["src"]);
        let root = Path::new("/project");
        let log = vec![ToolCall::Read {
            path: "tests/ddv/test_foo.py".to_string(),
        }];
        assert!(verify_ddv_isolation(&log, &config, root).is_ok());
    }

    #[test]
    fn test_ddv_isolation_catches_bash() {
        let config = test_config(vec!["src"]);
        let root = Path::new("/project");
        let log = vec![ToolCall::Bash {
            command: "cat src/main.rs".to_string(),
        }];
        assert!(verify_ddv_isolation(&log, &config, root).is_err());
    }

    #[test]
    fn test_ddv_isolation_allows_unrelated_bash() {
        let config = test_config(vec!["src"]);
        let root = Path::new("/project");
        let log = vec![ToolCall::Bash {
            command: "python -m pytest tests/".to_string(),
        }];
        assert!(verify_ddv_isolation(&log, &config, root).is_ok());
    }
}
