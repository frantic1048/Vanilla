use anyhow::Result;
use colored::Colorize;

use crate::build::get_order_package;
use crate::context::Context;
use crate::formats::get_renderer;
use crate::output::log;

/// Sample a deployed config and output as Nickel-ready reference
pub fn sample_package(ctx: &Context, package: &str) -> Result<Option<String>> {
    let order_pkg = match get_order_package(ctx, package) {
        Ok(pkg) => pkg,
        Err(e) => {
            log::warn(&format!("Skipping {} ({})", package, e));
            return Ok(None);
        }
    };

    // Check if package should apply for this system
    if !order_pkg.should_apply(
        &ctx.metadata.os,
        &ctx.metadata.arch,
        &ctx.metadata.hostname,
    ) {
        if ctx.verbose {
            log::info(&format!("Skipping {} (condition not met)", package));
        }
        return Ok(None);
    }

    let pkg_dir = ctx.orders_dir.join(package);
    let global_prefix = order_pkg.global_prefix();
    let mut output = String::new();

    for (i, file_entry) in order_pkg.blend.files.iter().enumerate() {
        // Check per-file condition
        if !file_entry.should_apply(
            &ctx.metadata.os,
            &ctx.metadata.arch,
            &ctx.metadata.hostname,
        ) {
            if ctx.verbose {
                log::info(&format!(
                    "Skipping file {} (condition not met)",
                    file_entry.source
                ));
            }
            continue;
        }

        if i > 0 {
            output.push_str("\n---\n\n");
        }

        output.push_str(&format!("{} {}\n", "Source:".bold(), file_entry.source));

        // Process each target path
        for target_path in file_entry.target_paths(global_prefix) {
            let target = ctx.expand_path(&target_path);
            let source_path = pkg_dir.join(&file_entry.source);

            // Check if source exists on disk (plaintext mode)
            let is_plaintext = source_path.exists() || file_entry.config.is_none();

            output.push_str(&format!("{} {}\n", "Target:".bold(), target.display()));

            if is_plaintext {
                if target.is_dir() {
                    output.push_str(&format!("{} plaintext (directory)\n", "Type:".bold()));
                    output.push_str(&format!("\n{}\n", "Files:".bold()));

                    for entry in walkdir::WalkDir::new(&target).min_depth(1).max_depth(3) {
                        if let Ok(e) = entry {
                            let rel = e.path().strip_prefix(&target).unwrap_or(e.path());
                            let prefix = if e.file_type().is_dir() {
                                "  [dir] "
                            } else {
                                "  "
                            };
                            output.push_str(&format!("{}{}\n", prefix, rel.display()));
                        }
                    }
                } else if target.exists() {
                    output.push_str(&format!("{} plaintext (file)\n", "Type:".bold()));
                } else {
                    output.push_str(&format!(
                        "{} (target does not exist)\n",
                        "Status:".bold()
                    ));
                }
            } else {
                // Structured config
                if !target.exists() {
                    output.push_str(&format!(
                        "{} (not deployed yet)\n",
                        "Status:".bold()
                    ));
                    continue;
                }

                let content = std::fs::read_to_string(&target)?;
                let format = file_entry.effective_format();

                output.push_str(&format!("{} {:?}\n", "Format:".bold(), format));

                // Parse the deployed config
                let renderer = get_renderer(format);
                match renderer.parse(&content) {
                    Ok(parsed) => {
                        output.push_str(&format!("\n{}\n", "Config (as Nickel):".bold()));
                        let nickel_output = serde_json::to_string_pretty(&parsed)?;
                        output.push_str(&nickel_output);
                    }
                    Err(e) => {
                        output.push_str(&format!("\n{}\n", "Raw content:".bold()));
                        output.push_str(&format!("  (failed to parse: {})\n", e));
                        output.push_str(&content);
                    }
                }
            }

            output.push('\n');
        }
    }

    if output.is_empty() {
        return Ok(None);
    }

    Ok(Some(output))
}
