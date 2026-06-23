mod cli;
mod commands;
mod compose;
mod context;
mod diff;
mod formats;
mod immutable;
mod metadata;
mod migration;
mod nickel;
mod output;
mod sandbox;
mod state;
mod sync;

use clap::Parser;

use cli::{Cli, Commands};
use commands::{cmd_check, cmd_format, cmd_init, cmd_status, cmd_sync, cmd_table, cmd_view};
use context::{Context, sandbox_mode_from_cli_and_config};
use output::log;
use sandbox::SandboxMode;
use sync::{SyncMode, TerminalPrompter};

fn main() {
    let cli = Cli::parse();
    let sandbox_mode = match sandbox_mode_from_cli_and_config(&cli) {
        Ok(mode) => mode,
        Err(e) => {
            log::error(&format!("Error: {e}"));
            std::process::exit(1);
        }
    };
    let sandbox_installed = match sandbox_mode {
        SandboxMode::Force | SandboxMode::Prefer => match sandbox::install() {
            Ok(()) if cli.verbose => {
                log::info("Enabled process sandbox");
                true
            }
            Ok(()) => true,
            Err(e) if sandbox_mode == SandboxMode::Force => {
                log::error(&format!("Error: failed to enable process sandbox: {e}"));
                std::process::exit(1);
            }
            Err(e) => {
                log::warn(&format!("failed to enable process sandbox: {e}"));
                false
            }
        },
        SandboxMode::Never => {
            if cli.verbose {
                log::info("Process sandbox disabled");
            }
            false
        }
    };

    #[cfg(not(debug_assertions))]
    let _ = sandbox_installed;

    #[cfg(debug_assertions)]
    if sandbox_installed && let Err(e) = sandbox::run_probe_from_env() {
        log::error(&format!("Error: sandbox probe failed: {e}"));
        std::process::exit(1);
    }

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
        Some(Commands::Check { orders }) => cmd_check(&ctx, &orders),
        Some(Commands::Format { orders, check }) => cmd_format(&ctx, &orders, check),
        Some(Commands::Table) => cmd_table(&ctx),
        Some(Commands::Init { upgrade }) => cmd_init(&ctx, upgrade),
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
