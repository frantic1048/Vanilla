use console::style;

use crate::commands::helpers::select_orders;
use crate::context::Context;
use crate::nickel::{format_source, generated};
use crate::output::log;

/// Format command: format order.ncl files with Nickel's in-process formatter.
pub fn cmd_format(ctx: &Context, orders: &[String], check: bool) -> anyhow::Result<()> {
    generated::assert_orders_ready(&ctx.orders_dir)?;

    let selected = select_orders(ctx, orders);
    let mut visited = 0usize;
    let mut changed = Vec::new();
    let mut failed = false;

    for order_name in &selected {
        let ncl_path = ctx.orders_dir.join(order_name).join("order.ncl");
        if !ncl_path.exists() {
            log::error(&format!("Order '{order_name}' not found"));
            failed = true;
            continue;
        }

        visited += 1;
        let source = std::fs::read_to_string(&ncl_path)
            .map_err(|e| anyhow::anyhow!("Failed to read {}: {e}", ncl_path.display()))?;
        let formatted = format_source(&source)
            .map_err(|e| anyhow::anyhow!("Failed to format {}: {e:#}", ncl_path.display()))?;

        if formatted == source {
            if ctx.verbose {
                println!("{} {}", style("\u{2713}").green(), ncl_path.display());
            }
            continue;
        }

        changed.push(ncl_path.clone());
        let action = if check || ctx.dry_run {
            "would format"
        } else {
            "formatting"
        };
        println!("{} {}", style(action).yellow(), ncl_path.display());

        if !check && !ctx.dry_run {
            std::fs::write(&ncl_path, formatted)
                .map_err(|e| anyhow::anyhow!("Failed to write {}: {e}", ncl_path.display()))?;
        }
    }

    if failed {
        anyhow::bail!("Nickel format failed");
    }

    if check && !changed.is_empty() {
        anyhow::bail!("{} order.ncl file(s) need formatting", changed.len());
    }

    if ctx.dry_run && !changed.is_empty() {
        log::info("Dry run: no files were written");
    } else if !check {
        log::success(&format!("Formatted {} order.ncl file(s)", changed.len()));
    }

    if check && changed.is_empty() {
        log::success(&format!(
            "Checked formatting for {visited} order.ncl file(s)"
        ));
    }

    Ok(())
}
