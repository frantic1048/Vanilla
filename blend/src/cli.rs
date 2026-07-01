use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::sandbox::SandboxMode;

const LONG_ABOUT: &str = "\
Cross-platform dotfiles manager with Nickel DSL

Tags: [read] writes nothing; [source] writes Blend Source files; [target] writes deployed files or Blend runtime state.";

const HELP_TEMPLATE: &str = "\
{about}

{usage-heading} {usage}{after-help}
Options:
{options}";

const COMMAND_HELP: &str = "\
Inspect:
  status  [read] Show order deployment status (default)
  view    [read] Preview generated config and diff from Target files
  table   [read] Output order info as HTML table

Maintain:
  check   [read] Validate Source order definitions
  format  [source] Format Source order files
  init    [source, target] Initialize or refresh Blend metadata and config
  sync    [source, target] Reconcile Source orders and Target files

Other:
  help    Print this message or the help of the given subcommand(s)
";

#[derive(Parser)]
#[command(
    name = "blend",
    version,
    about = "Cross-platform dotfiles manager with Nickel DSL",
    long_about = LONG_ABOUT,
    help_template = HELP_TEMPLATE,
    after_help = COMMAND_HELP
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Preview mutating commands without writing files
    #[arg(short = 'n', long, global = true)]
    pub dry_run: bool,

    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Override Target home for ~ expansion and metadata.home (default: $HOME)
    #[arg(long, global = true)]
    pub home: Option<PathBuf>,

    /// Override Blend Source root; default: nearest ancestor with orders/, then remembered state
    #[arg(long = "blend-dir", global = true)]
    pub blend_dir: Option<PathBuf>,

    /// Process sandbox policy
    #[arg(long, value_enum, global = true)]
    pub sandbox: Option<SandboxMode>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(flatten)]
    Inspect(InspectCommands),

    #[command(flatten)]
    Maintain(MaintainCommands),
}

#[derive(Subcommand)]
pub enum InspectCommands {
    /// [read] Show order deployment status (default)
    Status,

    /// [read] Preview generated config and diff from Target files
    View {
        /// Orders to view (default: all)
        orders: Vec<String>,

        /// Only show generated content (no diff)
        #[arg(short = 'c', long, conflicts_with = "all")]
        content_only: bool,

        /// Show both generated content and diff
        #[arg(short = 'a', long)]
        all: bool,

        /// Omit up-to-date files from output (only show files with changes)
        #[arg(short = 's', long)]
        short: bool,
    },

    /// [read] Output order info as HTML table
    Table,
}

#[derive(Subcommand)]
pub enum MaintainCommands {
    /// [source, target] Reconcile Source orders and Target files
    #[command(alias = "s")]
    Sync {
        /// Orders to sync (default: all)
        orders: Vec<String>,

        /// Force-resolve: apply all Source values to Targets without prompting
        #[arg(
            long = "force-source-to-target",
            conflicts_with = "force_target_to_source"
        )]
        force_source_to_target: bool,

        /// Force-resolve: apply all Target values back to Source without prompting
        #[arg(long = "force-target-to-source")]
        force_target_to_source: bool,

        /// Disable .ncl rewrite when applying Target to Source; show diff for manual merge
        #[arg(long)]
        no_rewrite: bool,
    },

    /// [read] Validate Source order definitions
    Check {
        /// Orders to check (default: all)
        orders: Vec<String>,
    },

    /// [source] Format Source order files
    #[command(alias = "fmt")]
    Format {
        /// Orders to format (default: all)
        orders: Vec<String>,

        /// Check formatting without writing changes
        #[arg(long)]
        check: bool,
    },

    /// [source, target] Initialize or refresh Blend metadata and config
    #[command(long_about = "\
[source, target] Initialize or refresh Blend metadata and config

Writes or refreshes orders/order.contract.ncl and orders/metadata.ncl. For a new Source root, also creates a starter blend order and deploys its config to Target. If no Source root is found, init bootstraps the current directory.")]
    Init {
        /// Apply breaking contract migrations (required when upgrading across breaking versions)
        #[arg(long)]
        upgrade: bool,
    },
}
