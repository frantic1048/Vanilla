mod build;
mod cli;
mod context;
mod diff;
mod formats;
mod metadata;
mod nickel;
mod output;
mod sync;
mod upgrade;

use clap::Parser;
use colored::Colorize;
use rayon::prelude::*;

use build::{build_package, discover_packages, get_order_package, write_result};
use cli::{Cli, Commands};
use context::Context;
use diff::{check_file_sync, diff_configs};
use output::log;
use sync::sample_package;

fn main() {
    let cli = Cli::parse();
    let ctx = Context::new(&cli);

    if ctx.verbose {
        log::info(&format!("Home directory: {}", ctx.home_dir.display()));
        log::info(&format!("Orders directory: {}", ctx.orders_dir.display()));
        log::info(&format!("OS: {}, Arch: {}", ctx.metadata.os, ctx.metadata.arch));
    }

    let result = match cli.command {
        Some(Commands::Ship { packages }) => cmd_ship(&ctx, &packages),
        Some(Commands::View { packages, content_only, all }) => {
            cmd_view(&ctx, &packages, content_only, all)
        }
        Some(Commands::Sample { packages }) => cmd_sample(&ctx, &packages),
        Some(Commands::Table) => cmd_table(&ctx),
        Some(Commands::Upgrade { step }) => {
            upgrade::cmd_upgrade(&ctx, &step, |ctx| cmd_ship(ctx, &[]))
        }
        None => cmd_status(&ctx),
    };

    if let Err(e) = result {
        log::error(&format!("Error: {}", e));
        std::process::exit(1);
    }
}

/// Ship command: generate and deploy configs to target
fn cmd_ship(ctx: &Context, packages: &[String]) -> anyhow::Result<()> {
    let all_packages = discover_packages(&ctx.orders_dir);

    if all_packages.is_empty() {
        log::warn("No packages found in orders directory");
        return Ok(());
    }

    let to_ship: Vec<String> = if packages.is_empty() {
        all_packages.into_iter().collect()
    } else {
        packages
            .iter()
            .filter(|p| {
                if all_packages.contains(*p) {
                    true
                } else {
                    log::warn(&format!("Package '{}' not found", p));
                    false
                }
            })
            .cloned()
            .collect()
    };

    let mut shipped = 0;
    let mut errors = 0;

    for pkg in &to_ship {
        match build_package(ctx, pkg) {
            Ok(results) => {
                if results.is_empty() {
                    // Package skipped (condition not met)
                    continue;
                }

                for result in results {
                    if ctx.dry_run {
                        log::info_important(&format!(
                            "[dry-run] {}:{} -> {}",
                            pkg,
                            result.source_name,
                            result.target.display()
                        ));
                        if ctx.verbose && !result.is_plaintext {
                            println!("{}", result.content.dimmed());
                        }
                    } else {
                        match write_result(&result, false) {
                            Ok(()) => {
                                log::success(&format!(
                                    "Shipped {}:{} -> {}",
                                    pkg,
                                    result.source_name,
                                    result.target.display()
                                ));
                                shipped += 1;
                            }
                            Err(e) => {
                                log::error(&format!(
                                    "Failed to ship {}:{}: {}",
                                    pkg, result.source_name, e
                                ));
                                errors += 1;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                log::error(&format!("Failed to ship {}: {}", pkg, e));
                errors += 1;
            }
        }
    }

    if !ctx.dry_run {
        log::success(&format!(
            "Shipped {} file(s){}",
            shipped,
            if errors > 0 {
                format!(" ({} errors)", errors)
            } else {
                String::new()
            }
        ));
    }

    Ok(())
}

/// View command: show generated config and/or diff from deployed
fn cmd_view(
    ctx: &Context,
    packages: &[String],
    content_only: bool,
    show_all: bool,
) -> anyhow::Result<()> {
    let all_packages = discover_packages(&ctx.orders_dir);
    let viewing_specific = !packages.is_empty();

    let to_view: Vec<String> = if packages.is_empty() {
        all_packages.into_iter().collect()
    } else {
        packages.to_vec()
    };

    let show_content = content_only || show_all;
    let show_diff = !content_only;
    let mut has_changes = false;

    let shorten_path = |path: &std::path::Path| -> String {
        let s = path.to_string_lossy();
        let home = ctx.home_dir.to_string_lossy();
        if s.starts_with(home.as_ref()) {
            format!("~{}", &s[home.len()..])
        } else {
            s.into_owned()
        }
    };

    for pkg in &to_view {
        match build_package(ctx, pkg) {
            Ok(results) => {
                if results.is_empty() {
                    // Package skipped — only show when specific packages were requested
                    if viewing_specific {
                        log::info(&format!("{} skipped (condition not met)", pkg));
                    }
                    continue;
                }

                println!("\n{}", pkg.cyan().bold());

                for result in results {
                    let target_display = shorten_path(&result.target);
                    let file_header = format!(
                        "  {} -> {}",
                        result.source_name, target_display
                    );

                    if result.is_plaintext {
                        if let Some(source_path) = &result.source_path {
                            let is_dir = source_path.is_dir();
                            // Collect inline annotations
                            let mut annotations = Vec::new();
                            if show_content {
                                let kind = if is_dir { "directory" } else { "file" };
                                annotations.push(format!("(plaintext {})", kind).dimmed().to_string());
                            }

                            if show_diff && !is_dir {
                                let source_content = std::fs::read_to_string(source_path)?;
                                if result.target.exists() {
                                    let deployed = std::fs::read_to_string(&result.target)?;
                                    let diff_result = diff_configs(
                                        nickel::Format::Plaintext,
                                        &source_content,
                                        &deployed,
                                        &result.ignore_keys,
                                    );
                                    if diff_result.has_changes {
                                        println!("{}", file_header);
                                        for line in diff_result.output.lines() {
                                            println!("    {}", line);
                                        }
                                        has_changes = true;
                                    } else {
                                        annotations.push("(no changes)".dimmed().to_string());
                                        println!("{} {}", file_header, annotations.join(" "));
                                    }
                                } else {
                                    annotations.push("(not deployed)".yellow().to_string());
                                    println!("{} {}", file_header, annotations.join(" "));
                                    has_changes = true;
                                }
                            } else if show_diff && is_dir {
                                annotations.push("(directory)".dimmed().to_string());
                                println!("{} {}", file_header, annotations.join(" "));
                            } else {
                                if annotations.is_empty() {
                                    println!("{}", file_header);
                                } else {
                                    println!("{} {}", file_header, annotations.join(" "));
                                }
                            }
                        }
                        continue;
                    }

                    // Compute diff status for inline display
                    let diff_status = if show_diff {
                        if result.target.exists() {
                            let deployed = std::fs::read_to_string(&result.target)?;
                            let format = nickel::Format::from_path(&result.source_name);
                            let diff_result = diff_configs(
                                format,
                                &result.content,
                                &deployed,
                                &result.ignore_keys,
                            );
                            Some(diff_result)
                        } else {
                            None // not deployed
                        }
                    } else {
                        None
                    };

                    // Print file header with inline status when compact
                    let has_diff_output = match &diff_status {
                        Some(dr) => dr.has_changes,
                        None if show_diff => true, // not deployed
                        _ => false,
                    };

                    if has_diff_output || show_content {
                        // Multi-line: header on its own line, details below
                        if !show_diff {
                            println!("{}", file_header);
                        } else if !result.target.exists() {
                            println!("{} {}", file_header, "(not deployed)".yellow());
                            has_changes = true;
                        } else {
                            println!("{}", file_header);
                        }

                        if show_content {
                            for line in result.content.lines() {
                                println!("    {}", line.dimmed());
                            }
                        }

                        if let Some(dr) = &diff_status {
                            if dr.has_changes {
                                for line in dr.output.lines() {
                                    println!("    {}", line);
                                }
                                has_changes = true;
                            }
                        }
                    } else {
                        // Compact: status inline
                        println!("{} {}", file_header, "(no changes)".dimmed());
                    }
                }
            }
            Err(e) => {
                log::error(&format!("Failed to evaluate {}: {}", pkg, e));
            }
        }
    }

    if show_diff && !has_changes && !to_view.is_empty() {
        println!();
        log::success("All packages are up to date");
    }

    Ok(())
}

/// Sample command: capture deployed config as reference
fn cmd_sample(ctx: &Context, packages: &[String]) -> anyhow::Result<()> {
    let all_packages = discover_packages(&ctx.orders_dir);

    let to_sample: Vec<String> = if packages.is_empty() {
        all_packages.into_iter().collect()
    } else {
        packages.to_vec()
    };

    for pkg in &to_sample {
        match sample_package(ctx, pkg) {
            Ok(Some(output)) => {
                println!("\n{} {}", "Package:".bold(), pkg.cyan());
                println!("{}", output);
            }
            Ok(None) => {
                // Package skipped
            }
            Err(e) => {
                log::error(&format!("Failed to sample {}: {}", pkg, e));
            }
        }
    }

    Ok(())
}

/// Table command: output package info as HTML table for README
fn cmd_table(ctx: &Context) -> anyhow::Result<()> {
    let packages = discover_packages(&ctx.orders_dir);

    let profiles: &[(&str, &str, &str)] = &[
        ("linux", "x86_64", "linux-x86_64"),
        ("darwin", "x86_64", "macos-x86_64"),
        ("darwin", "aarch64", "macos-aarch64"),
    ];

    // Collect package data: (name, profile_matches, match_count)
    let mut pkg_data: Vec<(String, Vec<bool>, usize)> = Vec::new();

    for pkg in &packages {
        match get_order_package(ctx, pkg) {
            Ok(order_pkg) => {
                let matches: Vec<bool> = profiles
                    .iter()
                    .map(|(os, arch, _)| order_pkg.applies_on_platform(os, arch))
                    .collect();
                let match_count = matches.iter().filter(|&&m| m).count();
                pkg_data.push((pkg.clone(), matches, match_count));
            }
            Err(e) => {
                log::warn(&format!("Skipping {} (eval error: {})", pkg, e));
            }
        }
    }

    // Sort: more profiles first, then alphabetical
    pkg_data.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.cmp(&b.0)));

    // Output HTML table
    print!("<table><thead><tr><th>package</th><th colspan=\"3\">profiles</th></tr></thead><tbody>");
    for (name, matches, _) in &pkg_data {
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

/// Status command: show available packages and their state
fn cmd_status(ctx: &Context) -> anyhow::Result<()> {
    if !ctx.orders_dir.is_dir() {
        log::error(&format!(
            "Orders directory not found: {}",
            ctx.orders_dir.display()
        ));
        std::process::exit(1);
    }

    let packages = discover_packages(&ctx.orders_dir);
    log::success(&format!("Found {} packages in orders/", packages.len()));

    let pkg_w = 20;
    let file_w = 20;
    let status_w = 10;
    let diff_w = 5;

    println!(
        "\n{} {} {} {} {}",
        format!("{:<pkg_w$}", "PACKAGE").bold(),
        format!("{:<file_w$}", "FILE").bold(),
        format!("{:<status_w$}", "STATUS").bold(),
        format!("{:<diff_w$}", "DIFF").bold(),
        "TARGET".bold()
    );
    println!("{}", "-".repeat(pkg_w + file_w + status_w + diff_w + 40));

    let mut pkg_list: Vec<_> = packages.into_iter().collect::<Vec<_>>();
    pkg_list.sort();

    // Evaluate all packages in parallel, collect formatted rows
    let row_groups: Vec<Vec<String>> = pkg_list
        .par_iter()
        .map(|pkg| {
            let mut rows = Vec::new();
            match get_order_package(ctx, pkg) {
                Ok(order_pkg) => {
                    let applies = order_pkg.should_apply(
                        &ctx.metadata.os,
                        &ctx.metadata.arch,
                        &ctx.metadata.hostname,
                    );

                    if !applies {
                        rows.push(format!(
                            "{} {} {} {} {}",
                            format!("{:<pkg_w$}", pkg).dimmed(),
                            format!("{:<file_w$}", "-").dimmed(),
                            format!("{:<status_w$}", "skipped").dimmed(),
                            format!("{:<diff_w$}", "\u{00b7}").dimmed(),
                            "(condition not met)".dimmed()
                        ));
                        return rows;
                    }

                    let files = &order_pkg.blend.files;
                    let global_prefix = order_pkg.global_prefix();
                    for (i, file_entry) in files.iter().enumerate() {
                        let file_applies = file_entry.should_apply(
                            &ctx.metadata.os,
                            &ctx.metadata.arch,
                            &ctx.metadata.hostname,
                        );

                        if !file_applies {
                            if ctx.verbose {
                                let pkg_display = if i == 0 { pkg.as_str() } else { "" };
                                rows.push(format!(
                                    "{} {} {} {} {}",
                                    format!("{:<pkg_w$}", pkg_display).dimmed(),
                                    format!("{:<file_w$}", &file_entry.source).dimmed(),
                                    format!("{:<status_w$}", "skipped").dimmed(),
                                    format!("{:<diff_w$}", "\u{00b7}").dimmed(),
                                    "(condition not met)".dimmed()
                                ));
                            }
                            continue;
                        }

                        for (j, target_path) in file_entry.target_paths(global_prefix).iter().enumerate() {
                            let target = ctx.expand_path(target_path);

                            let pkg_display = if i == 0 && j == 0 {
                                format!("{:<pkg_w$}", pkg).cyan().to_string()
                            } else {
                                format!("{:<pkg_w$}", "")
                            };

                            let source_name = &file_entry.source;
                            let is_dir = ctx.orders_dir.join(pkg).join(source_name).is_dir();
                            let source_display = if source_name.len() > file_w {
                                format!("{:<file_w$}", format!("{}...", &source_name[..file_w - 3]))
                            } else if is_dir {
                                format!("{:<file_w$}", format!("{}/", source_name))
                            } else {
                                format!("{:<file_w$}", source_name)
                            };

                            let (status, diff_display) = if target.exists() {
                                let pkg_dir = ctx.orders_dir.join(pkg);
                                let sync = check_file_sync(
                                    &pkg_dir,
                                    file_entry,
                                    &target,
                                    order_pkg.global_ignore(),
                                );
                                let diff_col = match sync {
                                    Some(true) => format!("{:<diff_w$}", "\u{2713}").green().to_string(),
                                    Some(false) => format!("{:<diff_w$}", "\u{2260}").yellow().to_string(),
                                    None => format!("{:<diff_w$}", "\u{00b7}").dimmed().to_string(),
                                };
                                (
                                    format!("{:<status_w$}", "deployed").green().to_string(),
                                    diff_col,
                                )
                            } else {
                                (
                                    format!("{:<status_w$}", "pending").yellow().to_string(),
                                    format!("{:<diff_w$}", "\u{00b7}").dimmed().to_string(),
                                )
                            };

                            let target_str = target.to_string_lossy();
                            let home_str = ctx.home_dir.to_string_lossy();
                            let target_display = if target_str.starts_with(home_str.as_ref()) {
                                format!("~{}", &target_str[home_str.len()..])
                            } else {
                                target_str.into_owned()
                            };

                            rows.push(format!(
                                "{} {} {} {} {}",
                                pkg_display,
                                source_display,
                                status,
                                diff_display,
                                target_display
                            ));
                        }
                    }
                }
                Err(e) => {
                    rows.push(format!(
                        "{} {} {} {} {}",
                        format!("{:<pkg_w$}", pkg).red(),
                        format!("{:<file_w$}", "-"),
                        format!("{:<status_w$}", "error").red(),
                        format!("{:<diff_w$}", "\u{00b7}").dimmed(),
                        e.to_string().red()
                    ));
                }
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
