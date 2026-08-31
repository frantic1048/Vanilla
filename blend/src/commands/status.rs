use console::style;
use rayon::prelude::*;

use crate::commands::helpers::compute_managed_dir_diffs;
use crate::compose::{discover_orders, get_order};
use crate::context::Context;
use crate::diff::check_file_sync;
use crate::fs_node::{NodeKind, node_kind};
use crate::nickel::generated;
use crate::output::log;

/// Status command: show available orders and their state
pub fn cmd_status(ctx: &Context) -> anyhow::Result<()> {
    generated::assert_orders_ready(&ctx.orders_dir)?;

    let orders = discover_orders(&ctx.orders_dir);
    log::success(&format!("Found {} orders in orders/", orders.len()));

    let order_w = 20;
    let file_w = 20;
    let status_w = 10;
    let diff_w = 5;

    println!(
        "\n{} {} {} {} {}",
        style(format!("{:<order_w$}", "ORDER")).bold(),
        style(format!("{:<file_w$}", "FILE")).bold(),
        style(format!("{:<status_w$}", "STATUS")).bold(),
        style(format!("{:<diff_w$}", "DIFF")).bold(),
        style("TARGET").bold()
    );
    println!("{}", "-".repeat(order_w + file_w + status_w + diff_w + 40));

    let mut order_list: Vec<_> = orders.into_iter().collect::<Vec<_>>();
    order_list.sort();

    let timing = std::env::var("BLEND_TIMING").is_ok();
    let row_groups: Vec<Vec<String>> = order_list
        .par_iter()
        .map(|order_name| {
            let t_order = std::time::Instant::now();
            let mut rows = Vec::new();
            match get_order(ctx, order_name) {
                Ok(order) => {
                    let applies = order.should_apply(
                        &ctx.metadata.os,
                        &ctx.metadata.arch,
                        &ctx.metadata.hostname,
                    );

                    if !applies {
                        rows.push(format!(
                            "{} {} {} {} {}",
                            style(format!("{:<order_w$}", order_name)).dim(),
                            style(format!("{:<file_w$}", "-")).dim(),
                            style(format!("{:<status_w$}", "skipped")).dim(),
                            style(format!("{:<diff_w$}", "\u{00b7}")).dim(),
                            style("(condition not met)").dim()
                        ));
                        return rows;
                    }

                    let files = &order.blend.files;
                    let global_prefix = order.global_prefix();
                    let mut shown_order = false;
                    for file_entry in files {
                        let file_applies = file_entry.should_apply(
                            &ctx.metadata.os,
                            &ctx.metadata.arch,
                            &ctx.metadata.hostname,
                        );

                        if !file_applies {
                            if ctx.verbose {
                                let order_display = if shown_order {
                                    String::new()
                                } else {
                                    shown_order = true;
                                    order_name.to_string()
                                };
                                rows.push(format!(
                                    "{} {} {} {} {}",
                                    style(format!("{:<order_w$}", order_display)).dim(),
                                    style(format!("{:<file_w$}", file_entry.name)).dim(),
                                    style(format!("{:<status_w$}", "skipped")).dim(),
                                    style(format!("{:<diff_w$}", "\u{00b7}")).dim(),
                                    style("(condition not met)").dim()
                                ));
                            }
                            continue;
                        }

                        for (j, target_path) in
                            file_entry.target_paths(global_prefix).iter().enumerate()
                        {
                            let target = ctx.expand_path(target_path);

                            let order_display = if !shown_order && j == 0 {
                                shown_order = true;
                                style(format!("{:<order_w$}", order_name))
                                    .cyan()
                                    .to_string()
                            } else {
                                format!("{:<order_w$}", "")
                            };

                            let source_name = &file_entry.name;
                            let is_dir = file_entry
                                .from_file
                                .as_ref()
                                .map(|f| ctx.orders_dir.join(order_name).join(f).is_dir())
                                .unwrap_or(false);
                            let source_display = if source_name.len() > file_w {
                                format!("{:<file_w$}", format!("{}...", &source_name[..file_w - 3]))
                            } else if is_dir {
                                format!("{:<file_w$}", format!("{}/", source_name))
                            } else {
                                format!("{:<file_w$}", source_name)
                            };

                            let expected_kind = if file_entry.symlink {
                                NodeKind::Symlink
                            } else if is_dir {
                                NodeKind::Directory
                            } else {
                                NodeKind::File
                            };
                            let (target_kind, mut row_error) = match node_kind(&target) {
                                Ok(kind) => (kind, None),
                                Err(error) => {
                                    (None, Some(format!("could not inspect target: {error}")))
                                }
                            };

                            let (status, diff_display) = if row_error.is_some() {
                                (
                                    style(format!("{:<status_w$}", "error")).red().to_string(),
                                    style(format!("{:<diff_w$}", "\u{00b7}")).dim().to_string(),
                                )
                            } else if file_entry.symlink {
                                // Symlink entry: check if symlink exists and points correctly
                                let source_path = ctx
                                    .orders_dir
                                    .join(order_name)
                                    .join(file_entry.from_file.as_deref().unwrap_or(""));
                                let canonical = source_path.canonicalize().ok();
                                match target_kind {
                                    Some(NodeKind::Symlink) => match std::fs::read_link(&target) {
                                        Ok(existing)
                                            if canonical.as_deref() == Some(existing.as_path()) =>
                                        {
                                            (
                                                style(format!("{:<status_w$}", "linked"))
                                                    .green()
                                                    .to_string(),
                                                style(format!("{:<diff_w$}", "\u{2713}"))
                                                    .green()
                                                    .to_string(),
                                            )
                                        }
                                        Ok(_) => (
                                            style(format!("{:<status_w$}", "mismatch"))
                                                .yellow()
                                                .to_string(),
                                            style(format!("{:<diff_w$}", "\u{2260}"))
                                                .yellow()
                                                .to_string(),
                                        ),
                                        Err(error) => {
                                            row_error = Some(format!(
                                                "could not read target symlink: {error}"
                                            ));
                                            (
                                                style(format!("{:<status_w$}", "error"))
                                                    .red()
                                                    .to_string(),
                                                style(format!("{:<diff_w$}", "\u{00b7}"))
                                                    .dim()
                                                    .to_string(),
                                            )
                                        }
                                    },
                                    Some(_) => (
                                        style(format!("{:<status_w$}", "mismatch"))
                                            .yellow()
                                            .to_string(),
                                        style(format!("{:<diff_w$}", "\u{2260}"))
                                            .yellow()
                                            .to_string(),
                                    ),
                                    None => (
                                        style(format!("{:<status_w$}", "pending"))
                                            .yellow()
                                            .to_string(),
                                        style(format!("{:<diff_w$}", "\u{00b7}")).dim().to_string(),
                                    ),
                                }
                            } else if let Some(actual_kind) = target_kind {
                                let order_dir = ctx.orders_dir.join(order_name);
                                let direct_mismatch = actual_kind != expected_kind;
                                let directory_diffs = if direct_mismatch || !is_dir {
                                    Ok(None)
                                } else {
                                    let source_dir = order_dir
                                        .join(file_entry.from_file.as_deref().unwrap_or_default());
                                    let local_dir = file_entry
                                        .local
                                        .as_ref()
                                        .map(|local| order_dir.join(local));
                                    let mut ignore_keys = order.global_ignore().to_vec();
                                    ignore_keys.extend(file_entry.ignore.iter().cloned());
                                    compute_managed_dir_diffs(
                                        &source_dir,
                                        &target,
                                        local_dir.as_deref(),
                                        &file_entry.exclude,
                                        &ignore_keys,
                                    )
                                    .map(Some)
                                };

                                let directory_diffs = match directory_diffs {
                                    Ok(value) => value,
                                    Err(error) => {
                                        row_error = Some(format!(
                                            "could not build managed inventory: {error}"
                                        ));
                                        None
                                    }
                                };
                                let inner_mismatch =
                                    directory_diffs.as_ref().is_some_and(|diffs| {
                                        diffs.iter().any(|diff| diff.has_type_mismatch())
                                    });

                                if row_error.is_some() {
                                    (
                                        style(format!("{:<status_w$}", "error")).red().to_string(),
                                        style(format!("{:<diff_w$}", "\u{00b7}")).dim().to_string(),
                                    )
                                } else if direct_mismatch || inner_mismatch {
                                    (
                                        style(format!("{:<status_w$}", "mismatch"))
                                            .yellow()
                                            .to_string(),
                                        style(format!("{:<diff_w$}", "\u{2260}"))
                                            .yellow()
                                            .to_string(),
                                    )
                                } else {
                                    let sync = if let Some(file_diffs) = directory_diffs {
                                        Ok((!file_diffs.is_empty()).then(|| {
                                            file_diffs.iter().all(|diff| !diff.has_changes)
                                        }))
                                    } else {
                                        check_file_sync(
                                            &order_dir,
                                            file_entry,
                                            &target,
                                            order.global_ignore(),
                                        )
                                    };
                                    let diff_col = match sync {
                                        Ok(Some(true)) => style(format!("{:<diff_w$}", "\u{2713}"))
                                            .green()
                                            .to_string(),
                                        Ok(Some(false)) => {
                                            style(format!("{:<diff_w$}", "\u{2260}"))
                                                .yellow()
                                                .to_string()
                                        }
                                        Ok(None) => style(format!("{:<diff_w$}", "\u{00b7}"))
                                            .dim()
                                            .to_string(),
                                        Err(error) => {
                                            row_error = Some(format!(
                                                "could not compare target content: {error}"
                                            ));
                                            style(format!("{:<diff_w$}", "\u{00b7}"))
                                                .dim()
                                                .to_string()
                                        }
                                    };
                                    let status = if row_error.is_some() {
                                        style(format!("{:<status_w$}", "error")).red().to_string()
                                    } else {
                                        style(format!("{:<status_w$}", "deployed"))
                                            .green()
                                            .to_string()
                                    };
                                    (status, diff_col)
                                }
                            } else {
                                (
                                    style(format!("{:<status_w$}", "pending"))
                                        .yellow()
                                        .to_string(),
                                    style(format!("{:<diff_w$}", "\u{00b7}")).dim().to_string(),
                                )
                            };

                            let target_str = target.to_string_lossy();
                            let home_str = ctx.home_dir.to_string_lossy();
                            let target_display = if target_str.starts_with(home_str.as_ref()) {
                                format!("~{}", &target_str[home_str.len()..])
                            } else {
                                target_str.into_owned()
                            };

                            let target_display = if let Some(error) = row_error {
                                format!("{target_display} ({error})")
                            } else {
                                target_display
                            };

                            rows.push(format!(
                                "{} {} {} {} {}",
                                order_display, source_display, status, diff_display, target_display
                            ));
                        }
                    }
                }
                Err(e) => {
                    let dash_display = format!("{:<file_w$}", "-");
                    rows.push(format!(
                        "{} {} {} {} {}",
                        style(format!("{:<order_w$}", order_name)).red(),
                        dash_display,
                        style(format!("{:<status_w$}", "error")).red(),
                        style(format!("{:<diff_w$}", "\u{00b7}")).dim(),
                        style(e.to_string()).red()
                    ));
                }
            }
            if timing {
                eprintln!(
                    "[timing] order {} total={}us rows={}",
                    order_name,
                    t_order.elapsed().as_micros(),
                    rows.len()
                );
            }
            rows
        })
        .collect();

    for rows in row_groups {
        for row in rows {
            println!("{}", row);
        }
    }

    println!();
    log::info(&format!(
        "System: {} / {} / {}",
        ctx.metadata.os, ctx.metadata.arch, ctx.metadata.hostname
    ));

    Ok(())
}
