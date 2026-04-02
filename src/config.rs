use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub project: ProjectConfig,
    #[serde(default)]
    pub models: ModelsConfig,
    #[serde(default)]
    pub limits: LimitsConfig,
    #[serde(default)]
    pub review: ReviewConfig,
    #[serde(default)]
    pub git: GitConfig,
    #[serde(default)]
    pub terminal: TerminalConfig,
    #[serde(default)]
    pub paths: PathsConfig,
    #[serde(default)]
    pub commands: CommandsConfig,
    #[serde(default)]
    pub agent: AgentConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsConfig {
    #[serde(default = "default_opus")]
    pub scope: String,
    #[serde(default = "default_opus")]
    pub refine: String,
    #[serde(default = "default_sonnet")]
    pub build: String,
    #[serde(default = "default_opus")]
    pub audit: String,
}

impl Default for ModelsConfig {
    fn default() -> Self {
        Self {
            scope: default_opus(),
            refine: default_opus(),
            build: default_sonnet(),
            audit: default_opus(),
        }
    }
}

fn default_opus() -> String {
    "opus".to_string()
}
fn default_sonnet() -> String {
    "sonnet".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitsConfig {
    #[serde(default = "default_max_spiral_passes")]
    pub max_spiral_passes: u32,
    #[serde(default = "default_max_ralph_iterations")]
    pub max_ralph_iterations: u32,
    #[serde(default = "default_max_tasks_per_pass")]
    pub max_tasks_per_pass: u32,
    #[serde(default = "default_stall_threshold")]
    pub stall_threshold: u32,
    #[serde(default)]
    pub budget_usd: f64,
    #[serde(default = "default_budget_warn_pct")]
    pub budget_warn_pct: u32,
    #[serde(default = "default_idle_timeout_secs")]
    pub idle_timeout_secs: u64,
    #[serde(default = "default_max_agent_retries")]
    pub max_agent_retries: u32,
}

impl Default for LimitsConfig {
    fn default() -> Self {
        Self {
            max_spiral_passes: default_max_spiral_passes(),
            max_ralph_iterations: default_max_ralph_iterations(),
            max_tasks_per_pass: default_max_tasks_per_pass(),
            stall_threshold: default_stall_threshold(),
            budget_usd: 0.0,
            budget_warn_pct: default_budget_warn_pct(),
            idle_timeout_secs: default_idle_timeout_secs(),
            max_agent_retries: default_max_agent_retries(),
        }
    }
}

fn default_max_spiral_passes() -> u32 {
    5
}
fn default_max_ralph_iterations() -> u32 {
    15
}
fn default_max_tasks_per_pass() -> u32 {
    5
}
fn default_stall_threshold() -> u32 {
    2
}
fn default_budget_warn_pct() -> u32 {
    80
}
fn default_idle_timeout_secs() -> u64 {
    300
}
fn default_max_agent_retries() -> u32 {
    2
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewConfig {
    #[serde(default = "default_true")]
    pub pause: bool,
}

impl Default for ReviewConfig {
    fn default() -> Self {
        Self {
            pause: default_true(),
        }
    }
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    #[serde(default = "default_true")]
    pub auto_commit: bool,
    #[serde(default)]
    pub auto_push: bool,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            auto_commit: true,
            auto_push: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    #[serde(default = "default_true")]
    pub collapse_output: bool,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            collapse_output: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsConfig {
    #[serde(default = "default_lisa_root")]
    pub lisa_root: String,
    #[serde(default = "default_source")]
    pub source: Vec<String>,
    #[serde(default = "default_tests_bounds")]
    pub tests_bounds: String,
    #[serde(default = "default_tests_software")]
    pub tests_software: String,
    #[serde(default = "default_tests_integration")]
    pub tests_integration: String,
}

impl Default for PathsConfig {
    fn default() -> Self {
        Self {
            lisa_root: default_lisa_root(),
            source: default_source(),
            tests_bounds: default_tests_bounds(),
            tests_software: default_tests_software(),
            tests_integration: default_tests_integration(),
        }
    }
}

fn default_lisa_root() -> String {
    ".lisa".to_string()
}
fn default_source() -> Vec<String> {
    vec![]
}
fn default_tests_bounds() -> String {
    String::new()
}
fn default_tests_software() -> String {
    String::new()
}
fn default_tests_integration() -> String {
    String::new()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Extra CLI arguments passed to every `claude` invocation.
    /// Each element is one argument, e.g. ["--max-turns", "50"].
    #[serde(default)]
    pub extra_args: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommandsConfig {
    #[serde(default)]
    pub setup: String,
    #[serde(default)]
    pub build: String,
    #[serde(default)]
    pub test_all: String,
    #[serde(default)]
    pub test_bounds: String,
    #[serde(default)]
    pub test_software: String,
    #[serde(default)]
    pub test_integration: String,
    #[serde(default)]
    pub lint: String,
}

impl Config {
    pub fn load(project_root: &Path) -> Result<Self> {
        let config_path = project_root.join("lisa.toml");
        let content = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read {}", config_path.display()))?;
        let config: Config =
            toml::from_str(&content).with_context(|| "Failed to parse lisa.toml")?;
        Ok(config)
    }

    pub fn lisa_root(&self, project_root: &Path) -> PathBuf {
        project_root.join(&self.paths.lisa_root)
    }

    pub fn source_dirs_display(&self) -> String {
        self.paths.source.join(", ")
    }

    /// Check that paths are configured (non-empty).
    /// Returns Ok if paths are set, or an error directing the user to run init or fill lisa.toml.
    pub fn validate_paths(&self) -> Result<()> {
        if self.paths.source.is_empty()
            || self.paths.tests_bounds.is_empty()
            || self.paths.tests_software.is_empty()
            || self.paths.tests_integration.is_empty()
        {
            anyhow::bail!(
                "Paths not configured in lisa.toml [paths] section.\n\
                 Run `lisa init` to auto-detect project structure, or manually fill in:\n\
                 source, tests_bounds, tests_software, tests_integration"
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_default_config() {
        let toml_str = default_config_toml("test-project");
        let config: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.project.name, "test-project");
        assert_eq!(config.models.scope, "opus");
        assert_eq!(config.models.build, "sonnet");
        assert_eq!(config.limits.max_spiral_passes, 5);
        assert_eq!(config.limits.max_ralph_iterations, 15);
        assert_eq!(config.limits.max_tasks_per_pass, 5);
        assert_eq!(config.limits.stall_threshold, 2);
        assert!(config.review.pause);
        assert!(config.git.auto_commit);
        assert!(!config.git.auto_push);
        assert!(config.terminal.collapse_output);
        assert_eq!(config.paths.lisa_root, ".lisa");
        assert!(config.paths.source.is_empty());
        assert_eq!(config.paths.tests_bounds, "");
        assert_eq!(config.limits.idle_timeout_secs, 300);
        assert_eq!(config.limits.max_agent_retries, 2);
        assert!(config.agent.extra_args.is_empty());
    }

    #[test]
    fn test_parse_agent_extra_args() {
        let toml_str = r#"
[project]
name = "with-args"

[agent]
extra_args = ["--max-turns", "50", "--verbose"]
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(
            config.agent.extra_args,
            vec!["--max-turns", "50", "--verbose"]
        );
    }

    #[test]
    fn test_parse_minimal_config() {
        let toml_str = r#"
[project]
name = "minimal"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.project.name, "minimal");
        // All defaults should apply
        assert_eq!(config.models.scope, "opus");
        assert_eq!(config.limits.max_spiral_passes, 5);
        assert!(config.review.pause);
        assert!(config.agent.extra_args.is_empty());
    }

    #[test]
    fn test_source_dirs_display() {
        let toml_str = default_config_toml("test");
        let config: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.source_dirs_display(), "");
    }

    #[test]
    fn test_validate_paths_empty() {
        let toml_str = default_config_toml("test");
        let config: Config = toml::from_str(&toml_str).unwrap();
        assert!(config.validate_paths().is_err());
    }

    #[test]
    fn test_validate_paths_filled() {
        let toml_str = r#"
[project]
name = "filled"

[paths]
source = ["src"]
tests_bounds = "tests/bounds"
tests_software = "tests/software"
tests_integration = "tests/integration"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.validate_paths().is_ok());
    }
}

pub fn default_config_toml(name: &str) -> String {
    format!(
        r#"[project]
name = "{name}"

[models]
scope = "opus"
refine = "opus"
build = "sonnet"
audit = "opus"

[limits]
max_spiral_passes = 5
max_ralph_iterations = 15
max_tasks_per_pass = 5
stall_threshold = 2
# budget_usd = 0.0       # 0 = unlimited
# budget_warn_pct = 80   # warn at this % of budget
idle_timeout_secs = 300  # kill agent after 5 min with no output
max_agent_retries = 2    # auto-retry on idle timeout before surfacing to human

[review]
# Human review gates. When false, loop runs fully autonomously.
pause = true

[git]
auto_commit = true
auto_push = false

[terminal]
# Collapse agent streaming output to summary lines after completion
collapse_output = true

[paths]
# Where process artifacts live (relative to project root)
lisa_root = ".lisa"

# Where deliverable code goes (relative to project root).
# Resolved by the init agent; fill manually if needed.
source = []

# Test directories (relative to project root).
# Resolved by the init agent; fill manually if needed.
tests_bounds = ""
tests_software = ""
tests_integration = ""

[agent]
# Extra CLI flags passed to every claude invocation.
# Each element becomes one argument, e.g. ["--max-turns", "50"]
extra_args = []

[commands]
# These get populated by the scope agent, but can be pre-filled
setup = ""
build = ""
test_all = ""
test_bounds = ""
test_software = ""
test_integration = ""
lint = ""
"#
    )
}
