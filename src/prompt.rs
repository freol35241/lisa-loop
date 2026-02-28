use crate::config::Config;
use std::path::Path;

// Compiled-in prompts
pub const PROMPT_SCOPE: &str = include_str!("../prompts/PROMPT_scope.md");
pub const PROMPT_REFINE: &str = include_str!("../prompts/PROMPT_refine.md");
pub const PROMPT_DDV_RED: &str = include_str!("../prompts/PROMPT_ddv_red.md");
pub const PROMPT_BUILD: &str = include_str!("../prompts/PROMPT_build.md");
pub const PROMPT_EXECUTE: &str = include_str!("../prompts/PROMPT_execute.md");
pub const PROMPT_VALIDATE: &str = include_str!("../prompts/PROMPT_validate.md");
pub const PROMPT_FINALIZE: &str = include_str!("../prompts/PROMPT_finalize.md");

#[derive(Debug, Clone, Copy)]
pub enum Phase {
    Scope,
    Refine,
    DdvRed,
    Build,
    Execute,
    Validate,
    Finalize,
}

impl Phase {
    pub fn model_key(&self, config: &Config) -> String {
        match self {
            Phase::Scope => config.models.scope.clone(),
            Phase::Refine => config.models.refine.clone(),
            Phase::DdvRed => config.models.ddv.clone(),
            Phase::Build => config.models.build.clone(),
            Phase::Execute => config.models.execute.clone(),
            Phase::Validate | Phase::Finalize => config.models.validate.clone(),
        }
    }
}

/// Load prompt for a phase. Prefers local .lisa/prompts/ if ejected, otherwise uses compiled-in.
pub fn load_prompt(phase: Phase, lisa_root: &Path) -> String {
    let local_path = match phase {
        Phase::Scope => lisa_root.join("prompts/scope.md"),
        Phase::Refine => lisa_root.join("prompts/refine.md"),
        Phase::DdvRed => lisa_root.join("prompts/ddv_red.md"),
        Phase::Build => lisa_root.join("prompts/build.md"),
        Phase::Execute => lisa_root.join("prompts/execute.md"),
        Phase::Validate => lisa_root.join("prompts/validate.md"),
        Phase::Finalize => lisa_root.join("prompts/finalize.md"),
    };

    if local_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&local_path) {
            return content;
        }
    }

    match phase {
        Phase::Scope => PROMPT_SCOPE.to_string(),
        Phase::Refine => PROMPT_REFINE.to_string(),
        Phase::DdvRed => PROMPT_DDV_RED.to_string(),
        Phase::Build => PROMPT_BUILD.to_string(),
        Phase::Execute => PROMPT_EXECUTE.to_string(),
        Phase::Validate => PROMPT_VALIDATE.to_string(),
        Phase::Finalize => PROMPT_FINALIZE.to_string(),
    }
}

/// Render the prompt with path substitutions
pub fn render_prompt(prompt: &str, config: &Config) -> String {
    let lisa_root = &config.paths.lisa_root;
    let source_dirs = config.source_dirs_display();
    let tests_ddv = &config.paths.tests_ddv;
    let tests_software = &config.paths.tests_software;
    let tests_integration = &config.paths.tests_integration;

    prompt
        .replace("{{lisa_root}}", lisa_root)
        .replace("{{source_dirs}}", &source_dirs)
        .replace("{{tests_ddv}}", tests_ddv)
        .replace("{{tests_software}}", tests_software)
        .replace("{{tests_integration}}", tests_integration)
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
- AGENTS: {}/AGENTS.md
- Methodology: {}/methodology/
- Spiral: {}/spiral/
- Validation: {}/validation/
- References: {}/references/
- Plots: {}/plots/
- Source code: {} (deliverable)
- DDV tests: {}
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
        source_dirs,
        config.paths.tests_ddv,
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
        Phase::Scope => "Scope",
        Phase::Refine => "Refine",
        Phase::DdvRed => "DDV Red",
        Phase::Build => "Build",
        Phase::Execute => "Execute",
        Phase::Validate => "Validate",
        Phase::Finalize => "Finalize",
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
    let rendered = render_prompt(&prompt, config);

    let mut input = preamble;
    if let Some(extra) = extra_context {
        input.push('\n');
        input.push_str(extra);
        input.push_str("\n\n");
    }
    input.push_str(&rendered);
    input
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
        let prompt = "Read ASSIGNMENT.md and {{tests_ddv}}/ tests.";
        let rendered = render_prompt(prompt, &config);
        assert_eq!(rendered, "Read ASSIGNMENT.md and tests/ddv/ tests.");
    }

    #[test]
    fn test_render_prompt_source_dirs() {
        let config = test_config();
        let prompt = "Source at {{source_dirs}}.";
        let rendered = render_prompt(prompt, &config);
        assert_eq!(rendered, "Source at src.");
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
    }

    #[test]
    fn test_context_preamble_with_redirect() {
        let config = test_config();
        let preamble = build_context_preamble(&config, 2, "Refine", true);
        assert!(preamble.contains("Human redirect: .lisa/spiral/pass-1/human-redirect.md"));
    }

    #[test]
    fn test_compiled_prompts_not_empty() {
        assert!(!PROMPT_SCOPE.is_empty());
        assert!(!PROMPT_REFINE.is_empty());
        assert!(!PROMPT_DDV_RED.is_empty());
        assert!(!PROMPT_BUILD.is_empty());
        assert!(!PROMPT_EXECUTE.is_empty());
        assert!(!PROMPT_VALIDATE.is_empty());
        assert!(!PROMPT_FINALIZE.is_empty());
    }
}
