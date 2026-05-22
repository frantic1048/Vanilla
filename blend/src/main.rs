mod cli;
mod commands;
mod compose;
mod context;
mod diff;
mod formats;
mod metadata;
mod nickel;
mod output;
mod state;
mod sync;

use clap::Parser;

use cli::{Cli, Commands};
use commands::{cmd_init, cmd_status, cmd_sync, cmd_table, cmd_view};
use context::Context;
use output::log;
use sync::{SyncMode, TerminalPrompter};

fn main() {
    let cli = Cli::parse();
    let ctx = match Context::new(&cli) {
        Ok(ctx) => ctx,
        Err(e) => {
            log::error(&format!("Error: {e}"));
            std::process::exit(1);
        }
    };

    if ctx.verbose {
        log::info(&format!("Home directory: {}", ctx.home_dir.display()));
        log::info(&format!("Blend directory: {}", ctx.blend_dir.display()));
        log::info(&format!("Orders directory: {}", ctx.orders_dir.display()));
        log::info(&format!(
            "OS: {}, Arch: {}",
            ctx.metadata.os, ctx.metadata.arch
        ));
    }

    let result = match cli.command {
        Some(Commands::Sync {
            orders,
            force_source_to_target,
            force_target_to_source,
            no_rewrite,
        }) => {
            let mode = if force_source_to_target {
                SyncMode::ApplySourceToTargetAll
            } else if force_target_to_source {
                SyncMode::ApplyTargetToSourceAll
            } else {
                SyncMode::Interactive
            };
            cmd_sync(&ctx, &orders, mode, no_rewrite, &TerminalPrompter)
        }
        Some(Commands::View {
            orders,
            content_only,
            all,
            short,
        }) => cmd_view(&ctx, &orders, content_only, all, short),
        Some(Commands::Table) => cmd_table(&ctx),
        Some(Commands::Init) => cmd_init(&ctx),
        None => cmd_status(&ctx),
    };

    if let Err(e) = result {
        log::error(&format!("Error: {e}"));
        std::process::exit(1);
    }

    if let Err(e) = ctx.update_config_after_success() {
        log::error(&format!("Error: {e}"));
        std::process::exit(1);
    }
}
