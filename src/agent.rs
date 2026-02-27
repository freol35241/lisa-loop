use anyhow::{Context, Result};
use base64::Engine;
use crossterm::style::Color;
use serde_json::Value;
use std::io::{BufRead, BufReader, IsTerminal, Write};
use std::process::{Command, Stdio};
use std::time::Instant;

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

pub fn run_agent(
    input: &str,
    model: &str,
    label: &str,
    collapse_output: bool,
) -> Result<AgentResult> {
    let start = Instant::now();
    let mut stats = AgentStats::default();
    let mut tool_log = Vec::new();
    let mut result_text = String::new();

    terminal::log_info(&format!(
        "Calling agent: {} (model: {})",
        label, model
    ));

    if collapse_output && std::io::stdout().is_terminal() {
        print!("  ");
        terminal::print_colored(&format!("▸ {} ...", label), Color::Cyan);
        println!();
    }

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
                                    let name = item
                                        .get("name")
                                        .and_then(|n| n.as_str())
                                        .unwrap_or("");
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

                                    if !collapse_output {
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

    let _ = child.wait();
    let elapsed = start.elapsed().as_secs();

    // If collapse mode, try to decode base64 result_text if needed
    if result_text.is_empty() {
        // Try to extract from the tool log as fallback
    }

    // Print summary
    let mut summary = format!("{} tools", stats.tool_count);
    if stats.file_writes > 0 {
        summary.push_str(&format!(", {} files written", stats.file_writes));
    }
    if stats.test_runs > 0 {
        summary.push_str(&format!(", {} test runs", stats.test_runs));
    }

    if collapse_output && std::io::stdout().is_terminal() {
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
            let cmd = input
                .get("command")
                .and_then(|c| c.as_str())
                .unwrap_or("");
            let first_line = cmd.lines().next().unwrap_or("");
            let truncated = if first_line.len() > 80 {
                &first_line[..80]
            } else {
                first_line
            };
            format!("Bash $ {}", truncated)
        }
        "Glob" => {
            let pattern = input
                .get("pattern")
                .and_then(|p| p.as_str())
                .unwrap_or("");
            format!("Glob {}", pattern)
        }
        "Grep" => {
            let pattern = input
                .get("pattern")
                .and_then(|p| p.as_str())
                .unwrap_or("");
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

/// Decode base64 result text (used in NDJSON parsing)
#[allow(dead_code)]
fn decode_b64_result(encoded: &str) -> String {
    base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .unwrap_or_default()
}
