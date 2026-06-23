use std::path::Path;

use anyhow::{Context as AnyhowContext, bail};

use crate::commands::sync::cmd_sync;
use crate::context::Context;
use crate::nickel::generated;
use crate::output::log;
use crate::sync::{SyncMode, TerminalPrompter};

/// Init command: write or refresh `orders/order.contract.ncl` and
/// `orders/metadata.ncl`, then bootstrap a starter blend config order for new
/// blend directories. Honors `--dry-run` by checking freshness (read-only)
/// instead of writing.
///
/// When `upgrade` is true, breaking contract migrations are allowed.
pub fn cmd_init(ctx: &Context, upgrade: bool) -> anyhow::Result<()> {
    if ctx.dry_run {
        return generated::assert_orders_ready(&ctx.orders_dir);
    }

    let should_create_starter = !starter_path(ctx).exists();
    if should_create_starter {
        ensure_scaffold_target_clean(ctx)?;
    }

    generated::ensure_orders_ready(&ctx.orders_dir, upgrade)?;

    if should_create_starter {
        write_blend_starter(ctx)?;
        let orders = vec!["blend".to_string()];
        cmd_sync(
            ctx,
            &orders,
            SyncMode::Interactive,
            false,
            &TerminalPrompter,
        )?;
    }

    Ok(())
}

fn starter_path(ctx: &Context) -> std::path::PathBuf {
    ctx.orders_dir.join("blend").join("order.ncl")
}

fn ensure_scaffold_target_clean(ctx: &Context) -> anyhow::Result<()> {
    if !ctx.blend_dir.exists() {
        return Ok(());
    }

    let entries = std::fs::read_dir(&ctx.blend_dir)
        .with_context(|| format!("Failed to read {}", ctx.blend_dir.display()))?;

    let mut dirty = Vec::new();
    for entry in entries {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let path = entry.path();

        match name.as_ref() {
            ".git" => {}
            "orders" if is_clean_initial_orders_dir(&path)? => {}
            _ => dirty.push(name.into_owned()),
        }
    }

    if !dirty.is_empty() {
        dirty.sort();
        bail!(
            "{} is not clean enough for `blend init`: found {}. Run init in an empty directory or pass --blend-dir to a clean path.",
            ctx.blend_dir.display(),
            dirty.join(", ")
        );
    }

    Ok(())
}

fn is_clean_initial_orders_dir(path: &Path) -> anyhow::Result<bool> {
    if !path.is_dir() {
        return Ok(false);
    }

    for entry in
        std::fs::read_dir(path).with_context(|| format!("Failed to read {}", path.display()))?
    {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let entry_path = entry.path();

        match name.as_ref() {
            "order.contract.ncl" | "metadata.ncl" => {}
            "blend" if is_empty_dir(&entry_path)? => {}
            _ => return Ok(false),
        }
    }

    Ok(true)
}

fn is_empty_dir(path: &Path) -> anyhow::Result<bool> {
    if !path.is_dir() {
        return Ok(false);
    }

    Ok(std::fs::read_dir(path)
        .with_context(|| format!("Failed to read {}", path.display()))?
        .next()
        .transpose()?
        .is_none())
}

fn write_blend_starter(ctx: &Context) -> anyhow::Result<()> {
    let order_dir = ctx.orders_dir.join("blend");
    std::fs::create_dir_all(&order_dir)
        .with_context(|| format!("Failed to create {}", order_dir.display()))?;

    let path = starter_path(ctx);
    std::fs::write(&path, starter_ncl())
        .with_context(|| format!("Failed to write {}", path.display()))?;
    log::info(&format!("created {}", path.display()));
    Ok(())
}

fn starter_ncl() -> String {
    r#"let { Order, BlendOrder, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/blend/"],
    files = [
      {
        name = "config.toml",
        from_config = {
          sandbox = "prefer",
        },
      },
    ],
  },
} | Order | BlendOrder
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use clap::Parser;
    use tempfile::TempDir;

    fn ctx_for(orders: &std::path::Path) -> Context {
        let cli = Cli::parse_from([
            "blend",
            "--blend-dir",
            orders.to_str().unwrap(),
            "--home",
            orders.to_str().unwrap(), // home doesn't matter for init
        ]);
        Context::new(&cli).unwrap()
    }

    #[test]
    fn cmd_init_creates_files() {
        let tmp = TempDir::new().unwrap();
        let ctx = ctx_for(tmp.path());
        cmd_init(&ctx, false).unwrap();
        assert!(tmp.path().join("orders/order.contract.ncl").exists());
        assert!(tmp.path().join("orders/metadata.ncl").exists());
        assert!(tmp.path().join("orders/blend/order.ncl").exists());
        assert!(tmp.path().join(".config/blend/config.toml").exists());
    }

    #[test]
    fn cmd_init_is_idempotent() {
        let tmp = TempDir::new().unwrap();
        let ctx = ctx_for(tmp.path());
        cmd_init(&ctx, false).unwrap();
        cmd_init(&ctx, false).unwrap(); // second call must not error
    }

    #[test]
    fn cmd_init_refuses_dirty_new_scaffold_dir() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("README.md"), "already mine\n").unwrap();
        let ctx = ctx_for(tmp.path());

        let err = cmd_init(&ctx, false).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("not clean enough"), "got: {msg}");
        assert!(!tmp.path().join("orders/order.contract.ncl").exists());
    }

    #[test]
    fn cmd_init_does_not_overwrite_existing_blend_order() {
        let tmp = TempDir::new().unwrap();
        let order_dir = tmp.path().join("orders/blend");
        std::fs::create_dir_all(&order_dir).unwrap();
        let order = order_dir.join("order.ncl");
        std::fs::write(&order, "{ blend = { files = [] } }\n").unwrap();

        let ctx = ctx_for(tmp.path());
        cmd_init(&ctx, false).unwrap();

        assert_eq!(
            std::fs::read_to_string(order).unwrap(),
            "{ blend = { files = [] } }\n"
        );
        assert!(!tmp.path().join(".config/blend/config.toml").exists());
    }
}
