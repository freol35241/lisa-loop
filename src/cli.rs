use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "lisa",
    about = "Lisa Loop — Rigorous engineering problem-solving with AI agents",
    after_long_help = "\
WORKFLOW:

  lisa init → scaffold .lisa/ + ASSIGNMENT.md
                    │
                    ▼
  ┌───────────────────────────────────────────────┐
  │  SCOPE (Pass 0)                               │
  │  Define methods, acceptance criteria, staged   │
  │  plan — no code yet                           │◄── refine loop
  └──────────────────┬────────────────────────────┘
                     ▼
  ┌───────────────────────────────────────────────┐
  │  INDEPENDENT VERIFICATION (one-time prologue) │
  │  Test criteria from literature, before code    │
  └──────────────────┬────────────────────────────┘
                     ▼
  ┌───────────────────────────────────────────────┐
  │  SPIRAL PASSES (Pass 1..N)                    │
  │  Refine → Build → Validate                    │
  │  Each pass widens scope & tightens tolerances │◄── human gate
  └──────────────────┬────────────────────────────┘
                     ▼
  lisa finalize → answer.md + report.md
"
)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new Lisa Loop project
    Init {
        /// Assignment name (defaults to directory name)
        #[arg(long)]
        name: Option<String>,
        /// Technology preference (e.g., "Python 3.11+ with NumPy/SciPy")
        #[arg(long)]
        tech: Option<String>,
    },
    /// Run the full spiral (scope if needed, then iterate)
    Run {
        /// Maximum number of spiral passes
        #[arg(long)]
        max_passes: Option<u32>,
        /// Skip all human review gates
        #[arg(long)]
        no_pause: bool,
        /// Show full agent output (overrides collapse_output config)
        #[arg(long, short)]
        verbose: bool,
    },
    /// Resume from saved state
    Resume {
        /// Skip all human review gates
        #[arg(long)]
        no_pause: bool,
        /// Show full agent output (overrides collapse_output config)
        #[arg(long, short)]
        verbose: bool,
    },
    /// Run only Pass 0 (scoping)
    Scope,
    /// Run the DDV Agent (write or extend domain verification scenarios)
    Ddv,
    /// Print current spiral state
    Status,
    /// Check environment and prerequisites
    Doctor,
    /// Produce final deliverables
    Finalize,
    /// Copy compiled-in prompts to .lisa/prompts/ for customization
    EjectPrompts,
    /// Show pass-by-pass history (answer, tests, recommendation)
    History,
    /// Roll back to a previous pass boundary
    Rollback {
        /// Pass number to roll back to (e.g., 1 for end of pass 1)
        pass: u32,
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
    /// Continue with a follow-up question after a completed spiral
    Continue {
        /// The follow-up question or task
        question: String,
        /// Maximum number of additional spiral passes
        #[arg(long)]
        max_passes: Option<u32>,
        /// Skip all human review gates
        #[arg(long)]
        no_pause: bool,
        /// Show full agent output (overrides collapse_output config)
        #[arg(long, short)]
        verbose: bool,
    },
}
