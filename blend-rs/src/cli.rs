use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "blend", version, about = "Cross-platform dotfiles manager with Nickel DSL")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Show what would be done without making changes
    #[arg(short = 'n', long, global = true)]
    pub dry_run: bool,

    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Override home directory (for testing)
    #[arg(long, global = true)]
    pub home: Option<PathBuf>,

    /// Override orders directory (default: ../orders relative to blend-rs)
    #[arg(long, global = true)]
    pub orders: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate and deploy configs to target locations
    Ship {
        /// Packages to ship (default: all)
        packages: Vec<String>,
    },

    /// Preview generated config and diff from deployed
    View {
        /// Packages to view (default: all)
        packages: Vec<String>,

        /// Only show generated content (no diff)
        #[arg(short = 'c', long)]
        content_only: bool,

        /// Show both generated content and diff
        #[arg(short = 'a', long)]
        all: bool,
    },

    /// Capture deployed config as reference (reverse of ship)
    Sample {
        /// Packages to sample (default: all)
        packages: Vec<String>,
    },

    /// Output package info as HTML table (for README generation)
    Table,

    /// System upgrade: update packages, tools, and dotfiles
    #[command(alias = "s")]
    Upgrade {
        #[command(subcommand)]
        step: Option<UpgradeStep>,
    },
}

#[derive(Subcommand)]
pub enum UpgradeStep {
    /// Update Homebrew packages (macOS only)
    Homebrew,
    /// Update system packages via paru (Linux/Arch only)
    Pacman,
    /// Update Proto toolchain versions
    Proto,
}
