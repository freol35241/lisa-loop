use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "state")]
pub enum SpiralState {
    NotStarted,
    Scoping,
    ScopeReview,
    ScopeComplete,
    DdvAgent,
    DdvAgentReview,
    DdvAgentComplete,
    InPass { pass: u32, phase: PassPhase },
    RefineComplete { pass: u32 },
    RefineReview { pass: u32 },
    BuildComplete { pass: u32 },
    ValidateComplete { pass: u32 },
    PassReview { pass: u32 },
    Exploring { pass: u32, explore_id: u32 },
    ExploreReview { pass: u32, explore_id: u32 },
    Complete { final_pass: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "phase")]
pub enum PassPhase {
    Refine,
    Build { iteration: u32 },
    Validate,
}

impl std::fmt::Display for SpiralState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpiralState::NotStarted => write!(f, "Not started"),
            SpiralState::Scoping => write!(f, "Scoping"),
            SpiralState::ScopeReview => write!(f, "Scope review"),
            SpiralState::ScopeComplete => write!(f, "Scope complete"),
            SpiralState::DdvAgent => write!(f, "DDV Agent"),
            SpiralState::DdvAgentReview => write!(f, "DDV Agent review"),
            SpiralState::DdvAgentComplete => write!(f, "DDV Agent complete"),
            SpiralState::InPass { pass, phase } => write!(f, "Pass {} — {}", pass, phase),
            SpiralState::RefineComplete { pass } => write!(f, "Pass {} — Refine complete", pass),
            SpiralState::RefineReview { pass } => write!(f, "Pass {} — Refine review", pass),
            SpiralState::BuildComplete { pass } => write!(f, "Pass {} — Build complete", pass),
            SpiralState::ValidateComplete { pass } => {
                write!(f, "Pass {} — Validate complete", pass)
            }
            SpiralState::PassReview { pass } => write!(f, "Pass {} — Review", pass),
            SpiralState::Exploring { pass, explore_id } => {
                write!(f, "Pass {} — Exploring (id {})", pass, explore_id)
            }
            SpiralState::ExploreReview { pass, explore_id } => {
                write!(f, "Pass {} — Explore review (id {})", pass, explore_id)
            }
            SpiralState::Complete { final_pass } => write!(f, "Complete (pass {})", final_pass),
        }
    }
}

impl std::fmt::Display for PassPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PassPhase::Refine => write!(f, "Refine"),
            PassPhase::Build { iteration } => write!(f, "Build (iteration {})", iteration),
            PassPhase::Validate => write!(f, "Validate"),
        }
    }
}

/// Wrapper struct for TOML serialization
#[derive(Debug, Serialize, Deserialize)]
struct StateFile {
    #[serde(flatten)]
    state: SpiralState,
}

pub fn load_state(lisa_root: &Path) -> Result<SpiralState> {
    let state_path = lisa_root.join("state.toml");
    if !state_path.exists() {
        return Ok(SpiralState::NotStarted);
    }
    let content = std::fs::read_to_string(&state_path)
        .with_context(|| format!("Failed to read {}", state_path.display()))?;
    let file: StateFile = toml::from_str(&content).with_context(|| "Failed to parse state.toml")?;
    Ok(file.state)
}

pub fn save_state(lisa_root: &Path, state: &SpiralState) -> Result<()> {
    let state_path = lisa_root.join("state.toml");
    std::fs::create_dir_all(lisa_root)?;
    let file = StateFile {
        state: state.clone(),
    };
    let content = toml::to_string_pretty(&file).with_context(|| "Failed to serialize state")?;
    std::fs::write(&state_path, content)
        .with_context(|| format!("Failed to write {}", state_path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_roundtrip_not_started() {
        let state = SpiralState::NotStarted;
        let file = StateFile {
            state: state.clone(),
        };
        let toml_str = toml::to_string_pretty(&file).unwrap();
        let parsed: StateFile = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.state, state);
    }

    #[test]
    fn test_state_roundtrip_scoping() {
        let state = SpiralState::Scoping;
        let file = StateFile {
            state: state.clone(),
        };
        let toml_str = toml::to_string_pretty(&file).unwrap();
        let parsed: StateFile = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.state, state);
    }

    #[test]
    fn test_state_roundtrip_in_pass_build() {
        let state = SpiralState::InPass {
            pass: 3,
            phase: PassPhase::Build { iteration: 7 },
        };
        let file = StateFile {
            state: state.clone(),
        };
        let toml_str = toml::to_string_pretty(&file).unwrap();
        let parsed: StateFile = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.state, state);
    }

    #[test]
    fn test_state_roundtrip_scope_complete() {
        let state = SpiralState::ScopeComplete;
        let file = StateFile {
            state: state.clone(),
        };
        let toml_str = toml::to_string_pretty(&file).unwrap();
        let parsed: StateFile = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.state, state);
    }

    #[test]
    fn test_state_roundtrip_complete() {
        let state = SpiralState::Complete { final_pass: 4 };
        let file = StateFile {
            state: state.clone(),
        };
        let toml_str = toml::to_string_pretty(&file).unwrap();
        let parsed: StateFile = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.state, state);
    }

    #[test]
    fn test_state_roundtrip_ddv_agent() {
        let state = SpiralState::DdvAgent;
        let file = StateFile {
            state: state.clone(),
        };
        let toml_str = toml::to_string_pretty(&file).unwrap();
        let parsed: StateFile = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.state, state);
    }

    #[test]
    fn test_state_roundtrip_ddv_agent_review() {
        let state = SpiralState::DdvAgentReview;
        let file = StateFile {
            state: state.clone(),
        };
        let toml_str = toml::to_string_pretty(&file).unwrap();
        let parsed: StateFile = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.state, state);
    }

    #[test]
    fn test_state_roundtrip_ddv_agent_complete() {
        let state = SpiralState::DdvAgentComplete;
        let file = StateFile {
            state: state.clone(),
        };
        let toml_str = toml::to_string_pretty(&file).unwrap();
        let parsed: StateFile = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.state, state);
    }

    #[test]
    fn test_state_roundtrip_refine_review() {
        let state = SpiralState::RefineReview { pass: 3 };
        let file = StateFile {
            state: state.clone(),
        };
        let toml_str = toml::to_string_pretty(&file).unwrap();
        let parsed: StateFile = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.state, state);
    }

    #[test]
    fn test_state_roundtrip_refine_complete() {
        let state = SpiralState::RefineComplete { pass: 2 };
        let file = StateFile {
            state: state.clone(),
        };
        let toml_str = toml::to_string_pretty(&file).unwrap();
        let parsed: StateFile = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.state, state);
    }

    #[test]
    fn test_state_roundtrip_build_complete() {
        let state = SpiralState::BuildComplete { pass: 3 };
        let file = StateFile {
            state: state.clone(),
        };
        let toml_str = toml::to_string_pretty(&file).unwrap();
        let parsed: StateFile = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.state, state);
    }

    #[test]
    fn test_state_roundtrip_validate_complete() {
        let state = SpiralState::ValidateComplete { pass: 1 };
        let file = StateFile {
            state: state.clone(),
        };
        let toml_str = toml::to_string_pretty(&file).unwrap();
        let parsed: StateFile = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.state, state);
    }

    #[test]
    fn test_state_roundtrip_exploring() {
        let state = SpiralState::Exploring {
            pass: 2,
            explore_id: 1,
        };
        let file = StateFile {
            state: state.clone(),
        };
        let toml_str = toml::to_string_pretty(&file).unwrap();
        let parsed: StateFile = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.state, state);
    }

    #[test]
    fn test_state_roundtrip_explore_review() {
        let state = SpiralState::ExploreReview {
            pass: 3,
            explore_id: 2,
        };
        let file = StateFile {
            state: state.clone(),
        };
        let toml_str = toml::to_string_pretty(&file).unwrap();
        let parsed: StateFile = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.state, state);
    }

    #[test]
    fn test_state_display() {
        assert_eq!(format!("{}", SpiralState::NotStarted), "Not started");
        assert_eq!(
            format!("{}", SpiralState::RefineComplete { pass: 2 }),
            "Pass 2 — Refine complete"
        );
        assert_eq!(
            format!("{}", SpiralState::RefineReview { pass: 2 }),
            "Pass 2 — Refine review"
        );
        assert_eq!(
            format!("{}", SpiralState::BuildComplete { pass: 3 }),
            "Pass 3 — Build complete"
        );
        assert_eq!(
            format!("{}", SpiralState::ValidateComplete { pass: 1 }),
            "Pass 1 — Validate complete"
        );
        assert_eq!(
            format!(
                "{}",
                SpiralState::InPass {
                    pass: 2,
                    phase: PassPhase::Build { iteration: 1 }
                }
            ),
            "Pass 2 — Build (iteration 1)"
        );
    }
}
