use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::agent::UsageInfo;
use crate::terminal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvocationRecord {
    pub phase: String,
    pub pass: u32,
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_input_tokens: u64,
    pub cache_read_input_tokens: u64,
    pub cost_usd: f64,
    pub elapsed_secs: u64,
    pub timestamp: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageLedger {
    #[serde(default)]
    pub invocations: Vec<InvocationRecord>,
}

impl UsageLedger {
    pub fn total_cost(&self) -> f64 {
        self.invocations.iter().map(|r| r.cost_usd).sum()
    }

    pub fn total_input_tokens(&self) -> u64 {
        self.invocations.iter().map(|r| r.input_tokens).sum()
    }

    pub fn total_output_tokens(&self) -> u64 {
        self.invocations.iter().map(|r| r.output_tokens).sum()
    }

    pub fn pass_cost(&self, pass: u32) -> f64 {
        self.invocations
            .iter()
            .filter(|r| r.pass == pass)
            .map(|r| r.cost_usd)
            .sum()
    }

    pub fn invocation_count(&self) -> usize {
        self.invocations.len()
    }
}

pub fn load_usage(lisa_root: &Path) -> Result<UsageLedger> {
    let path = lisa_root.join("usage.toml");
    if !path.exists() {
        return Ok(UsageLedger::default());
    }
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let ledger: UsageLedger =
        toml::from_str(&content).with_context(|| "Failed to parse usage.toml")?;
    Ok(ledger)
}

pub fn save_usage(lisa_root: &Path, ledger: &UsageLedger) -> Result<()> {
    let path = lisa_root.join("usage.toml");
    std::fs::create_dir_all(lisa_root)?;
    let content = toml::to_string_pretty(ledger).with_context(|| "Failed to serialize usage")?;
    std::fs::write(&path, content)
        .with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

/// Record an invocation and return cumulative cost.
pub fn record_invocation(
    lisa_root: &Path,
    phase: &str,
    pass: u32,
    model: &str,
    usage: &UsageInfo,
    elapsed_secs: u64,
) -> Result<f64> {
    let mut ledger = load_usage(lisa_root)?;
    ledger.invocations.push(InvocationRecord {
        phase: phase.to_string(),
        pass,
        model: model.to_string(),
        input_tokens: usage.input_tokens,
        output_tokens: usage.output_tokens,
        cache_creation_input_tokens: usage.cache_creation_input_tokens,
        cache_read_input_tokens: usage.cache_read_input_tokens,
        cost_usd: usage.cost_usd,
        elapsed_secs,
        timestamp: chrono::Local::now().to_rfc3339(),
    });
    save_usage(lisa_root, &ledger)?;
    Ok(ledger.total_cost())
}

/// Check budget. Bail if over budget_usd (when > 0). Warn if over warn threshold.
pub fn check_budget(cumulative_cost: f64, budget_usd: f64, budget_warn_pct: u32) -> Result<()> {
    if budget_usd <= 0.0 {
        return Ok(()); // unlimited
    }

    if cumulative_cost >= budget_usd {
        anyhow::bail!(
            "Budget exceeded: ${:.4} spent of ${:.2} limit. \
             Increase limits.budget_usd in lisa.toml or run `lisa resume` after adjusting.",
            cumulative_cost,
            budget_usd
        );
    }

    let warn_threshold = budget_usd * (budget_warn_pct as f64 / 100.0);
    if cumulative_cost >= warn_threshold {
        terminal::log_warn(&format!(
            "Budget warning: ${:.4} spent of ${:.2} limit ({}% threshold).",
            cumulative_cost, budget_usd, budget_warn_pct
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_ledger() -> UsageLedger {
        UsageLedger {
            invocations: vec![
                InvocationRecord {
                    phase: "scope".to_string(),
                    pass: 0,
                    model: "opus".to_string(),
                    input_tokens: 1000,
                    output_tokens: 500,
                    cache_creation_input_tokens: 0,
                    cache_read_input_tokens: 200,
                    cost_usd: 0.05,
                    elapsed_secs: 30,
                    timestamp: "2025-01-01T00:00:00+00:00".to_string(),
                },
                InvocationRecord {
                    phase: "build".to_string(),
                    pass: 1,
                    model: "sonnet".to_string(),
                    input_tokens: 2000,
                    output_tokens: 1000,
                    cache_creation_input_tokens: 100,
                    cache_read_input_tokens: 0,
                    cost_usd: 0.03,
                    elapsed_secs: 45,
                    timestamp: "2025-01-01T00:01:00+00:00".to_string(),
                },
            ],
        }
    }

    #[test]
    fn test_total_cost() {
        let ledger = sample_ledger();
        assert!((ledger.total_cost() - 0.08).abs() < 1e-10);
    }

    #[test]
    fn test_total_tokens() {
        let ledger = sample_ledger();
        assert_eq!(ledger.total_input_tokens(), 3000);
        assert_eq!(ledger.total_output_tokens(), 1500);
    }

    #[test]
    fn test_pass_cost() {
        let ledger = sample_ledger();
        assert!((ledger.pass_cost(0) - 0.05).abs() < 1e-10);
        assert!((ledger.pass_cost(1) - 0.03).abs() < 1e-10);
        assert!((ledger.pass_cost(99)).abs() < 1e-10);
    }

    #[test]
    fn test_empty_ledger() {
        let ledger = UsageLedger::default();
        assert!((ledger.total_cost()).abs() < 1e-10);
        assert_eq!(ledger.total_input_tokens(), 0);
        assert_eq!(ledger.total_output_tokens(), 0);
        assert_eq!(ledger.invocation_count(), 0);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let ledger = sample_ledger();
        let toml_str = toml::to_string_pretty(&ledger).unwrap();
        let parsed: UsageLedger = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.invocations.len(), 2);
        assert!((parsed.total_cost() - 0.08).abs() < 1e-10);
    }

    #[test]
    fn test_check_budget_unlimited() {
        assert!(check_budget(100.0, 0.0, 80).is_ok());
    }

    #[test]
    fn test_check_budget_exceeded() {
        assert!(check_budget(1.5, 1.0, 80).is_err());
    }

    #[test]
    fn test_check_budget_under() {
        assert!(check_budget(0.5, 1.0, 80).is_ok());
    }
}
