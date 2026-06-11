use console::style;

use crate::compose::discover_orders;
use crate::context::Context;
use crate::nickel::{NickelEvaluator, Order, generated};
use crate::output::log;

fn selected_orders(ctx: &Context, orders: &[String]) -> Vec<String> {
    let mut selected: Vec<String> = if orders.is_empty() {
        discover_orders(&ctx.orders_dir).into_iter().collect()
    } else {
        orders.to_vec()
    };
    selected.sort();
    selected
}

fn validate_order_semantics(ctx: &Context, order_name: &str, order: &Order) -> anyhow::Result<()> {
    if !order.should_apply(&ctx.metadata.os, &ctx.metadata.arch, &ctx.metadata.hostname) {
        return Ok(());
    }

    let order_dir = ctx.orders_dir.join(order_name);

    for file_entry in &order.blend.files {
        if !file_entry.should_apply(&ctx.metadata.os, &ctx.metadata.arch, &ctx.metadata.hostname) {
            continue;
        }

        let Some(from_file) = &file_entry.from_file else {
            continue;
        };

        let source_path = order_dir.join(from_file);
        if !source_path.exists() {
            anyhow::bail!(
                "File entry '{}': source file not found at {}",
                file_entry.name,
                source_path.display()
            );
        }
    }

    Ok(())
}

/// Check command: typecheck and evaluate order.ncl files through Nickel.
pub fn cmd_check(ctx: &Context, orders: &[String]) -> anyhow::Result<()> {
    generated::assert_orders_ready(&ctx.orders_dir)?;

    let evaluator = NickelEvaluator::new(&ctx.metadata);
    let selected = selected_orders(ctx, orders);
    let checking_specific = !orders.is_empty();
    let mut checked = 0usize;
    let mut failed = false;

    for order_name in &selected {
        let ncl_path = ctx.orders_dir.join(order_name).join("order.ncl");
        if !ncl_path.exists() {
            log::error(&format!("Order '{order_name}' not found"));
            failed = true;
            continue;
        }

        match evaluator.evaluate(&ncl_path).and_then(|order| {
            validate_order_semantics(ctx, order_name, &order)?;
            Ok(order)
        }) {
            Ok(_) => {
                checked += 1;
                if ctx.verbose || checking_specific {
                    println!("{} {}", style("\u{2713}").green(), order_name);
                }
            }
            Err(err) => {
                failed = true;
                log::error(&format!("{order_name}: {err:#}"));
            }
        }
    }

    if failed {
        anyhow::bail!("Nickel check failed");
    }

    log::success(&format!("Checked {checked} order(s)"));
    Ok(())
}
