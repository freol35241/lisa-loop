use crate::config::Config;
use crate::terminal;
use anyhow::Result;
use std::path::Path;

// Compiled-in prompts
pub const PROMPT_INIT: &str = include_str!("../prompts/PROMPT_init.md");
/// Legacy scope prompt — kept for backward compatibility with ejected prompts.
/// New code uses Research, ValidationDesign, and Planning phases instead.
pub const PROMPT_SCOPE: &str = include_str!("../prompts/PROMPT_scope.md");
pub const PROMPT_RESEARCH: &str = include_str!("../prompts/PROMPT_research.md");
pub const PROMPT_VALIDATION_DESIGN: &str = include_str!("../prompts/PROMPT_validation_design.md");
pub const PROMPT_PLANNING: &str = include_str!("../prompts/PROMPT_planning.md");
pub const PROMPT_REFINE: &str = include_str!("../prompts/PROMPT_refine.md");
pub const PROMPT_REFINE_METHODOLOGY: &str = include_str!("../prompts/PROMPT_refine_methodology.md");
pub const PROMPT_REFINE_PLAN: &str = include_str!("../prompts/PROMPT_refine_plan.md");
pub const PROMPT_BOUNDS: &str = include_str!("../prompts/PROMPT_bounds.md");
pub const PROMPT_BUILD: &str = include_str!("../prompts/PROMPT_build.md");
pub const PROMPT_AUDIT: &str = include_str!("../prompts/PROMPT_audit.md");
pub const PROMPT_FINALIZE: &str = include_str!("../prompts/PROMPT_finalize.md");
pub const PROMPT_EXPLORE: &str = include_str!("../prompts/PROMPT_explore.md");

// Compiled-in skill files
pub const SKILL_ENGINEERING_JUDGMENT: &str = include_str!("../skills/engineering_judgment.md");
pub const SKILL_DIMENSIONAL_ANALYSIS: &str = include_str!("../skills/dimensional_analysis.md");
pub const SKILL_NUMERICAL_STABILITY: &str = include_str!("../skills/numerical_stability.md");
pub const SKILL_LITERATURE_GROUNDING: &str = include_str!("../skills/literature_grounding.md");

pub const SKILLS: &[(&str, &str)] = &[
    ("engineering-judgment.md", SKILL_ENGINEERING_JUDGMENT),
    ("dimensional-analysis.md", SKILL_DIMENSIONAL_ANALYSIS),
    ("numerical-stability.md", SKILL_NUMERICAL_STABILITY),
    ("literature-grounding.md", SKILL_LITERATURE_GROUNDING),
];

// Compiled-in scope artifact specs (read on demand by the scoping agent)
const SCOPE_SPEC_METHODOLOGY: &str = include_str!("../prompts/scope/methodology_spec.md");
const SCOPE_SPEC_LITERATURE_SURVEY: &str =
    include_str!("../prompts/scope/literature_survey_spec.md");
const SCOPE_SPEC_SPIRAL_PLAN: &str = include_str!("../prompts/scope/spiral_plan_spec.md");
const SCOPE_SPEC_STACK_SELECTION: &str = include_str!("../prompts/scope/stack_selection_spec.md");
const SCOPE_SPEC_VALIDATION: &str = include_str!("../prompts/scope/validation_specs.md");
const SCOPE_SPEC_IMPLEMENTATION_PLAN: &str =
    include_str!("../prompts/scope/implementation_plan_spec.md");
pub const SCOPE_SPECS: &[(&str, &str)] = &[
    ("methodology_spec.md", SCOPE_SPEC_METHODOLOGY),
    ("literature_survey_spec.md", SCOPE_SPEC_LITERATURE_SURVEY),
    ("spiral_plan_spec.md", SCOPE_SPEC_SPIRAL_PLAN),
    ("stack_selection_spec.md", SCOPE_SPEC_STACK_SELECTION),
    ("validation_specs.md", SCOPE_SPEC_VALIDATION),
    (
        "implementation_plan_spec.md",
        SCOPE_SPEC_IMPLEMENTATION_PLAN,
    ),
];

#[derive(Debug, Clone, Copy)]
pub enum Phase {
    Init,
    /// Legacy: kept for ejected-prompt backward compat. Use Research/ValidationDesign/Planning.
    #[allow(dead_code)]
    Scope,
    Research,
    ValidationDesign,
    Planning,
    /// Legacy: kept for ejected-prompt backward compat. Use RefineMethodology/RefinePlan.
    #[allow(dead_code)]
    Refine,
    RefineMethodology,
    RefinePlan,
    Bounds,
    Build,
    Audit,
    Finalize,
    Explore,
}

impl Phase {
    pub fn model_key(&self, config: &Config) -> String {
        match self {
            Phase::Init
            | Phase::Scope
            | Phase::Research
            | Phase::ValidationDesign
            | Phase::Planning => config.models.scope.clone(),
            Phase::Refine | Phase::RefineMethodology | Phase::RefinePlan => {
                config.models.refine.clone()
            }
            Phase::Bounds => config.models.bounds.clone(),
            Phase::Build => config.models.build.clone(),
            Phase::Audit | Phase::Finalize => config.models.audit.clone(),
            Phase::Explore => config.models.scope.clone(),
        }
    }
}

/// Load prompt for a phase. Prefers local .lisa/prompts/ if ejected, otherwise uses compiled-in.
pub fn load_prompt(phase: Phase, lisa_root: &Path) -> String {
    let local_path = match phase {
        Phase::Init => lisa_root.join("prompts/init.md"),
        Phase::Scope => lisa_root.join("prompts/scope.md"),
        Phase::Research => lisa_root.join("prompts/research.md"),
        Phase::ValidationDesign => lisa_root.join("prompts/validation_design.md"),
        Phase::Planning => lisa_root.join("prompts/planning.md"),
        Phase::Refine => lisa_root.join("prompts/refine.md"),
        Phase::RefineMethodology => lisa_root.join("prompts/refine_methodology.md"),
        Phase::RefinePlan => lisa_root.join("prompts/refine_plan.md"),
        Phase::Bounds => lisa_root.join("prompts/bounds.md"),
        Phase::Build => lisa_root.join("prompts/build.md"),
        Phase::Audit => lisa_root.join("prompts/audit.md"),
        Phase::Finalize => lisa_root.join("prompts/finalize.md"),
        Phase::Explore => lisa_root.join("prompts/explore.md"),
    };

    if local_path.exists() {
        match std::fs::read_to_string(&local_path) {
            Ok(content) => return content,
            Err(e) => {
                terminal::log_warn(&format!(
                    "Could not read ejected prompt {}: {} — using built-in prompt",
                    local_path.display(),
                    e
                ));
            }
        }
    }

    match phase {
        Phase::Init => PROMPT_INIT.to_string(),
        Phase::Scope => PROMPT_SCOPE.to_string(),
        Phase::Research => PROMPT_RESEARCH.to_string(),
        Phase::ValidationDesign => PROMPT_VALIDATION_DESIGN.to_string(),
        Phase::Planning => PROMPT_PLANNING.to_string(),
        Phase::Refine => PROMPT_REFINE.to_string(),
        Phase::RefineMethodology => PROMPT_REFINE_METHODOLOGY.to_string(),
        Phase::RefinePlan => PROMPT_REFINE_PLAN.to_string(),
        Phase::Bounds => PROMPT_BOUNDS.to_string(),
        Phase::Build => PROMPT_BUILD.to_string(),
        Phase::Audit => PROMPT_AUDIT.to_string(),
        Phase::Finalize => PROMPT_FINALIZE.to_string(),
        Phase::Explore => PROMPT_EXPLORE.to_string(),
    }
}

/// Render the prompt with path substitutions.
/// `pass` is the current spiral pass number (used for `{{pass}}` placeholder).
pub fn render_prompt(prompt: &str, config: &Config, pass: Option<u32>) -> String {
    let lisa_root = &config.paths.lisa_root;
    let source_dirs = config.source_dirs_display();
    let tests_bounds = &config.paths.tests_bounds;
    let tests_software = &config.paths.tests_software;
    let tests_integration = &config.paths.tests_integration;
    let pass_str = pass.unwrap_or(0).to_string();

    let result = prompt
        .replace("{{lisa_root}}", lisa_root)
        .replace("{{source_dirs}}", &source_dirs)
        .replace("{{tests_bounds}}", tests_bounds)
        .replace("{{tests_software}}", tests_software)
        .replace("{{tests_integration}}", tests_integration)
        .replace(
            "{{max_tasks_per_pass}}",
            &config.limits.max_tasks_per_pass.to_string(),
        )
        .replace("{{pass}}", &pass_str);

    // Warn about unrecognized placeholders
    let placeholder_re = regex::Regex::new(r"\{\{(\w+)\}\}").unwrap();
    for cap in placeholder_re.captures_iter(&result) {
        terminal::log_warn(&format!(
            "Unrecognized prompt placeholder: {{{{{}}}}} — left as-is",
            &cap[1]
        ));
    }

    result
}

/// Build the context preamble that gets prepended to every agent invocation
pub fn build_context_preamble(
    config: &Config,
    current_pass: u32,
    current_phase: &str,
    human_redirect: bool,
) -> String {
    let lisa_root = &config.paths.lisa_root;
    let source_dirs = config.source_dirs_display();

    let mut ctx = format!(
        r#"## Lisa Loop Context

### Project
- Name: {}
- Lisa root: {}

### Paths
- ASSIGNMENT: ASSIGNMENT.md
- Stack: {}/STACK.md
- Methodology: {}/methodology/
- Spiral: {}/spiral/
- Validation: {}/validation/
- References: {}/references/
- Plots: {}/spiral/pass-{}/plots/
- Source code: {} (deliverable)
- Bounds tests: {}
- Software tests: {}
- Integration tests: {}

### Current State
- Spiral pass: {}
- Phase: {}
"#,
        config.project.name,
        lisa_root,
        lisa_root,
        lisa_root,
        lisa_root,
        lisa_root,
        lisa_root,
        lisa_root,
        current_pass,
        source_dirs,
        config.paths.tests_bounds,
        config.paths.tests_software,
        config.paths.tests_integration,
        current_pass,
        current_phase,
    );

    if current_pass > 0 {
        let prev_pass = current_pass - 1;
        ctx.push_str(&format!(
            "- Previous pass results: {}/spiral/pass-{}/\n",
            lisa_root, prev_pass
        ));
    }

    if human_redirect && current_pass > 0 {
        let prev_pass = current_pass - 1;
        ctx.push_str(&format!(
            "- Human redirect: {}/spiral/pass-{}/human-redirect.md\n",
            lisa_root, prev_pass
        ));
    }

    // Idle timeout heartbeat guidance
    ctx.push_str(&format!(
        "\n### Heartbeat\n\
         Lisa Loop will kill your process if no tool output is received for {} seconds.\n\
         During long-running operations (compilation, test suites, complex analysis),\n\
         emit periodic heartbeats by running: `echo \"[heartbeat] still working...\"`\n\
         Do this every few minutes during operations that may take a while.\n",
        config.limits.idle_timeout_secs
    ));

    ctx
}

/// Build complete prompt input for an agent: context preamble + rendered prompt
pub fn build_agent_input(
    phase: Phase,
    config: &Config,
    lisa_root: &Path,
    current_pass: u32,
    extra_context: Option<&str>,
) -> String {
    let phase_name = match phase {
        Phase::Init => "Init",
        Phase::Scope => "Scope",
        Phase::Research => "Research",
        Phase::ValidationDesign => "Validation Design",
        Phase::Planning => "Planning",
        Phase::Refine => "Refine",
        Phase::RefineMethodology => "Refine Methodology",
        Phase::RefinePlan => "Refine Plan",
        Phase::Bounds => "Bounds",
        Phase::Build => "Build",
        Phase::Audit => "Audit",
        Phase::Finalize => "Finalize",
        Phase::Explore => "Explore",
    };

    let has_redirect = if current_pass > 0 {
        let prev = current_pass - 1;
        lisa_root
            .join(format!("spiral/pass-{}/human-redirect.md", prev))
            .exists()
    } else {
        false
    };

    let preamble = build_context_preamble(config, current_pass, phase_name, has_redirect);
    let prompt = load_prompt(phase, lisa_root);
    let rendered = render_prompt(&prompt, config, Some(current_pass));

    let mut input = preamble;
    if let Some(extra) = extra_context {
        input.push('\n');
        input.push_str(extra);
        input.push_str("\n\n");
    }
    input.push_str(&rendered);
    input
}

/// Write scope artifact spec files to .lisa/prompts/scope/ if they don't already exist.
/// Renders {{placeholder}} substitutions so the agent sees concrete paths and values.
/// Preserves user-customized files (same pattern as eject-prompts).
pub fn ensure_scope_specs(lisa_root: &Path, config: &Config) -> Result<()> {
    let scope_dir = lisa_root.join("prompts/scope");
    std::fs::create_dir_all(&scope_dir)?;
    for (filename, content) in SCOPE_SPECS {
        let path = scope_dir.join(filename);
        if !path.exists() {
            let rendered = render_prompt(content, config, Some(0));
            std::fs::write(&path, rendered)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;

    fn test_config() -> Config {
        let toml_str = config::default_config_toml("test-project");
        toml::from_str(&toml_str).unwrap()
    }

    #[test]
    fn test_render_prompt_substitutions() {
        let config = test_config();
        let prompt = "Read ASSIGNMENT.md and {{tests_bounds}}/ tests.";
        let rendered = render_prompt(prompt, &config, None);
        assert_eq!(rendered, "Read ASSIGNMENT.md and / tests.");
    }

    #[test]
    fn test_render_prompt_source_dirs() {
        let config = test_config();
        let prompt = "Source at {{source_dirs}}.";
        let rendered = render_prompt(prompt, &config, None);
        assert_eq!(rendered, "Source at .");
    }

    #[test]
    fn test_render_prompt_substitutions_with_filled_paths() {
        let toml_str = r#"
[project]
name = "test"

[paths]
source = ["src"]
tests_bounds = "tests/bounds"
tests_software = "tests/software"
tests_integration = "tests/integration"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        let prompt = "Read {{tests_bounds}}/ and {{source_dirs}}.";
        let rendered = render_prompt(prompt, &config, Some(1));
        assert_eq!(rendered, "Read tests/bounds/ and src.");
    }

    #[test]
    fn test_render_prompt_pass_placeholder() {
        let config = test_config();
        let prompt = "Plots at .lisa/spiral/pass-{{pass}}/plots/";
        let rendered = render_prompt(prompt, &config, Some(3));
        assert_eq!(rendered, "Plots at .lisa/spiral/pass-3/plots/");
    }

    #[test]
    fn test_context_preamble_pass_0() {
        let config = test_config();
        let preamble = build_context_preamble(&config, 0, "Scope", false);
        assert!(preamble.contains("Name: test-project"));
        assert!(preamble.contains("Spiral pass: 0"));
        assert!(preamble.contains("Phase: Scope"));
        assert!(!preamble.contains("Previous pass"));
    }

    #[test]
    fn test_context_preamble_pass_2() {
        let config = test_config();
        let preamble = build_context_preamble(&config, 2, "Build", false);
        assert!(preamble.contains("Spiral pass: 2"));
        assert!(preamble.contains("Previous pass results: .lisa/spiral/pass-1/"));
        assert!(preamble.contains("Plots: .lisa/spiral/pass-2/plots/"));
    }

    #[test]
    fn test_context_preamble_with_redirect() {
        let config = test_config();
        let preamble = build_context_preamble(&config, 2, "Refine", true);
        assert!(preamble.contains("Human redirect: .lisa/spiral/pass-1/human-redirect.md"));
    }

    #[test]
    fn test_compiled_prompts_not_empty() {
        assert!(!PROMPT_INIT.is_empty());
        assert!(!PROMPT_SCOPE.is_empty());
        assert!(!PROMPT_RESEARCH.is_empty());
        assert!(!PROMPT_VALIDATION_DESIGN.is_empty());
        assert!(!PROMPT_PLANNING.is_empty());
        assert!(!PROMPT_REFINE.is_empty());
        assert!(!PROMPT_REFINE_METHODOLOGY.is_empty());
        assert!(!PROMPT_REFINE_PLAN.is_empty());
        assert!(!PROMPT_BOUNDS.is_empty());
        assert!(!PROMPT_BUILD.is_empty());
        assert!(!PROMPT_AUDIT.is_empty());
        assert!(!PROMPT_FINALIZE.is_empty());
        assert!(!PROMPT_EXPLORE.is_empty());
    }

    #[test]
    fn test_scope_specs_not_empty() {
        assert_eq!(SCOPE_SPECS.len(), 6);
        for (filename, content) in SCOPE_SPECS {
            assert!(!filename.is_empty(), "spec filename is empty");
            assert!(
                !content.is_empty(),
                "spec content is empty for {}",
                filename
            );
        }
    }

    #[test]
    fn test_scope_prompt_references_all_specs() {
        for (filename, _) in SCOPE_SPECS {
            let reference = format!("prompts/scope/{}", filename);
            assert!(
                PROMPT_SCOPE.contains(&reference),
                "PROMPT_scope.md does not reference {}",
                filename,
            );
        }
    }

    #[test]
    fn test_ensure_scope_specs_creates_files() {
        let tmp = tempfile::tempdir().unwrap();
        let lisa_root = tmp.path();
        let config = test_config();
        ensure_scope_specs(lisa_root, &config).unwrap();

        for (filename, _) in SCOPE_SPECS {
            let path = lisa_root.join("prompts/scope").join(filename);
            assert!(path.exists(), "{} was not created", filename);
            let written = std::fs::read_to_string(&path).unwrap();
            // Specs should be rendered — no raw {{lisa_root}} placeholders remaining
            assert!(
                !written.contains("{{lisa_root}}"),
                "{} still contains unrendered {{{{lisa_root}}}} placeholder",
                filename,
            );
        }
    }

    #[test]
    fn test_ensure_scope_specs_preserves_existing() {
        let tmp = tempfile::tempdir().unwrap();
        let lisa_root = tmp.path();
        let config = test_config();
        let scope_dir = lisa_root.join("prompts/scope");
        std::fs::create_dir_all(&scope_dir).unwrap();

        // Write a customized version of the first spec
        let (filename, _) = SCOPE_SPECS[0];
        let custom = "# My custom spec\n";
        std::fs::write(scope_dir.join(filename), custom).unwrap();

        ensure_scope_specs(lisa_root, &config).unwrap();

        // Custom file should be preserved
        let content = std::fs::read_to_string(scope_dir.join(filename)).unwrap();
        assert_eq!(content, custom, "existing file was overwritten");

        // Other files should have been created
        for (fname, _) in &SCOPE_SPECS[1..] {
            let path = scope_dir.join(fname);
            assert!(path.exists(), "{} was not created", fname);
        }
    }
}
