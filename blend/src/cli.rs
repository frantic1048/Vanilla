use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::sandbox::SandboxMode;

#[derive(Parser)]
#[command(
    name = "blend",
    version,
    about = "Cross-platform dotfiles manager with Nickel DSL"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Show what would be done without making changes
    #[arg(short = 'n', long, global = true)]
    pub dry_run: bool,

    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Override Target home for ~ expansion and metadata.home (default: $HOME)
    #[arg(long, global = true)]
    pub home: Option<PathBuf>,

    /// Override blend directory (the parent directory that contains orders/)
    #[arg(long = "blend-dir", global = true)]
    pub blend_dir: Option<PathBuf>,

    /// Process sandbox policy
    #[arg(long, value_enum, global = true)]
    pub sandbox: Option<SandboxMode>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Bidirectional sync between Source (orders) and Target (deployed files)
    #[command(alias = "s")]
    Sync {
        /// Orders to sync (default: all)
        orders: Vec<String>,

        /// Force-resolve: apply all Source values to Targets without prompting
        #[arg(long = "force-source-to-target")]
        force_source_to_target: bool,

        /// Force-resolve: apply all Target values back to Source without prompting
        #[arg(long = "force-target-to-source")]
        force_target_to_source: bool,

        /// Disable .ncl rewrite when applying Target to Source; show diff for manual merge
        #[arg(long)]
        no_rewrite: bool,
    },

    /// Preview generated config and diff from deployed
    View {
        /// Orders to view (default: all)
        orders: Vec<String>,

        /// Only show generated content (no diff)
        #[arg(short = 'c', long)]
        content_only: bool,

        /// Show both generated content and diff
        #[arg(short = 'a', long)]
        all: bool,

        /// Omit up-to-date files from output (only show files with changes)
        #[arg(short = 's', long)]
        short: bool,
    },

    /// Typecheck order.ncl files with Nickel
    Check {
        /// Orders to check (default: all)
        orders: Vec<String>,
    },

    /// Format order.ncl files with Nickel
    #[command(alias = "fmt")]
    Format {
        /// Orders to format (default: all)
        orders: Vec<String>,

        /// Check formatting without writing changes
        #[arg(long)]
        check: bool,
    },

    /// Output order info as HTML table (for README generation)
    Table,

    /// Generate or refresh orders/order.contract.ncl and orders/metadata.ncl
    Init,
}
