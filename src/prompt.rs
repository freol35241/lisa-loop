use crate::config::Config;
use std::path::Path;

// Compiled-in prompts
pub const PROMPT_INIT: &str = include_str!("../prompts/PROMPT_init.md");
pub const PROMPT_SCOPE: &str = include_str!("../prompts/PROMPT_scope.md");
pub const PROMPT_REFINE: &str = include_str!("../prompts/PROMPT_refine.md");
pub const PROMPT_DDV_AGENT: &str = include_str!("../prompts/PROMPT_ddv_agent.md");
pub const PROMPT_BUILD: &str = include_str!("../prompts/PROMPT_build.md");
pub const PROMPT_VALIDATE: &str = include_str!("../prompts/PROMPT_validate.md");
pub const PROMPT_FINALIZE: &str = include_str!("../prompts/PROMPT_finalize.md");
pub const PROMPT_EXPLORE: &str = include_str!("../prompts/PROMPT_explore.md");

#[derive(Debug, Clone, Copy)]
pub enum Phase {
    Init,
    Scope,
    Refine,
    DdvAgent,
    Build,
    Validate,
    Finalize,
    Explore,
}

impl Phase {
    pub fn model_key(&self, config: &Config) -> String {
        match self {
            Phase::Init | Phase::Scope => config.models.scope.clone(),
            Phase::Refine => config.models.refine.clone(),
            Phase::DdvAgent => config.models.ddv.clone(),
            Phase::Build => config.models.build.clone(),
            Phase::Validate | Phase::Finalize => config.models.validate.clone(),
            Phase::Explore => config.models.scope.clone(),
        }
    }
}

/// Load prompt for a phase. Prefers local .lisa/prompts/ if ejected, otherwise uses compiled-in.
pub fn load_prompt(phase: Phase, lisa_root: &Path) -> String {
    let local_path = match phase {
        Phase::Init => lisa_root.join("prompts/init.md"),
        Phase::Scope => lisa_root.join("prompts/scope.md"),
        Phase::Refine => lisa_root.join("prompts/refine.md"),
        Phase::DdvAgent => lisa_root.join("prompts/ddv_agent.md"),
        Phase::Build => lisa_root.join("prompts/build.md"),
        Phase::Validate => lisa_root.join("prompts/validate.md"),
        Phase::Finalize => lisa_root.join("prompts/finalize.md"),
        Phase::Explore => lisa_root.join("prompts/explore.md"),
    };

    if local_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&local_path) {
            return content;
        }
    }

    match phase {
        Phase::Init => PROMPT_INIT.to_string(),
        Phase::Scope => PROMPT_SCOPE.to_string(),
        Phase::Refine => PROMPT_REFINE.to_string(),
        Phase::DdvAgent => PROMPT_DDV_AGENT.to_string(),
        Phase::Build => PROMPT_BUILD.to_string(),
        Phase::Validate => PROMPT_VALIDATE.to_string(),
        Phase::Finalize => PROMPT_FINALIZE.to_string(),
        Phase::Explore => PROMPT_EXPLORE.to_string(),
    }
}

/// Render the prompt with path substitutions.
/// `pass` is the current spiral pass number (used for `{{pass}}` placeholder).
pub fn render_prompt(prompt: &str, config: &Config, pass: Option<u32>) -> String {
    let lisa_root = &config.paths.lisa_root;
    let source_dirs = config.source_dirs_display();
    let tests_ddv = &config.paths.tests_ddv;
    let tests_software = &config.paths.tests_software;
    let tests_integration = &config.paths.tests_integration;
    let pass_str = pass.unwrap_or(0).to_string();

    prompt
        .replace("{{lisa_root}}", lisa_root)
        .replace("{{source_dirs}}", &source_dirs)
        .replace("{{tests_ddv}}", tests_ddv)
        .replace("{{tests_software}}", tests_software)
        .replace("{{tests_integration}}", tests_integration)
        .replace(
            "{{max_tasks_per_pass}}",
            &config.limits.max_tasks_per_pass.to_string(),
        )
        .replace("{{pass}}", &pass_str)
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
        current_pass,
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
        Phase::Refine => "Refine",
        Phase::DdvAgent => "DDV Agent",
        Phase::Build => "Build",
        Phase::Validate => "Validate",
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
tests_ddv = "tests/ddv"
tests_software = "tests/software"
tests_integration = "tests/integration"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        let prompt = "Read {{tests_ddv}}/ and {{source_dirs}}.";
        let rendered = render_prompt(prompt, &config, Some(1));
        assert_eq!(rendered, "Read tests/ddv/ and src.");
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
        assert!(!PROMPT_REFINE.is_empty());
        assert!(!PROMPT_DDV_AGENT.is_empty());
        assert!(!PROMPT_BUILD.is_empty());
        assert!(!PROMPT_VALIDATE.is_empty());
        assert!(!PROMPT_FINALIZE.is_empty());
        assert!(!PROMPT_EXPLORE.is_empty());
    }
}
