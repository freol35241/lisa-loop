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
    ScopeResearch,
    ScopeResearchReview,
    ScopeValidation,
    ScopePlanning,
    InPass { pass: u32, phase: PassPhase },
    RefineMethodologyComplete { pass: u32 },
    RefineComplete { pass: u32 },
    RefineReview { pass: u32 },
    BuildComplete { pass: u32 },
    AuditComplete { pass: u32 },
    PassReview { pass: u32 },
    Exploring { pass: u32, explore_id: u32 },
    ExploreReview { pass: u32, explore_id: u32 },
    Complete { final_pass: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "phase")]
pub enum PassPhase {
    Refine,
    Bounds {
        task_id: u32,
    },
    Build {
        #[serde(default)]
        task_id: u32,
        iteration: u32,
    },
    Audit,
}

impl std::fmt::Display for SpiralState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpiralState::NotStarted => write!(f, "Not started"),
            SpiralState::Scoping => write!(f, "Scoping"),
            SpiralState::ScopeReview => write!(f, "Scope review"),
            SpiralState::ScopeComplete => write!(f, "Scope complete"),
            SpiralState::ScopeResearch => write!(f, "Scope — Research"),
            SpiralState::ScopeResearchReview => write!(f, "Scope — Research review"),
            SpiralState::ScopeValidation => write!(f, "Scope — Validation design"),
            SpiralState::ScopePlanning => write!(f, "Scope — Planning"),
            SpiralState::InPass { pass, phase } => write!(f, "Pass {} — {}", pass, phase),
            SpiralState::RefineMethodologyComplete { pass } => {
                write!(f, "Pass {} — Refine methodology complete", pass)
            }
            SpiralState::RefineComplete { pass } => write!(f, "Pass {} — Refine complete", pass),
            SpiralState::RefineReview { pass } => write!(f, "Pass {} — Refine review", pass),
            SpiralState::BuildComplete { pass } => write!(f, "Pass {} — Build complete", pass),
            SpiralState::AuditComplete { pass } => {
                write!(f, "Pass {} — Audit complete", pass)
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
            PassPhase::Bounds { task_id } => write!(f, "Bounds (task {})", task_id),
            PassPhase::Build {
                task_id, iteration, ..
            } => {
                if *task_id > 0 {
                    write!(f, "Build (task {}, iteration {})", task_id, iteration)
                } else {
                    write!(f, "Build (iteration {})", iteration)
                }
            }
            PassPhase::Audit => write!(f, "Audit"),
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
            phase: PassPhase::Build {
                task_id: 0,
                iteration: 7,
            },
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
    fn test_state_roundtrip_audit_complete() {
        let state = SpiralState::AuditComplete { pass: 1 };
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
            format!("{}", SpiralState::AuditComplete { pass: 1 }),
            "Pass 1 — Audit complete"
        );
        assert_eq!(
            format!(
                "{}",
                SpiralState::InPass {
                    pass: 2,
                    phase: PassPhase::Build {
                        task_id: 0,
                        iteration: 1
                    }
                }
            ),
            "Pass 2 — Build (iteration 1)"
        );
        assert_eq!(
            format!(
                "{}",
                SpiralState::InPass {
                    pass: 2,
                    phase: PassPhase::Build {
                        task_id: 3,
                        iteration: 1
                    }
                }
            ),
            "Pass 2 — Build (task 3, iteration 1)"
        );
        assert_eq!(
            format!(
                "{}",
                SpiralState::InPass {
                    pass: 1,
                    phase: PassPhase::Bounds { task_id: 2 }
                }
            ),
            "Pass 1 — Bounds (task 2)"
        );
        assert_eq!(
            format!("{}", SpiralState::ScopeResearch),
            "Scope — Research"
        );
        assert_eq!(
            format!("{}", SpiralState::ScopeResearchReview),
            "Scope — Research review"
        );
        assert_eq!(
            format!("{}", SpiralState::ScopeValidation),
            "Scope — Validation design"
        );
        assert_eq!(
            format!("{}", SpiralState::ScopePlanning),
            "Scope — Planning"
        );
    }

    #[test]
    fn test_state_roundtrip_scope_research() {
        let state = SpiralState::ScopeResearch;
        let file = StateFile {
            state: state.clone(),
        };
        let toml_str = toml::to_string_pretty(&file).unwrap();
        let parsed: StateFile = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.state, state);
    }

    #[test]
    fn test_state_roundtrip_scope_research_review() {
        let state = SpiralState::ScopeResearchReview;
        let file = StateFile {
            state: state.clone(),
        };
        let toml_str = toml::to_string_pretty(&file).unwrap();
        let parsed: StateFile = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.state, state);
    }

    #[test]
    fn test_state_roundtrip_scope_validation() {
        let state = SpiralState::ScopeValidation;
        let file = StateFile {
            state: state.clone(),
        };
        let toml_str = toml::to_string_pretty(&file).unwrap();
        let parsed: StateFile = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.state, state);
    }

    #[test]
    fn test_state_roundtrip_scope_planning() {
        let state = SpiralState::ScopePlanning;
        let file = StateFile {
            state: state.clone(),
        };
        let toml_str = toml::to_string_pretty(&file).unwrap();
        let parsed: StateFile = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.state, state);
    }

    #[test]
    fn test_state_roundtrip_in_pass_bounds() {
        let state = SpiralState::InPass {
            pass: 2,
            phase: PassPhase::Bounds { task_id: 5 },
        };
        let file = StateFile {
            state: state.clone(),
        };
        let toml_str = toml::to_string_pretty(&file).unwrap();
        let parsed: StateFile = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.state, state);
    }

    #[test]
    fn test_state_roundtrip_in_pass_build_with_task() {
        let state = SpiralState::InPass {
            pass: 1,
            phase: PassPhase::Build {
                task_id: 3,
                iteration: 2,
            },
        };
        let file = StateFile {
            state: state.clone(),
        };
        let toml_str = toml::to_string_pretty(&file).unwrap();
        let parsed: StateFile = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.state, state);
    }

    #[test]
    fn test_state_roundtrip_refine_methodology_complete() {
        let state = SpiralState::RefineMethodologyComplete { pass: 2 };
        let file = StateFile {
            state: state.clone(),
        };
        let toml_str = toml::to_string_pretty(&file).unwrap();
        let parsed: StateFile = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.state, state);
    }

    #[test]
    fn test_state_backward_compat_build_without_task_id() {
        // Simulates an old state.toml that only had iteration (no task_id)
        let toml_str = r#"
state = "InPass"
pass = 2

[phase]
phase = "Build"
iteration = 5
"#;
        let parsed: StateFile = toml::from_str(toml_str).unwrap();
        assert_eq!(
            parsed.state,
            SpiralState::InPass {
                pass: 2,
                phase: PassPhase::Build {
                    task_id: 0,
                    iteration: 5
                }
            }
        );
    }
}
