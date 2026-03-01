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
    #[serde(default = "default_opus")]
    pub ddv: String,
    #[serde(default = "default_sonnet")]
    pub build: String,
    #[serde(default = "default_opus")]
    pub execute: String,
    #[serde(default = "default_opus")]
    pub validate: String,
}

impl Default for ModelsConfig {
    fn default() -> Self {
        Self {
            scope: default_opus(),
            refine: default_opus(),
            ddv: default_opus(),
            build: default_sonnet(),
            execute: default_opus(),
            validate: default_opus(),
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
    #[serde(default = "default_stall_threshold")]
    pub stall_threshold: u32,
    #[serde(default)]
    pub budget_usd: f64,
    #[serde(default = "default_budget_warn_pct")]
    pub budget_warn_pct: u32,
}

impl Default for LimitsConfig {
    fn default() -> Self {
        Self {
            max_spiral_passes: default_max_spiral_passes(),
            max_ralph_iterations: default_max_ralph_iterations(),
            stall_threshold: default_stall_threshold(),
            budget_usd: 0.0,
            budget_warn_pct: default_budget_warn_pct(),
        }
    }
}

fn default_max_spiral_passes() -> u32 {
    5
}
fn default_max_ralph_iterations() -> u32 {
    50
}
fn default_stall_threshold() -> u32 {
    2
}
fn default_budget_warn_pct() -> u32 {
    80
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
    #[serde(default = "default_tests_ddv")]
    pub tests_ddv: String,
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
            tests_ddv: default_tests_ddv(),
            tests_software: default_tests_software(),
            tests_integration: default_tests_integration(),
        }
    }
}

fn default_lisa_root() -> String {
    ".lisa".to_string()
}
fn default_source() -> Vec<String> {
    vec!["src".to_string()]
}
fn default_tests_ddv() -> String {
    "tests/ddv".to_string()
}
fn default_tests_software() -> String {
    "tests/software".to_string()
}
fn default_tests_integration() -> String {
    "tests/integration".to_string()
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
    pub test_ddv: String,
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
        assert_eq!(config.limits.max_ralph_iterations, 50);
        assert_eq!(config.limits.stall_threshold, 2);
        assert!(config.review.pause);
        assert!(config.git.auto_commit);
        assert!(!config.git.auto_push);
        assert!(config.terminal.collapse_output);
        assert_eq!(config.paths.lisa_root, ".lisa");
        assert_eq!(config.paths.source, vec!["src"]);
        assert_eq!(config.paths.tests_ddv, "tests/ddv");
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
    }

    #[test]
    fn test_source_dirs_display() {
        let toml_str = default_config_toml("test");
        let config: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.source_dirs_display(), "src");
    }
}

pub fn default_config_toml(name: &str) -> String {
    format!(
        r#"[project]
name = "{name}"

[models]
scope = "opus"
refine = "opus"
ddv = "opus"
build = "sonnet"
execute = "opus"
validate = "opus"

[limits]
max_spiral_passes = 5
max_ralph_iterations = 50
stall_threshold = 2
# budget_usd = 0.0       # 0 = unlimited
# budget_warn_pct = 80   # warn at this % of budget

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

# Where deliverable code goes (relative to project root)
source = ["src"]

# Test directories (relative to project root)
tests_ddv = "tests/ddv"
tests_software = "tests/software"
tests_integration = "tests/integration"

[commands]
# These get populated by the scope agent, but can be pre-filled
setup = ""
build = ""
test_all = ""
test_ddv = ""
test_software = ""
test_integration = ""
lint = ""
"#
    )
}
