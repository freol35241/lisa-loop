use anyhow::{Context, Result};
use crossterm::style::Color;
use serde_json::Value;
use std::io::{BufRead, BufReader, IsTerminal, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::terminal;

#[derive(Debug, Default)]
pub struct AgentStats {
    pub tool_count: u32,
    pub file_writes: u32,
    pub test_runs: u32,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct AgentResult {
    pub result_text: String,
    pub stats: AgentStats,
    pub elapsed_secs: u64,
    pub tool_log: Vec<ToolCall>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ToolCall {
    Read { path: String },
    Write { path: String },
    Edit { path: String },
    Bash { command: String },
    Glob { pattern: String },
    Grep { pattern: String },
    Task { description: String },
    Other { name: String },
}

/// Shared state between NDJSON loop and ticker thread for collapsed-mode display.
#[derive(Debug, Default, Clone)]
struct LiveStatus {
    tool_count: u32,
    latest_tool: String,
}

pub fn run_agent(
    input: &str,
    model: &str,
    label: &str,
    collapse_output: bool,
    error_log_path: Option<&Path>,
) -> Result<AgentResult> {
    let start = Instant::now();
    let mut stats = AgentStats::default();
    let mut tool_log = Vec::new();
    let mut result_text = String::new();

    terminal::log_info(&format!("Calling agent: {} (model: {})", label, model));

    let is_tty = std::io::stdout().is_terminal();
    let collapsed = collapse_output && is_tty;
    if collapsed {
        let line = format_collapsed_line(label, 0, 0, 0, "");
        print!("  ");
        terminal::print_colored(&line, Color::Cyan);
        println!();
    }

    // Shared live status for collapsed mode
    let live_status = Arc::new(Mutex::new(LiveStatus::default()));

    // Elapsed time ticker thread (updates the line in collapse mode)
    let ticker_running = Arc::new(AtomicBool::new(collapsed));
    let ticker_handle = {
        let running = ticker_running.clone();
        let label = label.to_string();
        let tick_start = start;
        let live = live_status.clone();
        std::thread::spawn(move || {
            while running.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_secs(30));
                if !running.load(Ordering::Relaxed) {
                    break;
                }
                let elapsed = tick_start.elapsed().as_secs();
                let mins = elapsed / 60;
                let secs = elapsed % 60;
                let status = live.lock().unwrap().clone();
                let line = format_collapsed_line(
                    &label,
                    mins,
                    secs,
                    status.tool_count,
                    &status.latest_tool,
                );
                print!("\x1b[1A\x1b[2K  ");
                terminal::print_colored(&line, Color::Cyan);
                println!();
            }
        })
    };

    let mut child = Command::new("claude")
        .args([
            "-p",
            "--dangerously-skip-permissions",
            "--verbose",
            "--model",
            model,
            "--output-format",
            "stream-json",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to spawn claude CLI. Is it installed and on PATH?")?;

    // Write input to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes())?;
        // stdin is dropped here, closing it
    }

    // Read NDJSON stream
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => continue,
            };

            let parsed: Value = match serde_json::from_str(&line) {
                Ok(v) => v,
                Err(_) => continue,
            };

            match parsed.get("type").and_then(|t| t.as_str()) {
                Some("assistant") => {
                    if let Some(contents) = parsed
                        .get("message")
                        .and_then(|m| m.get("content"))
                        .and_then(|c| c.as_array())
                    {
                        for item in contents {
                            match item.get("type").and_then(|t| t.as_str()) {
                                Some("thinking") => {
                                    if !collapse_output {
                                        if let Some(thought) =
                                            item.get("thinking").and_then(|t| t.as_str())
                                        {
                                            let truncated = if thought.len() > 200 {
                                                format!("{}...", &thought[..200])
                                            } else {
                                                thought.to_string()
                                            };
                                            terminal::print_dim(&format!(
                                                "    [💭 {}] {}\n",
                                                terminal::ts(),
                                                truncated
                                            ));
                                        }
                                    }
                                }
                                Some("tool_use") => {
                                    stats.tool_count += 1;
                                    let name =
                                        item.get("name").and_then(|n| n.as_str()).unwrap_or("");
                                    let input_val =
                                        item.get("input").cloned().unwrap_or(Value::Null);

                                    let detail = format_tool_detail(name, &input_val);
                                    let call = parse_tool_call(name, &input_val);
                                    tool_log.push(call);

                                    // Count specific tool types
                                    if name == "Write" || name == "Edit" {
                                        stats.file_writes += 1;
                                    }
                                    if name == "Bash" {
                                        if let Some(cmd) =
                                            input_val.get("command").and_then(|c| c.as_str())
                                        {
                                            if cmd.contains("test") || cmd.contains("pytest") {
                                                stats.test_runs += 1;
                                            }
                                        }
                                    }

                                    if collapsed {
                                        // Update shared status and refresh the collapsed line
                                        {
                                            let mut status = live_status.lock().unwrap();
                                            status.tool_count = stats.tool_count;
                                            status.latest_tool = detail.clone();
                                        }
                                        let elapsed = start.elapsed().as_secs();
                                        let mins = elapsed / 60;
                                        let secs = elapsed % 60;
                                        let line = format_collapsed_line(
                                            label,
                                            mins,
                                            secs,
                                            stats.tool_count,
                                            &detail,
                                        );
                                        print!("\x1b[1A\x1b[2K  ");
                                        terminal::print_colored(&line, Color::Cyan);
                                        println!();
                                    } else {
                                        print!("    ");
                                        terminal::print_colored(
                                            &format!("[🔧 {}]", terminal::ts()),
                                            Color::Magenta,
                                        );
                                        println!(" {}", detail);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Some("result") => {
                    if let Some(text) = parsed.get("result").and_then(|r| r.as_str()) {
                        result_text = text.to_string();
                    }
                }
                _ => {}
            }
        }
    }

    let status = child.wait().context("Failed to wait for claude process")?;

    // Stop ticker thread
    ticker_running.store(false, Ordering::Relaxed);
    let _ = ticker_handle.join();

    if !status.success() {
        let code = status.code().unwrap_or(-1);
        let elapsed = start.elapsed().as_secs();

        // Show failure context in collapsed mode
        if collapsed {
            print!("\x1b[1A\x1b[2K  ");
            terminal::print_colored(
                &format!(
                    "x {} ({}s, {} tools — FAILED exit {})",
                    label, elapsed, stats.tool_count, code
                ),
                Color::Red,
            );
            println!();

            // Print last 5 tool calls
            let recent: Vec<&ToolCall> = tool_log.iter().rev().take(5).collect();
            if !recent.is_empty() {
                println!("    Last tool calls:");
                for call in recent.iter().rev() {
                    println!("      {}", format_tool_call_summary(call));
                }
            }
        }

        // Persist error context to file
        if let Some(path) = error_log_path {
            let mut content = "# Last Error\n\n".to_string();
            content.push_str(&format!("- **Agent:** {}\n", label));
            content.push_str(&format!("- **Exit code:** {}\n", code));
            content.push_str(&format!("- **Elapsed:** {}s\n", elapsed));
            content.push_str(&format!("- **Tool count:** {}\n", stats.tool_count));
            content.push_str("\n## Last 10 Tool Calls\n\n");
            let last_n: Vec<&ToolCall> = tool_log.iter().rev().take(10).collect();
            for (i, call) in last_n.iter().rev().enumerate() {
                content.push_str(&format!("{}. {}\n", i + 1, format_tool_call_summary(call)));
            }
            if !result_text.is_empty() {
                content.push_str("\n## Partial Result\n\n");
                content.push_str(&result_text);
                content.push('\n');
            }
            let _ = std::fs::write(path, &content);
        }

        anyhow::bail!(
            "Agent '{}' exited with code {}. Check the output above for errors. Run `lisa resume` to retry this phase.",
            label, code
        );
    }
    let elapsed = start.elapsed().as_secs();

    // Print summary
    let mut summary = format!("{} tools", stats.tool_count);
    if stats.file_writes > 0 {
        summary.push_str(&format!(", {} files written", stats.file_writes));
    }
    if stats.test_runs > 0 {
        summary.push_str(&format!(", {} test runs", stats.test_runs));
    }

    if collapsed {
        // Move up and overwrite the "▸ label ..." line
        print!("\x1b[1A\x1b[2K");
        print!("  ");
        terminal::print_colored("✓", Color::Green);
        println!(" {} ({}s, {})", label, elapsed, summary);
    } else {
        terminal::log_info(&format!("Agent finished ({}s, {})", elapsed, summary));
    }

    // Print result text
    if !result_text.is_empty() {
        println!();
        terminal::print_colored("    ── Result ──\n", Color::Magenta);
        for line in result_text.lines() {
            println!("    {}", line);
        }
        terminal::print_colored("    ── End ──\n", Color::Magenta);
        println!();
    }

    Ok(AgentResult {
        result_text,
        stats,
        elapsed_secs: elapsed,
        tool_log,
    })
}

/// Format the single-line collapsed status display.
pub fn format_collapsed_line(
    label: &str,
    mins: u64,
    secs: u64,
    tool_count: u32,
    latest_tool: &str,
) -> String {
    let term_width = crossterm::terminal::size()
        .map(|(w, _)| w as usize)
        .unwrap_or(80);

    let time_part = if mins > 0 || secs > 0 {
        format!(" {}m{:02}s", mins, secs)
    } else {
        String::new()
    };

    let tools_part = if tool_count > 0 {
        format!(" | {} tools", tool_count)
    } else {
        String::new()
    };

    // Build the base without the tool detail
    let base = format!("▸ {} ...{}{}", label, time_part, tools_part);

    if latest_tool.is_empty() || tool_count == 0 {
        return base;
    }

    // Budget for tool detail: term_width - base_len - " | " - 2 (indent)
    let prefix_len = base.len() + 3 + 2; // " | " separator + "  " indent
    if prefix_len + 4 >= term_width {
        return base;
    }
    let budget = term_width - prefix_len;
    let truncated = truncate_tool_detail(latest_tool, budget);
    format!("{} | {}", base, truncated)
}

/// Truncate a tool detail string to fit within max_len characters.
pub fn truncate_tool_detail(detail: &str, max_len: usize) -> String {
    if max_len < 4 {
        return String::new();
    }
    if detail.len() <= max_len {
        detail.to_string()
    } else {
        format!("{}...", &detail[..max_len - 3])
    }
}

/// Format a ToolCall for display in error context.
pub fn format_tool_call_summary(call: &ToolCall) -> String {
    match call {
        ToolCall::Read { path } => format!("Read {}", path),
        ToolCall::Write { path } => format!("Write {}", path),
        ToolCall::Edit { path } => format!("Edit {}", path),
        ToolCall::Bash { command } => {
            let first_line = command.lines().next().unwrap_or("");
            let truncated = if first_line.len() > 60 {
                format!("{}...", &first_line[..57])
            } else {
                first_line.to_string()
            };
            format!("Bash $ {}", truncated)
        }
        ToolCall::Glob { pattern } => format!("Glob {}", pattern),
        ToolCall::Grep { pattern } => format!("Grep {}", pattern),
        ToolCall::Task { description } => {
            let truncated = if description.len() > 50 {
                format!("{}...", &description[..47])
            } else {
                description.to_string()
            };
            format!("Task {}", truncated)
        }
        ToolCall::Other { name } => name.to_string(),
    }
}

fn format_tool_detail(name: &str, input: &Value) -> String {
    match name {
        "Read" => {
            let path = input
                .get("file_path")
                .and_then(|p| p.as_str())
                .unwrap_or("");
            format!("Read {}", path)
        }
        "Edit" => {
            let path = input
                .get("file_path")
                .and_then(|p| p.as_str())
                .unwrap_or("");
            format!("Edit {}", path)
        }
        "Write" => {
            let path = input
                .get("file_path")
                .and_then(|p| p.as_str())
                .unwrap_or("");
            format!("Write {}", path)
        }
        "Bash" => {
            let cmd = input.get("command").and_then(|c| c.as_str()).unwrap_or("");
            let first_line = cmd.lines().next().unwrap_or("");
            let truncated = if first_line.len() > 80 {
                &first_line[..80]
            } else {
                first_line
            };
            format!("Bash $ {}", truncated)
        }
        "Glob" => {
            let pattern = input.get("pattern").and_then(|p| p.as_str()).unwrap_or("");
            format!("Glob {}", pattern)
        }
        "Grep" => {
            let pattern = input.get("pattern").and_then(|p| p.as_str()).unwrap_or("");
            format!("Grep {}", pattern)
        }
        "Task" => {
            let desc = input
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("");
            format!("Task {}", desc)
        }
        _ => name.to_string(),
    }
}

fn parse_tool_call(name: &str, input: &Value) -> ToolCall {
    match name {
        "Read" => ToolCall::Read {
            path: input
                .get("file_path")
                .and_then(|p| p.as_str())
                .unwrap_or("")
                .to_string(),
        },
        "Write" => ToolCall::Write {
            path: input
                .get("file_path")
                .and_then(|p| p.as_str())
                .unwrap_or("")
                .to_string(),
        },
        "Edit" => ToolCall::Edit {
            path: input
                .get("file_path")
                .and_then(|p| p.as_str())
                .unwrap_or("")
                .to_string(),
        },
        "Bash" => ToolCall::Bash {
            command: input
                .get("command")
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .to_string(),
        },
        "Glob" => ToolCall::Glob {
            pattern: input
                .get("pattern")
                .and_then(|p| p.as_str())
                .unwrap_or("")
                .to_string(),
        },
        "Grep" => ToolCall::Grep {
            pattern: input
                .get("pattern")
                .and_then(|p| p.as_str())
                .unwrap_or("")
                .to_string(),
        },
        "Task" => ToolCall::Task {
            description: input
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("")
                .to_string(),
        },
        _ => ToolCall::Other {
            name: name.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_collapsed_line_no_tools() {
        let line = format_collapsed_line("Build: iter 1", 0, 0, 0, "");
        assert_eq!(line, "▸ Build: iter 1 ...");
    }

    #[test]
    fn test_format_collapsed_line_with_time_and_tools() {
        let line = format_collapsed_line("Build: iter 3", 2, 15, 7, "Read plan.md");
        assert!(line.contains("▸ Build: iter 3 ..."));
        assert!(line.contains("2m15s"));
        assert!(line.contains("7 tools"));
        assert!(line.contains("Read plan.md"));
    }

    #[test]
    fn test_format_collapsed_line_truncates_long_tool() {
        let long_tool = "Read /very/long/path/to/some/deeply/nested/directory/structure/that/goes/on/and/on/and/on/file.txt";
        let line = format_collapsed_line("Build: iter 1", 1, 30, 3, long_tool);
        // Should not exceed a reasonable width — exact length depends on terminal::size() mock
        // but the line should contain "..." if truncated
        assert!(line.contains("▸ Build: iter 1 ..."));
        assert!(line.contains("3 tools"));
    }

    #[test]
    fn test_truncate_tool_detail_short() {
        assert_eq!(truncate_tool_detail("Read foo.rs", 20), "Read foo.rs");
    }

    #[test]
    fn test_truncate_tool_detail_exact() {
        assert_eq!(truncate_tool_detail("abcde", 5), "abcde");
    }

    #[test]
    fn test_truncate_tool_detail_long() {
        let result = truncate_tool_detail("Read /very/long/path/to/file.txt", 15);
        assert_eq!(result, "Read /very/l...");
        assert_eq!(result.len(), 15);
    }

    #[test]
    fn test_truncate_tool_detail_tiny_budget() {
        assert_eq!(truncate_tool_detail("anything", 3), "");
        assert_eq!(truncate_tool_detail("anything", 0), "");
    }

    #[test]
    fn test_format_tool_call_summary_read() {
        let call = ToolCall::Read {
            path: "/src/main.rs".to_string(),
        };
        assert_eq!(format_tool_call_summary(&call), "Read /src/main.rs");
    }

    #[test]
    fn test_format_tool_call_summary_bash_truncates() {
        let call = ToolCall::Bash {
            command:
                "cargo test --all-features --workspace -- --nocapture some_very_long_test_name_here"
                    .to_string(),
        };
        let summary = format_tool_call_summary(&call);
        assert!(summary.starts_with("Bash $ "));
        assert!(summary.len() <= 67); // "Bash $ " (7) + 60 max
    }

    #[test]
    fn test_format_tool_call_summary_task_truncates() {
        let call = ToolCall::Task {
            description: "A very long task description that should be truncated to fit within a reasonable display width".to_string(),
        };
        let summary = format_tool_call_summary(&call);
        assert!(summary.starts_with("Task "));
        assert!(summary.contains("..."));
    }

    #[test]
    fn test_format_tool_call_summary_other() {
        let call = ToolCall::Other {
            name: "WebSearch".to_string(),
        };
        assert_eq!(format_tool_call_summary(&call), "WebSearch");
    }
}
