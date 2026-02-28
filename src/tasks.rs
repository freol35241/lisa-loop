use anyhow::Result;
use regex::Regex;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
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

/// Hash only the (index, status) pairs from plan.md tasks.
/// Ignores descriptions, checklists, and prose — only status transitions change the hash.
pub fn hash_task_statuses(plan_path: &Path) -> Result<u64> {
    let content = if plan_path.exists() {
        std::fs::read_to_string(plan_path)?
    } else {
        String::new()
    };
    let tasks = parse_tasks(&content);
    let pairs: Vec<(usize, &str)> = tasks
        .iter()
        .enumerate()
        .map(|(i, t)| (i, t.status.as_str()))
        .collect();
    let mut hasher = DefaultHasher::new();
    pairs.hash(&mut hasher);
    Ok(hasher.finish())
}

#[derive(Debug)]
struct Task {
    pass: u32,
    status: String,
}

fn parse_tasks(content: &str) -> Vec<Task> {
    let task_re = Regex::new(r"(?i)^#{2,4}\s+Task\s+\d").unwrap();
    let status_re = Regex::new(r"\*\*Status:\*\*\s+(\w+)").unwrap();
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
                    pass: current_pass.unwrap_or(1),
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
            pass: current_pass.unwrap_or(1),
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

    #[test]
    fn test_parse_tasks_heading_variations() {
        let content = r#"# Plan

## Task 1: Two hashes
- **Status:** TODO
- **Pass:** 1

#### Task 2: Four hashes
- **Status:**  DONE
- **Pass:** 2

### task 3: lowercase
- **Status:** IN_PROGRESS
"#;
        let tasks = parse_tasks(content);
        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0].status, "TODO");
        assert_eq!(tasks[0].pass, 1);
        assert_eq!(tasks[1].status, "DONE");
        assert_eq!(tasks[1].pass, 2);
        assert_eq!(tasks[2].status, "IN_PROGRESS");
        assert_eq!(tasks[2].pass, 1); // default when missing
    }

    #[test]
    fn test_hash_task_statuses_changes_on_status_change() {
        let dir = std::env::temp_dir().join("lisa_test_hash_status");
        std::fs::create_dir_all(&dir).unwrap();
        let plan = dir.join("plan.md");

        let content_v1 = "### Task 1: Foo\n- **Status:** TODO\n- **Pass:** 1\n\n### Task 2: Bar\n- **Status:** TODO\n- **Pass:** 1\n";
        std::fs::write(&plan, content_v1).unwrap();
        let hash1 = hash_task_statuses(&plan).unwrap();

        // Change a status
        let content_v2 = "### Task 1: Foo\n- **Status:** DONE\n- **Pass:** 1\n\n### Task 2: Bar\n- **Status:** TODO\n- **Pass:** 1\n";
        std::fs::write(&plan, content_v2).unwrap();
        let hash2 = hash_task_statuses(&plan).unwrap();

        assert_ne!(hash1, hash2, "Hash should change when task status changes");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_hash_task_statuses_stable_on_description_change() {
        let dir = std::env::temp_dir().join("lisa_test_hash_stable");
        std::fs::create_dir_all(&dir).unwrap();
        let plan = dir.join("plan.md");

        let content_v1 = "### Task 1: Original description\n- **Status:** TODO\n- **Pass:** 1\n- **Checklist:**\n  - [ ] Do something\n";
        std::fs::write(&plan, content_v1).unwrap();
        let hash1 = hash_task_statuses(&plan).unwrap();

        // Change description and checklist but keep same status
        let content_v2 = "### Task 1: Completely rewritten description with more words\n- **Status:** TODO\n- **Pass:** 1\n- **Checklist:**\n  - [ ] Do something different\n  - [ ] Extra item\n\nSome added prose here.\n";
        std::fs::write(&plan, content_v2).unwrap();
        let hash2 = hash_task_statuses(&plan).unwrap();

        assert_eq!(
            hash1, hash2,
            "Hash should NOT change when only descriptions change"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_hash_task_statuses_missing_file() {
        let path = Path::new("/tmp/lisa_nonexistent_plan.md");
        let hash = hash_task_statuses(path).unwrap();
        // Should not panic; returns hash of empty task list
        let hash2 = hash_task_statuses(path).unwrap();
        assert_eq!(hash, hash2, "Hash of missing file should be deterministic");
    }
}
