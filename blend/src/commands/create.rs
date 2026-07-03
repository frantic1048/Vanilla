use anyhow::{Context as AnyhowContext, bail};

use crate::context::Context;
use crate::nickel::{format_source, generated};
use crate::output::log;

/// Create command: scaffold a new empty order under orders/<order>.
pub fn cmd_create(ctx: &Context, order: &str) -> anyhow::Result<()> {
    generated::assert_orders_ready(&ctx.orders_dir)?;
    validate_order_name(order)?;

    let order_dir = ctx.orders_dir.join(order);
    let order_path = order_dir.join("order.ncl");
    if order_dir.exists() || order_path.exists() {
        bail!("Order '{order}' already exists");
    }

    let source = format_source(&starter_ncl())?;

    if ctx.dry_run {
        log::info(&format!("Dry run: would create {}", order_path.display()));
        println!("{source}");
        return Ok(());
    }

    std::fs::create_dir_all(&order_dir)
        .with_context(|| format!("Failed to create {}", order_dir.display()))?;
    std::fs::write(&order_path, source)
        .with_context(|| format!("Failed to write {}", order_path.display()))?;
    log::success(&format!("Created order '{order}'"));
    Ok(())
}

pub fn validate_order_name(order: &str) -> anyhow::Result<()> {
    if order.is_empty()
        || order == "."
        || order == ".."
        || order.contains('/')
        || order.contains('\\')
    {
        bail!("Invalid order name '{order}'");
    }
    Ok(())
}

fn starter_ncl() -> String {
    r#"let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    files = [],
  },
} | Order
"#
    .to_string()
}
