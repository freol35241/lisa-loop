use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "lisa",
    about = "Lisa Loop — Rigorous engineering problem-solving with AI agents"
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
    Resume,
    /// Run only Pass 0 (scoping)
    Scope,
    /// Print current spiral state
    Status,
    /// Check environment and prerequisites
    Doctor,
    /// Produce final deliverables
    Finalize,
    /// Copy compiled-in prompts to .lisa/prompts/ for customization
    EjectPrompts,
}
