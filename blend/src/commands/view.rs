use anyhow::Context as _;
use console::style;

use crate::commands::helpers::{
    compute_diff_for_result, compute_dir_file_diffs, target_type_mismatch,
};
use crate::compose::{build_order, discover_orders};
use crate::context::Context;
use crate::fs_node::{NodeKind, node_kind};
use crate::nickel::generated;
use crate::output::log;

/// View command: show generated config and/or diff from deployed
pub fn cmd_view(
    ctx: &Context,
    orders: &[String],
    content_only: bool,
    show_all: bool,
    short: bool,
) -> anyhow::Result<()> {
    generated::assert_orders_ready(&ctx.orders_dir)?;

    let all_orders = discover_orders(&ctx.orders_dir);
    let viewing_specific = !orders.is_empty();

    let to_view: Vec<String> = if orders.is_empty() {
        all_orders.into_iter().collect()
    } else {
        orders.to_vec()
    };

    let show_content = content_only || show_all;
    let show_diff = !content_only;
    let mut has_changes = false;
    let mut orders_found = 0;

    let shorten_path = |path: &std::path::Path| -> String {
        let s = path.to_string_lossy();
        let home = ctx.home_dir.to_string_lossy();
        if s.starts_with(home.as_ref()) {
            format!("~{}", &s[home.len()..])
        } else {
            s.into_owned()
        }
    };

    for order_name in &to_view {
        if !ctx.orders_dir.join(order_name).join("order.ncl").exists() {
            log::error(&format!("Order '{order_name}' not found"));
            continue;
        }
        orders_found += 1;
        match build_order(ctx, order_name) {
            Ok(results) => {
                if results.is_empty() {
                    if viewing_specific {
                        log::info(&format!("{order_name} skipped (condition not met)"));
                    }
                    continue;
                }

                println!("\n{}", style(order_name).cyan().bold());

                for result in &results {
                    let target_kind = node_kind(&result.target).with_context(|| {
                        format!("could not inspect target {}", result.target.display())
                    })?;
                    let target_display = shorten_path(&result.target);
                    let immutable_tag = if result.immutable {
                        format!(" {}", style("(immutable)").magenta())
                    } else {
                        String::new()
                    };
                    let file_header =
                        format!("  {} -> {}{}", result.name, target_display, immutable_tag);

                    if result.is_symlink {
                        if let Some(canonical) = &result.canonical_source {
                            let link_status =
                                match (target_kind, std::fs::read_link(&result.target)) {
                                    (Some(NodeKind::Symlink), Ok(existing))
                                        if existing == *canonical =>
                                    {
                                        if short {
                                            continue;
                                        }
                                        style("(linked)").green().to_string()
                                    }
                                    (Some(NodeKind::Symlink), Ok(_)) => {
                                        style("(wrong target)").yellow().to_string()
                                    }
                                    (Some(NodeKind::Symlink), Err(error)) => {
                                        return Err(error).with_context(|| {
                                            format!(
                                                "could not read target symlink {}",
                                                result.target.display()
                                            )
                                        });
                                    }
                                    (Some(actual), _) => style(format!(
                                        "(type mismatch: expected symlink, found {actual})"
                                    ))
                                    .yellow()
                                    .to_string(),
                                    (None, _) => style("(not linked)").yellow().to_string(),
                                };
                            println!(
                                "{} {} {}",
                                file_header,
                                style("(symlink)").dim(),
                                link_status
                            );
                        }
                        continue;
                    }

                    if result.is_plaintext {
                        if let Some(source_path) = &result.source_path {
                            let is_dir = source_path.is_dir();
                            let mut annotations = Vec::new();
                            if show_content {
                                let kind = if is_dir { "directory" } else { "file" };
                                annotations
                                    .push(style(format!("(plaintext {})", kind)).dim().to_string());
                            }

                            if show_diff && is_dir {
                                let target_missing = target_kind.is_none();
                                let direct_mismatch = target_type_mismatch(result)?;
                                // Enumerate per-node status only when the target
                                // root has the expected directory type.
                                let file_diffs = if direct_mismatch.is_none() {
                                    compute_dir_file_diffs(result)?
                                } else {
                                    Vec::new()
                                };
                                let any_file_changes = file_diffs.iter().any(|f| f.has_changes);

                                if target_missing {
                                    annotations.push(style("(not deployed)").yellow().to_string());
                                    println!("{} {}", file_header, annotations.join(" "));
                                    has_changes = true;
                                } else if let Some((expected, actual)) = direct_mismatch {
                                    annotations.push(
                                        style(format!(
                                            "(type mismatch: expected {expected}, found {actual})"
                                        ))
                                        .yellow()
                                        .to_string(),
                                    );
                                    println!("{} {}", file_header, annotations.join(" "));
                                    has_changes = true;
                                } else if file_diffs.is_empty() {
                                    if !short {
                                        annotations
                                            .push(style("(empty directory)").dim().to_string());
                                        println!("{} {}", file_header, annotations.join(" "));
                                    }
                                } else {
                                    // Print directory header
                                    if any_file_changes || !short {
                                        println!("{} {}", file_header, annotations.join(" "));
                                    }

                                    for f in &file_diffs {
                                        let rel = f.rel_path.display();
                                        if f.source_only {
                                            println!(
                                                "    {} {}",
                                                style("+").green(),
                                                style(format!("{} (not deployed)", rel)).green()
                                            );
                                            has_changes = true;
                                        } else if f.has_changes {
                                            let label = if let Some(target_kind) = f.target_kind
                                                && target_kind != f.expected_kind
                                            {
                                                format!(
                                                    "{} (type mismatch: expected {}, found {})",
                                                    rel, f.expected_kind, target_kind
                                                )
                                            } else {
                                                format!("{}", rel)
                                            };
                                            println!(
                                                "    {} {}",
                                                style("\u{2260}").yellow(),
                                                style(label).yellow()
                                            );
                                            if !f.diff_output.is_empty() {
                                                for line in f.diff_output.lines() {
                                                    println!("      {}", line);
                                                }
                                            }
                                            has_changes = true;
                                        } else if !short {
                                            println!(
                                                "    {}",
                                                style(format!("\u{2713} {}", rel)).dim()
                                            );
                                        }
                                    }
                                }
                            } else if show_diff && !is_dir {
                                let diff_result = compute_diff_for_result(result)?;
                                let direct_mismatch = target_type_mismatch(result)?;
                                if let Some((expected, actual)) = direct_mismatch {
                                    annotations.push(
                                        style(format!(
                                            "(type mismatch: expected {expected}, found {actual})"
                                        ))
                                        .yellow()
                                        .to_string(),
                                    );
                                }
                                if diff_result.has_changes {
                                    if annotations.is_empty() {
                                        println!("{}", file_header);
                                    } else {
                                        println!("{} {}", file_header, annotations.join(" "));
                                    }
                                    for line in diff_result.output.lines() {
                                        println!("    {}", line);
                                    }
                                    has_changes = true;
                                } else if target_kind.is_none() {
                                    annotations.push(style("(not deployed)").yellow().to_string());
                                    println!("{} {}", file_header, annotations.join(" "));
                                    has_changes = true;
                                } else if direct_mismatch.is_some() {
                                    println!("{} {}", file_header, annotations.join(" "));
                                    has_changes = true;
                                } else if !short {
                                    annotations.push(style("(no changes)").dim().to_string());
                                    println!("{} {}", file_header, annotations.join(" "));
                                }
                            } else if annotations.is_empty() {
                                println!("{}", file_header);
                            } else {
                                println!("{} {}", file_header, annotations.join(" "));
                            }
                        }
                        continue;
                    }

                    // Structured config
                    let diff_status = if show_diff {
                        Some(compute_diff_for_result(result)?)
                    } else {
                        None
                    };

                    let target_missing = target_kind.is_none();
                    let direct_mismatch = target_type_mismatch(result)?;
                    let has_diff_output = diff_status
                        .as_ref()
                        .is_some_and(|diff_result| diff_result.has_changes);

                    if target_missing || has_diff_output || show_content {
                        if !show_diff {
                            println!("{}", file_header);
                        } else if target_missing {
                            println!("{} {}", file_header, style("(not deployed)").yellow());
                            has_changes = true;
                        } else {
                            println!("{}", file_header);
                        }

                        if show_content {
                            for line in result.content.lines() {
                                println!("    {}", style(line).dim());
                            }
                        }

                        if let Some(dr) = &diff_status
                            && dr.has_changes
                        {
                            for line in dr.output.lines() {
                                println!("    {}", line);
                            }
                            has_changes = true;
                        }
                    } else if let Some((expected, actual)) = direct_mismatch {
                        println!(
                            "{} {}",
                            file_header,
                            style(format!(
                                "(type mismatch: expected {expected}, found {actual})"
                            ))
                            .yellow()
                        );
                        has_changes = true;
                    } else if !short {
                        println!("{} {}", file_header, style("(no changes)").dim());
                    }
                }
            }
            Err(e) => {
                log::error(&format!("Failed to evaluate {order_name}: {e}"));
            }
        }
    }

    if show_diff && !has_changes && orders_found > 0 {
        println!();
        log::success("All orders are up to date");
    }

    Ok(())
}
