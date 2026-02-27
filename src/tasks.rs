use anyhow::Result;
use regex::Regex;
use std::path::Path;

/// Parse methodology/plan.md and count tasks by status for a given max pass
pub fn count_uncompleted_tasks(plan_path: &Path, max_pass: u32) -> Result<u32> {
    if !plan_path.exists() {
        return Ok(0);
    }
    let content = std::fs::read_to_string(plan_path)?;
    let tasks = parse_tasks(&content);
    Ok(tasks
        .iter()
        .filter(|t| t.pass <= max_pass && (t.status == "TODO" || t.status == "IN_PROGRESS"))
        .count() as u32)
}

pub fn count_blocked_tasks(plan_path: &Path, max_pass: u32) -> Result<u32> {
    if !plan_path.exists() {
        return Ok(0);
    }
    let content = std::fs::read_to_string(plan_path)?;
    let tasks = parse_tasks(&content);
    Ok(tasks
        .iter()
        .filter(|t| t.pass <= max_pass && t.status == "BLOCKED")
        .count() as u32)
}

pub fn all_tasks_done(plan_path: &Path, max_pass: u32) -> Result<bool> {
    Ok(count_uncompleted_tasks(plan_path, max_pass)? == 0)
}

pub fn has_blocked_tasks(plan_path: &Path, max_pass: u32) -> Result<bool> {
    Ok(count_blocked_tasks(plan_path, max_pass)? > 0)
}

pub fn count_tasks_by_status(plan_path: &Path) -> Result<TaskCounts> {
    if !plan_path.exists() {
        return Ok(TaskCounts::default());
    }
    let content = std::fs::read_to_string(plan_path)?;
    let tasks = parse_tasks(&content);
    Ok(TaskCounts {
        total: tasks.len() as u32,
        todo: tasks.iter().filter(|t| t.status == "TODO").count() as u32,
        in_progress: tasks.iter().filter(|t| t.status == "IN_PROGRESS").count() as u32,
        done: tasks.iter().filter(|t| t.status == "DONE").count() as u32,
        blocked: tasks.iter().filter(|t| t.status == "BLOCKED").count() as u32,
    })
}

#[derive(Debug, Default)]
pub struct TaskCounts {
    pub total: u32,
    pub todo: u32,
    pub in_progress: u32,
    pub done: u32,
    pub blocked: u32,
}

#[derive(Debug)]
struct Task {
    pass: u32,
    status: String,
}

fn parse_tasks(content: &str) -> Vec<Task> {
    let task_re = Regex::new(r"^### Task").unwrap();
    let status_re = Regex::new(r"\*\*Status:\*\*\s*(\w+)").unwrap();
    let pass_re = Regex::new(r"\*\*Pass:\*\*\s*(\d+)").unwrap();

    let mut tasks = Vec::new();
    let mut current_status = None;
    let mut current_pass = None;
    let mut in_task = false;

    for line in content.lines() {
        if task_re.is_match(line) {
            // Save previous task if any
            if in_task {
                tasks.push(Task {
                    pass: current_pass.unwrap_or(9999),
                    status: current_status.unwrap_or_default(),
                });
            }
            in_task = true;
            current_status = None;
            current_pass = None;
        } else if in_task {
            if let Some(caps) = status_re.captures(line) {
                current_status = Some(caps[1].to_string());
            }
            if let Some(caps) = pass_re.captures(line) {
                if let Ok(p) = caps[1].parse::<u32>() {
                    current_pass = Some(p);
                }
            }
        }
    }

    // Don't forget the last task
    if in_task {
        tasks.push(Task {
            pass: current_pass.unwrap_or(9999),
            status: current_status.unwrap_or_default(),
        });
    }

    tasks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tasks() {
        let content = r#"# Implementation Plan

## Tasks

### Task 1: Setup infrastructure
- **Status:** DONE
- **Pass:** 1
- **Methodology:** N/A
- **Checklist:**
  - [x] Create project structure

### Task 2: Implement core
- **Status:** TODO
- **Pass:** 1
- **Methodology:** Section 2.1
- **Checklist:**
  - [ ] Implement equations

### Task 3: Advanced features
- **Status:** BLOCKED
- **Pass:** 2
- **Methodology:** Section 3.1
- **Dependencies:** Task 2
"#;
        let tasks = parse_tasks(content);
        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0].status, "DONE");
        assert_eq!(tasks[0].pass, 1);
        assert_eq!(tasks[1].status, "TODO");
        assert_eq!(tasks[1].pass, 1);
        assert_eq!(tasks[2].status, "BLOCKED");
        assert_eq!(tasks[2].pass, 2);
    }

    #[test]
    fn test_parse_tasks_empty() {
        let content = "# Implementation Plan\n\n## Tasks\n";
        let tasks = parse_tasks(content);
        assert_eq!(tasks.len(), 0);
    }
}
