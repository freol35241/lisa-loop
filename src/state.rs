use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "state")]
pub enum SpiralState {
    NotStarted,
    Scoping { attempt: u32 },
    ScopeReview,
    ScopeComplete,
    InPass { pass: u32, phase: PassPhase },
    PassReview { pass: u32 },
    Complete { final_pass: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "phase")]
pub enum PassPhase {
    Refine,
    DdvRed,
    Build { iteration: u32 },
    Execute,
    Validate,
}

impl std::fmt::Display for SpiralState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpiralState::NotStarted => write!(f, "Not started"),
            SpiralState::Scoping { attempt } => write!(f, "Scoping (attempt {})", attempt),
            SpiralState::ScopeReview => write!(f, "Scope review"),
            SpiralState::ScopeComplete => write!(f, "Scope complete"),
            SpiralState::InPass { pass, phase } => write!(f, "Pass {} — {}", pass, phase),
            SpiralState::PassReview { pass } => write!(f, "Pass {} — Review", pass),
            SpiralState::Complete { final_pass } => write!(f, "Complete (pass {})", final_pass),
        }
    }
}

impl std::fmt::Display for PassPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PassPhase::Refine => write!(f, "Refine"),
            PassPhase::DdvRed => write!(f, "DDV Red"),
            PassPhase::Build { iteration } => write!(f, "Build (iteration {})", iteration),
            PassPhase::Execute => write!(f, "Execute"),
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
        let state = SpiralState::Scoping { attempt: 2 };
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
    fn test_state_display() {
        assert_eq!(format!("{}", SpiralState::NotStarted), "Not started");
        assert_eq!(
            format!(
                "{}",
                SpiralState::InPass {
                    pass: 2,
                    phase: PassPhase::DdvRed
                }
            ),
            "Pass 2 — DDV Red"
        );
    }
}
