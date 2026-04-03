use anyhow::Result;
use regex::Regex;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::path::Path;

use crate::terminal;

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

pub fn count_tasks_by_status_for_pass(plan_path: &Path, pass: u32) -> Result<TaskCounts> {
    if !plan_path.exists() {
        return Ok(TaskCounts::default());
    }
    let content = std::fs::read_to_string(plan_path)?;
    let tasks = parse_tasks(&content);
    let filtered: Vec<&Task> = tasks.iter().filter(|t| t.pass == pass).collect();
    Ok(TaskCounts {
        total: filtered.len() as u32,
        todo: filtered.iter().filter(|t| t.status == "TODO").count() as u32,
        in_progress: filtered
            .iter()
            .filter(|t| t.status == "IN_PROGRESS")
            .count() as u32,
        done: filtered.iter().filter(|t| t.status == "DONE").count() as u32,
        blocked: filtered.iter().filter(|t| t.status == "BLOCKED").count() as u32,
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

/// Metadata for a task selected by the orchestrator.
#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub number: u32,
    pub name: String,
    pub methodology_ref: String,
    pub needs_bounds: bool,
}

/// Find the next eligible task: status == TODO or IN_PROGRESS, pass <= current_pass, all deps satisfied.
/// IN_PROGRESS tasks are included to allow retrying interrupted work after a crash/resume.
pub fn find_next_task(plan_path: &Path, current_pass: u32) -> Result<Option<TaskInfo>> {
    if !plan_path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(plan_path)?;
    let tasks = parse_tasks(&content);

    let done_ids: HashSet<u32> = tasks
        .iter()
        .filter(|t| t.status == "DONE")
        .map(|t| t.number)
        .collect();

    for task in &tasks {
        if (task.status != "TODO" && task.status != "IN_PROGRESS") || task.pass > current_pass {
            continue;
        }
        let deps_met = task.dependencies.iter().all(|d| done_ids.contains(d));
        if deps_met {
            if task.status == "IN_PROGRESS" {
                terminal::log_warn(&format!(
                    "Retrying interrupted task {} (was IN_PROGRESS from a previous run)",
                    task.number
                ));
            }
            let needs_bounds = !task.bounding_checks.eq_ignore_ascii_case("none")
                && !task.bounding_checks.is_empty();
            return Ok(Some(TaskInfo {
                number: task.number,
                name: task.name.clone(),
                methodology_ref: task.methodology.clone(),
                needs_bounds,
            }));
        }
    }

    Ok(None)
}

/// Mark a task as IN_PROGRESS in plan.md by task number.
pub fn mark_task_in_progress(plan_path: &Path, task_number: u32) -> Result<()> {
    let content = std::fs::read_to_string(plan_path)?;
    let task_heading_re =
        Regex::new(&format!(r"(?im)^(#{{2,4}}\s+Task\s+{}\b)", task_number)).unwrap();
    let status_re = Regex::new(r"(?m)^(- \*\*Status:\*\*\s+)TODO\s*$").unwrap();

    // Find the line range for this task
    let any_task_re = Regex::new(r"(?i)^#{2,4}\s+Task\s+\d").unwrap();
    let mut in_target = false;
    let mut result_lines: Vec<String> = Vec::new();
    let mut replaced = false;

    for line in content.lines() {
        if task_heading_re.is_match(line) {
            in_target = true;
            result_lines.push(line.to_string());
        } else if in_target && any_task_re.is_match(line) {
            // Hit next task heading — stop targeting
            in_target = false;
            result_lines.push(line.to_string());
        } else if in_target && !replaced {
            if let Some(caps) = status_re.captures(line) {
                let prefix = caps.get(1).unwrap().as_str();
                result_lines.push(format!("{}IN_PROGRESS", prefix));
                replaced = true;
                continue;
            }
            result_lines.push(line.to_string());
        } else {
            result_lines.push(line.to_string());
        }
    }

    if replaced {
        let mut output = result_lines.join("\n");
        // Preserve trailing newline if original had one
        if content.ends_with('\n') && !output.ends_with('\n') {
            output.push('\n');
        }
        std::fs::write(plan_path, output)?;
    }

    Ok(())
}

#[derive(Debug)]
struct Task {
    number: u32,
    name: String,
    pass: u32,
    status: String,
    methodology: String,
    bounding_checks: String,
    dependencies: Vec<u32>,
}

fn parse_tasks(content: &str) -> Vec<Task> {
    let task_re = Regex::new(r"(?i)^#{2,4}\s+Task\s+(\d+)(?::\s*(.*))?").unwrap();
    let status_re = Regex::new(r"\*\*Status:\*\*\s+(\w+)").unwrap();
    let pass_re = Regex::new(r"\*\*Pass:\*\*\s*(\d+)").unwrap();
    let methodology_re = Regex::new(r"\*\*Methodology:\*\*\s*(.+)").unwrap();
    let bounding_re = Regex::new(r"\*\*Bounding Checks:\*\*\s*(.+)").unwrap();
    let deps_re = Regex::new(r"\*\*Dependencies:\*\*\s*(.+)").unwrap();
    let dep_num_re = Regex::new(r"(?i)Task\s+(\d+)").unwrap();

    let mut tasks = Vec::new();
    let mut current_number: u32 = 0;
    let mut current_name = String::new();
    let mut current_status = None;
    let mut current_pass = None;
    let mut current_methodology = String::new();
    let mut current_bounding = String::new();
    let mut current_deps: Vec<u32> = Vec::new();
    let mut in_task = false;

    for line in content.lines() {
        if let Some(caps) = task_re.captures(line) {
            // Save previous task if any
            if in_task {
                tasks.push(Task {
                    number: current_number,
                    name: current_name.clone(),
                    pass: current_pass.unwrap_or(1),
                    status: current_status.unwrap_or_else(|| "TODO".to_string()),
                    methodology: current_methodology.clone(),
                    bounding_checks: current_bounding.clone(),
                    dependencies: current_deps.clone(),
                });
            }
            in_task = true;
            current_number = caps[1].parse().unwrap_or(0);
            current_name = caps
                .get(2)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();
            current_status = None;
            current_pass = None;
            current_methodology = String::new();
            current_bounding = String::new();
            current_deps = Vec::new();
        } else if in_task {
            if let Some(caps) = status_re.captures(line) {
                current_status = Some(caps[1].to_string());
            }
            if let Some(caps) = pass_re.captures(line) {
                if let Ok(p) = caps[1].parse::<u32>() {
                    current_pass = Some(p);
                }
            }
            if let Some(caps) = methodology_re.captures(line) {
                current_methodology = caps[1].trim().to_string();
            }
            if let Some(caps) = bounding_re.captures(line) {
                current_bounding = caps[1].trim().to_string();
            }
            if let Some(caps) = deps_re.captures(line) {
                let deps_str = &caps[1];
                // "None" or "none" means no dependencies
                if !deps_str.trim().eq_ignore_ascii_case("none") {
                    for dep_cap in dep_num_re.captures_iter(deps_str) {
                        if let Ok(n) = dep_cap[1].parse::<u32>() {
                            current_deps.push(n);
                        }
                    }
                }
            }
        }
    }

    // Don't forget the last task
    if in_task {
        tasks.push(Task {
            number: current_number,
            name: current_name,
            pass: current_pass.unwrap_or(1),
            status: current_status.unwrap_or_else(|| "TODO".to_string()),
            methodology: current_methodology,
            bounding_checks: current_bounding,
            dependencies: current_deps,
        });
    }

    // Warn about duplicate task numbers
    let mut seen_numbers = HashSet::new();
    for task in &tasks {
        if !seen_numbers.insert(task.number) {
            terminal::log_warn(&format!(
                "Duplicate task number {} found in plan.md — behavior may be unpredictable",
                task.number
            ));
        }
    }

    tasks
}

/// Detect circular dependencies among tasks. Returns a list of cycle descriptions.
pub fn detect_dependency_cycles(plan_path: &Path) -> Result<Vec<String>> {
    if !plan_path.exists() {
        return Ok(vec![]);
    }
    let content = std::fs::read_to_string(plan_path)?;
    let tasks = parse_tasks(&content);

    let task_numbers: HashSet<u32> = tasks.iter().map(|t| t.number).collect();
    let mut cycles = Vec::new();

    // DFS-based cycle detection
    let mut visited = HashSet::new();
    let mut in_stack = HashSet::new();

    fn dfs(
        node: u32,
        tasks: &[Task],
        task_numbers: &HashSet<u32>,
        visited: &mut HashSet<u32>,
        in_stack: &mut HashSet<u32>,
        path: &mut Vec<u32>,
        cycles: &mut Vec<String>,
    ) {
        visited.insert(node);
        in_stack.insert(node);
        path.push(node);

        if let Some(task) = tasks.iter().find(|t| t.number == node) {
            for &dep in &task.dependencies {
                if !task_numbers.contains(&dep) {
                    continue; // Skip references to non-existent tasks
                }
                if in_stack.contains(&dep) {
                    // Found a cycle — format the path from dep back to dep
                    let cycle_start = path.iter().position(|&n| n == dep).unwrap();
                    let cycle: Vec<String> = path[cycle_start..]
                        .iter()
                        .map(|n| format!("Task {}", n))
                        .collect();
                    cycles.push(format!("{} -> Task {}", cycle.join(" -> "), dep));
                } else if !visited.contains(&dep) {
                    dfs(dep, tasks, task_numbers, visited, in_stack, path, cycles);
                }
            }
        }

        path.pop();
        in_stack.remove(&node);
    }

    for task in &tasks {
        if !visited.contains(&task.number) {
            let mut path = Vec::new();
            dfs(
                task.number,
                &tasks,
                &task_numbers,
                &mut visited,
                &mut in_stack,
                &mut path,
                &mut cycles,
            );
        }
    }

    Ok(cycles)
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
        assert_eq!(tasks[0].number, 1);
        assert_eq!(tasks[0].name, "Setup infrastructure");
        assert_eq!(tasks[0].methodology, "N/A");
        assert!(tasks[0].dependencies.is_empty());
        assert_eq!(tasks[1].status, "TODO");
        assert_eq!(tasks[1].pass, 1);
        assert_eq!(tasks[1].number, 2);
        assert_eq!(tasks[1].name, "Implement core");
        assert_eq!(tasks[1].methodology, "Section 2.1");
        assert_eq!(tasks[2].status, "BLOCKED");
        assert_eq!(tasks[2].pass, 2);
        assert_eq!(tasks[2].number, 3);
        assert_eq!(tasks[2].dependencies, vec![2]);
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

    #[test]
    fn test_count_tasks_by_status_for_pass() {
        let dir = std::env::temp_dir().join("lisa_test_pass_filter");
        std::fs::create_dir_all(&dir).unwrap();
        let plan = dir.join("plan.md");

        let content = r#"# Implementation Plan

## Tasks

### Task 1: Setup
- **Status:** DONE
- **Pass:** 1

### Task 2: Core
- **Status:** TODO
- **Pass:** 1

### Task 3: Advanced
- **Status:** TODO
- **Pass:** 2

### Task 4: Polish
- **Status:** BLOCKED
- **Pass:** 2
"#;
        std::fs::write(&plan, content).unwrap();

        let pass1 = count_tasks_by_status_for_pass(&plan, 1).unwrap();
        assert_eq!(pass1.total, 2);
        assert_eq!(pass1.done, 1);
        assert_eq!(pass1.todo, 1);
        assert_eq!(pass1.blocked, 0);

        let pass2 = count_tasks_by_status_for_pass(&plan, 2).unwrap();
        assert_eq!(pass2.total, 2);
        assert_eq!(pass2.todo, 1);
        assert_eq!(pass2.blocked, 1);

        let pass3 = count_tasks_by_status_for_pass(&plan, 3).unwrap();
        assert_eq!(pass3.total, 0);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_find_next_task_selects_first_eligible() {
        let dir = std::env::temp_dir().join("lisa_test_find_next");
        std::fs::create_dir_all(&dir).unwrap();
        let plan = dir.join("plan.md");

        let content = r#"# Implementation Plan

### Task 1: Setup
- **Status:** DONE
- **Pass:** 1
- **Methodology:** N/A
- **Dependencies:** None

### Task 2: Core
- **Status:** TODO
- **Pass:** 1
- **Methodology:** Section 2.1
- **Dependencies:** Task 1

### Task 3: Advanced
- **Status:** TODO
- **Pass:** 1
- **Methodology:** Section 3.1
- **Dependencies:** Task 2
"#;
        std::fs::write(&plan, content).unwrap();

        let task = find_next_task(&plan, 1).unwrap().unwrap();
        assert_eq!(task.number, 2);
        assert_eq!(task.name, "Core");
        assert_eq!(task.methodology_ref, "Section 2.1");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_find_next_task_skips_unmet_deps() {
        let dir = std::env::temp_dir().join("lisa_test_find_skip");
        std::fs::create_dir_all(&dir).unwrap();
        let plan = dir.join("plan.md");

        let content = r#"# Plan

### Task 1: First
- **Status:** TODO
- **Pass:** 1
- **Dependencies:** Task 2

### Task 2: Second
- **Status:** TODO
- **Pass:** 1
- **Dependencies:** None
"#;
        std::fs::write(&plan, content).unwrap();

        // Task 1 depends on Task 2 which isn't DONE, so Task 2 is selected
        let task = find_next_task(&plan, 1).unwrap().unwrap();
        assert_eq!(task.number, 2);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_find_next_task_none_when_all_done() {
        let dir = std::env::temp_dir().join("lisa_test_find_none");
        std::fs::create_dir_all(&dir).unwrap();
        let plan = dir.join("plan.md");

        let content = "### Task 1: Done\n- **Status:** DONE\n- **Pass:** 1\n";
        std::fs::write(&plan, content).unwrap();

        let task = find_next_task(&plan, 1).unwrap();
        assert!(task.is_none());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_find_next_task_respects_pass() {
        let dir = std::env::temp_dir().join("lisa_test_find_pass");
        std::fs::create_dir_all(&dir).unwrap();
        let plan = dir.join("plan.md");

        let content = r#"### Task 1: Future
- **Status:** TODO
- **Pass:** 3
- **Dependencies:** None

### Task 2: Current
- **Status:** TODO
- **Pass:** 1
- **Dependencies:** None
"#;
        std::fs::write(&plan, content).unwrap();

        // Pass 1: should skip Task 1 (pass 3) and find Task 2
        let task = find_next_task(&plan, 1).unwrap().unwrap();
        assert_eq!(task.number, 2);

        // Pass 3: should find Task 1
        let task = find_next_task(&plan, 3).unwrap().unwrap();
        assert_eq!(task.number, 1);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_find_next_task_missing_methodology_defaults_empty() {
        let content =
            "### Task 1: Bare\n- **Status:** TODO\n- **Pass:** 1\n- **Dependencies:** None\n";
        let tasks = parse_tasks(content);
        assert_eq!(tasks[0].methodology, "");
    }

    #[test]
    fn test_mark_task_in_progress() {
        let dir = std::env::temp_dir().join("lisa_test_mark_ip");
        std::fs::create_dir_all(&dir).unwrap();
        let plan = dir.join("plan.md");

        let content = "### Task 1: First\n- **Status:** DONE\n- **Pass:** 1\n\n### Task 2: Second\n- **Status:** TODO\n- **Pass:** 1\n\n### Task 3: Third\n- **Status:** TODO\n- **Pass:** 2\n";
        std::fs::write(&plan, content).unwrap();

        mark_task_in_progress(&plan, 2).unwrap();

        let updated = std::fs::read_to_string(&plan).unwrap();
        assert!(updated.contains("### Task 1: First\n- **Status:** DONE"));
        assert!(updated.contains("### Task 2: Second\n- **Status:** IN_PROGRESS"));
        assert!(updated.contains("### Task 3: Third\n- **Status:** TODO"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_parse_tasks_multiple_deps() {
        let content = "### Task 5: Integration\n- **Status:** TODO\n- **Pass:** 2\n- **Dependencies:** Task 1, Task 3, Task 4\n";
        let tasks = parse_tasks(content);
        assert_eq!(tasks[0].number, 5);
        assert_eq!(tasks[0].dependencies, vec![1, 3, 4]);
    }

    #[test]
    fn test_find_next_task_selects_in_progress() {
        let dir = std::env::temp_dir().join("lisa_test_find_ip");
        std::fs::create_dir_all(&dir).unwrap();
        let plan = dir.join("plan.md");

        let content = r#"### Task 1: Done
- **Status:** DONE
- **Pass:** 1
- **Dependencies:** None

### Task 2: Interrupted
- **Status:** IN_PROGRESS
- **Pass:** 1
- **Dependencies:** Task 1

### Task 3: Pending
- **Status:** TODO
- **Pass:** 1
- **Dependencies:** Task 1
"#;
        std::fs::write(&plan, content).unwrap();

        // IN_PROGRESS tasks should be selected (retried after crash)
        let task = find_next_task(&plan, 1).unwrap().unwrap();
        assert_eq!(task.number, 2);
        assert_eq!(task.name, "Interrupted");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_missing_status_defaults_to_todo() {
        let content = "### Task 1: No status\n- **Pass:** 1\n- **Dependencies:** None\n";
        let tasks = parse_tasks(content);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].status, "TODO");
    }

    #[test]
    fn test_detect_dependency_cycles() {
        let dir = std::env::temp_dir().join("lisa_test_cycles");
        std::fs::create_dir_all(&dir).unwrap();
        let plan = dir.join("plan.md");

        let content = r#"### Task 1: A
- **Status:** TODO
- **Pass:** 1
- **Dependencies:** Task 2

### Task 2: B
- **Status:** TODO
- **Pass:** 1
- **Dependencies:** Task 1
"#;
        std::fs::write(&plan, content).unwrap();

        let cycles = detect_dependency_cycles(&plan).unwrap();
        assert!(!cycles.is_empty(), "Should detect circular dependency");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_detect_no_cycles() {
        let dir = std::env::temp_dir().join("lisa_test_no_cycles");
        std::fs::create_dir_all(&dir).unwrap();
        let plan = dir.join("plan.md");

        let content = r#"### Task 1: A
- **Status:** TODO
- **Pass:** 1
- **Dependencies:** None

### Task 2: B
- **Status:** TODO
- **Pass:** 1
- **Dependencies:** Task 1
"#;
        std::fs::write(&plan, content).unwrap();

        let cycles = detect_dependency_cycles(&plan).unwrap();
        assert!(
            cycles.is_empty(),
            "Should not detect cycles in acyclic graph"
        );

        std::fs::remove_dir_all(&dir).ok();
    }
}
