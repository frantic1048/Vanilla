use crate::compose::{discover_orders, get_order};
use crate::context::Context;
use crate::nickel::generated;
use crate::output::log;

/// Table command: output order info as HTML table for README
pub fn cmd_table(ctx: &Context) -> anyhow::Result<()> {
    generated::assert_orders_ready(&ctx.orders_dir)?;

    let orders = discover_orders(&ctx.orders_dir);

    let profiles: &[(&str, &str, &str)] = &[
        ("linux", "x86_64", "linux-x86_64"),
        ("darwin", "x86_64", "macos-x86_64"),
        ("darwin", "aarch64", "macos-aarch64"),
    ];

    let mut order_data: Vec<(String, Vec<bool>, usize)> = Vec::new();

    for order_name in &orders {
        match get_order(ctx, order_name) {
            Ok(order) => {
                let matches: Vec<bool> = profiles
                    .iter()
                    .map(|(os, arch, _)| order.applies_on_platform(os, arch))
                    .collect();
                let match_count = matches.iter().filter(|&&m| m).count();
                order_data.push((order_name.clone(), matches, match_count));
            }
            Err(e) => {
                log::warn(&format!("Skipping {order_name} (eval error: {e})"));
            }
        }
    }

    order_data.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.cmp(&b.0)));

    print!("<table><thead><tr><th>order</th><th colspan=\"3\">profiles</th></tr></thead><tbody>");
    for (name, matches, _) in &order_data {
        print!("\n<tr><td><a href=\"orders/{name}\">{name}</a></td>");
        for (i, (_os, _arch, label)) in profiles.iter().enumerate() {
            if matches[i] {
                print!("<td><code>{label}</code></td>");
            } else {
                print!("<td><code>&nbsp;...</code></td>");
            }
        }
        print!("</tr>");
    }
    println!("\n</tbody></table>");

    Ok(())
}
