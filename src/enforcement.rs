use anyhow::Result;
use std::path::Path;

use crate::agent::ToolCall;
use crate::config::Config;
use crate::git;
use crate::terminal;

/// Check that the DDV Red agent didn't access source files (isolation violation)
pub fn verify_ddv_isolation(tool_log: &[ToolCall], config: &Config, project_root: &Path) -> Result<()> {
    let source_dirs = &config.paths.source;

    let violations: Vec<&ToolCall> = tool_log
        .iter()
        .filter(|call| match call {
            ToolCall::Read { path } => is_under_source(path, source_dirs, project_root),
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
             The DDV agent must not read implementation source code.",
            violations.len()
        );
    }
    Ok(())
}

/// Check that build agent didn't modify DDV tests, revert if so
pub fn verify_ddv_tests_unmodified(config: &Config) -> Result<()> {
    let tests_ddv = &config.paths.tests_ddv;

    if git::has_any_modifications(tests_ddv)? {
        terminal::log_warn("Build agent modified DDV tests — reverting!");
        git::reset_path(tests_ddv)?;
        git::checkout_path(tests_ddv)?;
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
        // Look for cat/head/tail/less/more of source files
        if command.contains(&format!(" {}/", src))
            || command.contains(&format!(" ./{}/", src))
        {
            return true;
        }
    }
    false
}
