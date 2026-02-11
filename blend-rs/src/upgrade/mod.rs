use std::path::Path;
use std::process::Command;

use crate::cli::UpgradeStep;
use crate::context::Context;
use crate::output::log;

/// Run an external command, respecting dry-run and verbose flags.
/// Returns Ok(()) on success, Err on non-zero exit or spawn failure.
fn run_cmd(ctx: &Context, program: &str, args: &[&str], working_dir: &Path) -> anyhow::Result<()> {
    let cmd_str = format!("{} {}", program, args.join(" "));

    if ctx.dry_run {
        log::info(&format!("[dry-run] {}", cmd_str));
        return Ok(());
    }

    if ctx.verbose {
        log::info(&format!("$ {}", cmd_str));
    }

    let status = Command::new(program)
        .args(args)
        .current_dir(working_dir)
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run '{}': {}", program, e))?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "'{}' exited with status {}",
            cmd_str,
            status.code().unwrap_or(-1)
        ))
    }
}

/// Capture stdout of a command. Returns Ok(output) or Err on failure.
fn run_cmd_capture(
    ctx: &Context,
    program: &str,
    args: &[&str],
    working_dir: &Path,
) -> anyhow::Result<String> {
    let cmd_str = format!("{} {}", program, args.join(" "));

    if ctx.dry_run {
        log::info(&format!("[dry-run] {}", cmd_str));
        return Ok(String::new());
    }

    if ctx.verbose {
        log::info(&format!("$ {}", cmd_str));
    }

    let output = Command::new(program)
        .args(args)
        .current_dir(working_dir)
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to run '{}': {}", program, e))?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn upgrade_homebrew(ctx: &Context) -> anyhow::Result<()> {
    if ctx.metadata.os != "darwin" {
        if ctx.verbose {
            log::info("Skipping Homebrew (not macOS)");
        }
        return Ok(());
    }

    log::heading_info("Homebrew");
    let root = ctx.repo_root();
    run_cmd(ctx, "brew", &["update"], &root)?;
    run_cmd(ctx, "brew", &["bundle", "install", "--upgrade"], &root)?;
    Ok(())
}

fn upgrade_pacman(ctx: &Context) -> anyhow::Result<()> {
    if ctx.metadata.os != "linux" {
        if ctx.verbose {
            log::info("Skipping Pacman (not Linux)");
        }
        return Ok(());
    }

    log::heading_info("Pacman");
    let root = ctx.repo_root();
    run_cmd(
        ctx,
        "paru",
        &["-Sy", "archlinux-keyring", "archlinuxcn-keyring"],
        &root,
    )?;
    run_cmd(ctx, "paru", &["-Syuw"], &root)?;
    run_cmd(ctx, "paru", &["-Syu"], &root)?;

    // Remove orphan packages if any exist
    let orphans = run_cmd_capture(ctx, "paru", &["-Qdtq"], &root)?;
    if !orphans.is_empty() {
        let orphan_list: Vec<&str> = orphans.lines().collect();
        let mut args = vec!["-Rns"];
        args.extend(orphan_list.iter());
        run_cmd(ctx, "paru", &args, &root)?;
    } else if ctx.verbose {
        log::info("No orphan packages to remove");
    }

    Ok(())
}

fn upgrade_proto(ctx: &Context) -> anyhow::Result<()> {
    log::heading_info("Proto");
    let root = ctx.repo_root();
    run_cmd(ctx, "proto", &["clean", "--yes", "--days", "60"], &root)?;
    run_cmd(ctx, "proto", &["upgrade", "--yes"], &root)?;

    let proto_dir = ctx.orders_dir.join("proto").join(".proto");
    run_cmd(
        ctx,
        "proto",
        &["outdated", "--yes", "--update"],
        &proto_dir,
    )?;
    run_cmd(ctx, "proto", &["install", "--yes"], &proto_dir)?;
    Ok(())
}

fn upgrade_claude(ctx: &Context) -> anyhow::Result<()> {
    log::heading_info("Claude");
    let root = ctx.repo_root();
    run_cmd(ctx, "claude", &["update"], &root)?;
    Ok(())
}

/// Run system upgrade steps.
///
/// `ship_fn` is called at the end of a full upgrade to deploy dotfiles,
/// avoiding circular module dependency.
pub fn cmd_upgrade<F>(ctx: &Context, step: &Option<UpgradeStep>, ship_fn: F) -> anyhow::Result<()>
where
    F: FnOnce(&Context) -> anyhow::Result<()>,
{
    // Individual step: fail immediately
    if let Some(step) = step {
        return match step {
            UpgradeStep::Homebrew => upgrade_homebrew(ctx),
            UpgradeStep::Pacman => upgrade_pacman(ctx),
            UpgradeStep::Proto => upgrade_proto(ctx),
        };
    }

    // Full upgrade: continue on error, summarize at end
    log::heading_note("Starting system upgrade...");

    let steps: Vec<(&str, fn(&Context) -> anyhow::Result<()>)> = vec![
        ("Homebrew", upgrade_homebrew),
        ("Pacman", upgrade_pacman),
        ("Proto", upgrade_proto),
        ("Claude", upgrade_claude),
    ];

    let mut errors: Vec<String> = Vec::new();

    for (name, step_fn) in &steps {
        if let Err(e) = step_fn(ctx) {
            log::error(&format!("{} upgrade failed: {}", name, e));
            errors.push(format!("{}: {}", name, e));
        }
    }

    // Deploy dotfiles
    log::heading_info("Dotfiles");
    if let Err(e) = ship_fn(ctx) {
        log::error(&format!("Dotfile deployment failed: {}", e));
        errors.push(format!("Dotfiles: {}", e));
    }

    if errors.is_empty() {
        log::heading_note("System upgrade completed.");
    } else {
        log::heading_note("System upgrade completed with errors.");
        for err in &errors {
            log::error(&format!("  • {}", err));
        }
    }

    Ok(())
}
