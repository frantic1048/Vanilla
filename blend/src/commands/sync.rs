use crate::commands::helpers::{
    compute_diff_for_result, only_structural_symlink_changes, target_is_unexpected_symlink,
};
use crate::compose::{
    self, BuildResult, build_glob_set, collect_merged_files, discover_orders, write_result,
};
use crate::context::Context;
use crate::diff::{diff_configs_with_base, key_change_with_base_display, semantic_diff_keys};
use crate::formats::get_renderer;
use crate::nickel;
use crate::nickel::generated;
use crate::output::log;
use crate::sync::{self, KeyAction, KeyResolution, Prompter, SyncAction, SyncMode};

/// Sync command: bidirectional sync between repo and deployed configs
pub fn cmd_sync(
    ctx: &Context,
    orders: &[String],
    mode: SyncMode,
    no_rewrite: bool,
    prompter: &dyn Prompter,
) -> anyhow::Result<()> {
    if ctx.dry_run {
        generated::assert_orders_ready(&ctx.orders_dir)?;
    } else {
        generated::ensure_orders_ready(&ctx.orders_dir, false)?;
    }

    let all_orders = discover_orders(&ctx.orders_dir);

    if all_orders.is_empty() {
        log::warn("No orders found in orders directory");
        return Ok(());
    }

    let to_sync: Vec<String> = if orders.is_empty() {
        all_orders.into_iter().collect()
    } else {
        orders
            .iter()
            .filter(|p| {
                if all_orders.contains(*p) {
                    true
                } else {
                    log::warn(&format!("Order '{p}' not found"));
                    false
                }
            })
            .cloned()
            .collect()
    };

    // Phase 1: build all orders and collect results
    // Also track file_entry_index per result for from_config pull-back
    let mut built: Vec<(String, Vec<(BuildResult, usize)>)> = Vec::new();
    let mut build_errors = 0;

    for order_name in &to_sync {
        match build_order_with_indices(ctx, order_name) {
            Ok(results) => {
                if !results.is_empty() {
                    built.push((order_name.clone(), results));
                }
            }
            Err(e) => {
                log::error(&format!("Failed to build {order_name}: {e}"));
                build_errors += 1;
            }
        }
    }

    // Phase 2: for each built file, compute diff and determine action
    let mut source_to_target = 0;
    let mut target_to_source = 0;
    let mut skipped = 0;

    for (order_name, results) in &built {
        for (result, file_entry_index) in results {
            // Symlink entries: ensure the symlink exists and points to the right place
            if result.is_symlink {
                let needs_update = match std::fs::read_link(&result.target) {
                    Ok(existing) => result.canonical_source.as_deref() != Some(existing.as_path()),
                    Err(_) => true, // not a symlink or doesn't exist
                };

                if needs_update {
                    match write_result(result, ctx.dry_run) {
                        Ok(()) => {
                            if !ctx.dry_run {
                                log::success(&format!(
                                    "Linked {}:{} -> {}",
                                    order_name,
                                    result.name,
                                    result.target.display()
                                ));
                                source_to_target += 1;
                            }
                        }
                        Err(e) => {
                            log::error(&format!(
                                "Failed to link {}:{}: {}",
                                order_name, result.name, e
                            ));
                            build_errors += 1;
                        }
                    }
                } else if ctx.verbose {
                    log::info(&format!("{}:{} already linked", order_name, result.name));
                }
                continue;
            }

            // New target file — always apply Source -> Target.
            if !result.target.exists() {
                let snapshot = ctx
                    .state
                    .read(order_name, &result.target)
                    .unwrap_or_else(|e| {
                        log::warn(&format!(
                            "snapshot read failed for {}:{}: {}",
                            order_name, result.name, e
                        ));
                        None
                    });
                if let Some(annotation) =
                    sync::file_annotation_for_deleted_target(snapshot.as_deref())
                {
                    log::info_important(&format!(
                        "  [snapshot] since last sync: {}",
                        annotation.message()
                    ));
                }
                if ctx.dry_run {
                    log::info_important(&format!(
                        "[dry-run] {}:{} -> {} (new target, would apply Source -> Target)",
                        order_name,
                        result.name,
                        result.target.display()
                    ));
                } else {
                    match write_result(result, false) {
                        Ok(()) => {
                            log::success(&format!(
                                "Applied Source -> Target for {}:{} -> {} (new)",
                                order_name,
                                result.name,
                                result.target.display()
                            ));
                            source_to_target += 1;
                            refresh_snapshot_for_result(ctx, order_name, result);
                        }
                        Err(e) => {
                            log::error(&format!(
                                "Failed to apply Source -> Target for {}:{}: {}",
                                order_name, result.name, e
                            ));
                            build_errors += 1;
                        }
                    }
                }
                continue;
            }

            // Compute diff
            let diff_result = compute_diff_for_result(result);

            // Auto-redeploy when Source -> Target will only change file types (replace
            // unexpected symlinks with real files), no content edits. Two
            // shapes: top-level symlink with matching content, OR a
            // directory entry whose only drift is inner-file symlinks.
            let top_level_symlink = !diff_result.has_changes
                && target_is_unexpected_symlink(&result.target, result.is_symlink);
            let inner_symlink_only = only_structural_symlink_changes(result);

            if top_level_symlink || inner_symlink_only {
                if ctx.dry_run {
                    log::info_important(&format!(
                        "[dry-run] {}:{} content matches but target is a symlink — would re-deploy",
                        order_name, result.name
                    ));
                } else {
                    match write_result(result, false) {
                        Ok(()) => {
                            log::success(&format!(
                                "Re-deployed {}:{} (replaced symlink with real file/directory)",
                                order_name, result.name
                            ));
                            source_to_target += 1;
                            refresh_snapshot_for_result(ctx, order_name, result);
                        }
                        Err(e) => {
                            log::error(&format!(
                                "Failed to re-deploy {}:{}: {}",
                                order_name, result.name, e
                            ));
                            build_errors += 1;
                        }
                    }
                }
                continue;
            }

            if !diff_result.has_changes {
                if !ctx.dry_run {
                    refresh_snapshot_for_result(ctx, order_name, result);
                }
                if ctx.verbose {
                    log::info(&format!("{}:{} in sync", order_name, result.name));
                }
                continue;
            }

            // Determine if Target -> Source is possible for this entry.
            let can_pull = !no_rewrite && can_auto_pull(ctx, order_name, result, *file_entry_index);

            // For from_config entries in interactive mode, use per-key flow
            let is_from_config = !result.is_plaintext && !result.is_symlink;
            // Per-key only works if the deployed file can be semantically parsed
            let deployed_parseable = if is_from_config {
                let format = result.format;
                std::fs::read_to_string(&result.target)
                    .ok()
                    .and_then(|s| get_renderer(format).parse(&s).ok())
                    .is_some()
            } else {
                false
            };
            let use_per_key = is_from_config
                && matches!(mode, SyncMode::Interactive)
                && !ctx.dry_run
                && can_pull
                && deployed_parseable;

            if use_per_key {
                // Per-key interactive sync for from_config entries
                let format = result.format;
                let key_changes = semantic_diff_keys(
                    format,
                    &result.content,
                    &std::fs::read_to_string(&result.target).unwrap_or_default(),
                    &result.ignore_keys,
                );

                if key_changes.is_empty() {
                    if ctx.verbose {
                        log::info(&format!("{}:{} in sync (per-key)", order_name, result.name));
                    }
                    continue;
                }

                // Read & parse the snapshot once per entry so per-key annotations
                // reuse the parsed JSON. Missing or unparseable snapshots degrade
                // to unannotated 2-way (annotation = None for every key).
                let snapshot_json: Option<serde_json::Value> = match ctx
                    .state
                    .read(order_name, &result.target)
                {
                    Ok(Some(bytes)) => {
                        let format_renderer = get_renderer(format);
                        match std::str::from_utf8(&bytes)
                            .ok()
                            .and_then(|s| format_renderer.parse(s).ok())
                        {
                            Some(v) => Some(v),
                            None => {
                                log::warn(&format!(
                                    "snapshot for {}:{} is unparseable in format {:?}; falling back to 2-way",
                                    order_name, result.name, format
                                ));
                                None
                            }
                        }
                    }
                    Ok(None) => None,
                    Err(e) => {
                        log::warn(&format!(
                            "snapshot read failed for {}:{}: {}",
                            order_name, result.name, e
                        ));
                        None
                    }
                };
                let rendered_json_for_annot: serde_json::Value = get_renderer(format)
                    .parse(&result.content)
                    .unwrap_or_default();
                let deployed_json_for_annot: serde_json::Value =
                    std::fs::read_to_string(&result.target)
                        .ok()
                        .and_then(|s| get_renderer(format).parse(&s).ok())
                        .unwrap_or_default();

                // Display file header
                let target_display = sync::shorten_path(&result.target, &ctx.home_dir);
                println!(
                    "\n  {}:{} ({})",
                    console::style(order_name).cyan(),
                    result.name,
                    target_display
                );

                let mut decisions = std::collections::HashMap::new();
                let mut all_mode: Option<KeyResolution> = None;
                let mut quit = false;

                for change in &key_changes {
                    let action = if let Some(resolution) = all_mode {
                        match resolution {
                            KeyResolution::Source => KeyAction::UseSource,
                            KeyResolution::Target => KeyAction::UseTarget,
                        }
                    } else {
                        let key_annotation = sync::compute_key_annotation(
                            snapshot_json.as_ref(),
                            &rendered_json_for_annot,
                            &deployed_json_for_annot,
                            &change.path,
                        );
                        let prompt_change =
                            key_change_with_base_display(change, snapshot_json.as_ref());
                        prompter.ask_key_action(
                            order_name,
                            &result.name,
                            &prompt_change,
                            key_annotation,
                        )
                    };

                    match action {
                        KeyAction::UseSource => {
                            decisions.insert(change.path.clone(), KeyResolution::Source);
                        }
                        KeyAction::UseTarget => {
                            decisions.insert(change.path.clone(), KeyResolution::Target);
                        }
                        KeyAction::Skip => {
                            // No decision for this key -- skip means keep as-is
                        }
                        KeyAction::AllSource => {
                            all_mode = Some(KeyResolution::Source);
                            decisions.insert(change.path.clone(), KeyResolution::Source);
                        }
                        KeyAction::AllTarget => {
                            all_mode = Some(KeyResolution::Target);
                            decisions.insert(change.path.clone(), KeyResolution::Target);
                        }
                        KeyAction::Quit => {
                            quit = true;
                            break;
                        }
                    }
                }

                if quit {
                    log::info("Sync aborted by user");
                    return Ok(());
                }

                if decisions.is_empty() {
                    skipped += 1;
                    continue;
                }

                let has_source_resolutions =
                    decisions.values().any(|&v| v == KeyResolution::Source);
                let has_target_resolutions =
                    decisions.values().any(|&v| v == KeyResolution::Target);

                if has_source_resolutions || has_target_resolutions {
                    let format_renderer = get_renderer(format);
                    let source_json: serde_json::Value =
                        format_renderer.parse(&result.content).unwrap_or_default();
                    let target_json: serde_json::Value = std::fs::read_to_string(&result.target)
                        .ok()
                        .and_then(|s| format_renderer.parse(&s).ok())
                        .unwrap_or_default();

                    // Build merged JSON from decisions
                    let merged = sync::build_merged_json(&source_json, &target_json, &decisions);

                    // Write merged result to target
                    match format_renderer.render(&merged) {
                        Ok(merged_content) => {
                            let merged_result = BuildResult {
                                target: result.target.clone(),
                                content: merged_content,
                                is_plaintext: false,
                                source_path: None,
                                name: result.name.clone(),
                                format,
                                ignore_keys: result.ignore_keys.clone(),
                                is_symlink: false,
                                canonical_source: None,
                                exclude_patterns: vec![],
                                local_dir: None,
                                immutable: result.immutable,
                            };
                            match write_result(&merged_result, false) {
                                Ok(()) => {
                                    source_to_target += 1;
                                    refresh_snapshot_for_result(ctx, order_name, &merged_result);
                                    log::success(&format!(
                                        "Synced {}:{} ({} keys resolved)",
                                        order_name,
                                        result.name,
                                        decisions.len()
                                    ));
                                }
                                Err(e) => {
                                    log::error(&format!(
                                        "Failed to write merged {}:{}: {}",
                                        order_name, result.name, e
                                    ));
                                    build_errors += 1;
                                }
                            }
                        }
                        Err(e) => {
                            log::error(&format!(
                                "Failed to render merged {}:{}: {}",
                                order_name, result.name, e
                            ));
                            build_errors += 1;
                        }
                    }

                    // Apply target selections: surgically rewrite .ncl for those keys.
                    if has_target_resolutions && !no_rewrite {
                        let target_keys: Vec<String> = decisions
                            .iter()
                            .filter(|(_, resolution)| **resolution == KeyResolution::Target)
                            .map(|(k, _)| k.clone())
                            .collect();

                        match sync::pull_from_config_keys(
                            ctx,
                            order_name,
                            *file_entry_index,
                            &result.target,
                            format,
                            &target_keys,
                            false,
                        ) {
                            Ok(true) => {
                                target_to_source += 1;
                            }
                            Ok(false) => {
                                log::warn(&format!(
                                    "Some Target-selected keys in {}:{} could not be auto-rewritten",
                                    order_name, result.name
                                ));
                            }
                            Err(e) => {
                                log::error(&format!(
                                    "Failed to rewrite .ncl for {}:{}: {}",
                                    order_name, result.name, e
                                ));
                                build_errors += 1;
                            }
                        }
                    }
                } else {
                    skipped += 1;
                }
            } else {
                // Original whole-file flow for from_file, symlinks, and
                // non-interactive / non-pullable from_config entries

                let snapshot = ctx
                    .state
                    .read(order_name, &result.target)
                    .unwrap_or_else(|e| {
                        log::warn(&format!(
                            "snapshot read failed for {}:{}: {}",
                            order_name, result.name, e
                        ));
                        None
                    });
                let diff_with_base = compute_diff_with_base_for_result(result, snapshot.as_deref());
                let prompt_diff = diff_with_base.as_ref().unwrap_or(&diff_result);

                // Display the diff
                sync::display_conflict(
                    order_name,
                    &result.name,
                    &result.target,
                    prompt_diff,
                    &ctx.home_dir,
                );

                // Determine action
                let action = match mode {
                    SyncMode::ApplySourceToTargetAll => SyncAction::ApplySourceToTarget,
                    SyncMode::ApplyTargetToSourceAll => {
                        if can_pull {
                            SyncAction::ApplyTargetToSource
                        } else {
                            log::warn(&format!(
                                "Cannot apply target to source for {}:{} (from_config contains logic), skipping",
                                order_name, result.name
                            ));
                            SyncAction::Skip
                        }
                    }
                    SyncMode::Interactive => {
                        if ctx.dry_run {
                            let pull_note = if can_pull {
                                ", Target -> Source available"
                            } else {
                                ", manual merge needed"
                            };
                            log::info_important(&format!("[dry-run] would prompt{}", pull_note));
                            SyncAction::Skip
                        } else {
                            let deployed_bytes = std::fs::read(&result.target).unwrap_or_default();
                            let annotation = sync::compute_file_annotation(
                                snapshot.as_deref(),
                                result.content.as_bytes(),
                                &deployed_bytes,
                            );
                            prompter.ask_sync_action(
                                order_name,
                                &result.name,
                                &result.target,
                                &diff_result,
                                can_pull,
                                annotation,
                            )
                        }
                    }
                };

                match action {
                    SyncAction::ApplySourceToTarget => match write_result(result, ctx.dry_run) {
                        Ok(()) => {
                            if !ctx.dry_run {
                                log::success(&format!(
                                    "Applied Source -> Target for {}:{}",
                                    order_name, result.name
                                ));
                                source_to_target += 1;
                                refresh_snapshot_for_result(ctx, order_name, result);
                            }
                        }
                        Err(e) => {
                            log::error(&format!(
                                "Failed to apply Source -> Target for {}:{}: {}",
                                order_name, result.name, e
                            ));
                            build_errors += 1;
                        }
                    },
                    SyncAction::ApplyTargetToSource => {
                        let pull_result = if result.is_plaintext {
                            if let Some(source_path) = &result.source_path {
                                sync::pull_from_file(
                                    source_path,
                                    &result.target,
                                    result.local_dir.as_deref(),
                                    &result.exclude_patterns,
                                    ctx.dry_run,
                                )
                            } else {
                                Err(anyhow::anyhow!("No source path for plaintext entry"))
                            }
                        } else {
                            let format = result.format;
                            match sync::pull_from_config(
                                ctx,
                                order_name,
                                *file_entry_index,
                                &result.target,
                                format,
                                ctx.dry_run,
                            ) {
                                Ok(true) => Ok(()),
                                Ok(false) => {
                                    log::warn(
                                        "Cannot apply Target -> Source (from_config has logic)",
                                    );
                                    Ok(())
                                }
                                Err(e) => Err(e),
                            }
                        };

                        match pull_result {
                            Ok(()) => {
                                if !ctx.dry_run {
                                    log::success(&format!(
                                        "Applied Target -> Source for {}:{}",
                                        order_name, result.name
                                    ));
                                    target_to_source += 1;
                                    refresh_snapshot_for_result(ctx, order_name, result);
                                }
                            }
                            Err(e) => {
                                log::error(&format!(
                                    "Failed to apply Target -> Source for {}:{}: {}",
                                    order_name, result.name, e
                                ));
                                build_errors += 1;
                            }
                        }
                    }
                    SyncAction::Skip => {
                        skipped += 1;
                    }
                    SyncAction::Quit => {
                        log::info("Sync aborted by user");
                        return Ok(());
                    }
                }
            }
        }
    }

    // Summary
    if !ctx.dry_run {
        let error_note = if build_errors > 0 {
            format!(" ({} errors)", build_errors)
        } else {
            String::new()
        };
        log::success(&format!(
            "Sync complete: {} Source -> Target, {} Target -> Source, {} skipped{}",
            source_to_target, target_to_source, skipped, error_note
        ));
    }

    Ok(())
}

fn compute_diff_with_base_for_result(
    result: &BuildResult,
    snapshot: Option<&[u8]>,
) -> Option<crate::diff::DiffResult> {
    let base = std::str::from_utf8(snapshot?).ok()?;

    if result.is_symlink || !result.target.exists() {
        return None;
    }

    if result.is_plaintext {
        let source_path = result.source_path.as_ref()?;
        if source_path.is_dir() {
            return None;
        }
        let source_content = std::fs::read_to_string(source_path).ok()?;
        let deployed = std::fs::read_to_string(&result.target).ok()?;
        Some(diff_configs_with_base(
            nickel::Format::Plaintext,
            &source_content,
            &deployed,
            base,
            &result.ignore_keys,
        ))
    } else {
        let deployed = std::fs::read_to_string(&result.target).ok()?;
        Some(diff_configs_with_base(
            result.format,
            &result.content,
            &deployed,
            base,
            &result.ignore_keys,
        ))
    }
}

/// Refresh snapshots for a successfully-deployed `BuildResult`.
///
/// For single-file entries, writes one snapshot keyed by the deployed
/// target. For directory entries (plaintext source is a directory),
/// writes snapshots only for files that blend builds from Source/local
/// overlays. Target-only files are outside blend's ownership boundary and
/// intentionally ignored.
/// Symlink entries are skipped — they have no semantic content.
///
/// Failures are logged at warn level and are non-fatal — the deploy
/// already succeeded, and the next no-op confirm will heal the
/// snapshot.
fn refresh_snapshot_for_result(ctx: &Context, order_name: &str, result: &BuildResult) {
    if result.is_symlink {
        return;
    }

    if result.is_plaintext
        && result
            .source_path
            .as_ref()
            .is_some_and(|source_path| source_path.is_dir())
    {
        let Some(source_path) = &result.source_path else {
            return;
        };
        let exclude = match build_glob_set(&result.exclude_patterns) {
            Ok(exclude) => exclude,
            Err(e) => {
                log::warn(&format!(
                    "snapshot refresh: failed to compile exclude patterns for {}: {}",
                    result.name, e
                ));
                return;
            }
        };
        let managed_files = match collect_merged_files(
            source_path,
            result.local_dir.as_deref(),
            exclude.as_ref(),
        ) {
            Ok(files) => files,
            Err(e) => {
                log::warn(&format!(
                    "snapshot refresh: failed to collect managed files for {}: {}",
                    source_path.display(),
                    e
                ));
                return;
            }
        };

        for managed_file in managed_files {
            let target_path = result.target.join(&managed_file.rel_path);
            let bytes = match std::fs::read(&target_path) {
                Ok(b) => b,
                Err(e) => {
                    log::warn(&format!(
                        "snapshot refresh: failed to read {}: {}",
                        target_path.display(),
                        e
                    ));
                    continue;
                }
            };
            if let Err(e) = ctx.state.write(order_name, &target_path, &bytes) {
                log::warn(&format!(
                    "snapshot refresh: failed to write snapshot for {}: {}",
                    target_path.display(),
                    e
                ));
            }
        }
        return;
    }

    // Single-file entry — read deployed bytes and snapshot.
    let bytes = match std::fs::read(&result.target) {
        Ok(b) => b,
        Err(e) => {
            log::warn(&format!(
                "snapshot refresh: failed to read {}: {}",
                result.target.display(),
                e
            ));
            return;
        }
    };
    if let Err(e) = ctx.state.write(order_name, &result.target, &bytes) {
        log::warn(&format!(
            "snapshot refresh: failed to write snapshot for {}: {}",
            result.target.display(),
            e
        ));
    }
}

/// Check if Target -> Source can be applied automatically for a build result.
/// from_file entries can always be pulled. from_config entries use context-aware
/// shadow walk — Target -> Source is possible if the walk reaches literal leaves.
fn can_auto_pull(
    ctx: &Context,
    order_name: &str,
    result: &BuildResult,
    file_entry_index: usize,
) -> bool {
    if result.is_plaintext {
        return true;
    }

    let order_dir = ctx.orders_dir.join(order_name);
    let ncl_path = order_dir.join("order.ncl");

    let source = match std::fs::read_to_string(&ncl_path) {
        Ok(s) => s,
        Err(_) => return false,
    };

    match nickel::ast_utils::locate_from_config(&source, file_entry_index, &ctx.metadata) {
        Ok(result) => result.has_any_rewritable(),
        Err(_) => false,
    }
}

/// Build an order and return results with file_entry_index for each result
fn build_order_with_indices(
    ctx: &Context,
    order_name: &str,
) -> anyhow::Result<Vec<(BuildResult, usize)>> {
    let order_dir = ctx.orders_dir.join(order_name);
    let ncl_path = order_dir.join("order.ncl");

    if !ncl_path.exists() {
        return Ok(vec![]);
    }

    let evaluator = nickel::NickelEvaluator::new(&ctx.metadata);
    let order = evaluator.evaluate(&ncl_path)?;

    if !order.should_apply(&ctx.metadata.os, &ctx.metadata.arch, &ctx.metadata.hostname) {
        return Ok(vec![]);
    }

    let mut results = Vec::new();
    let global_ignore = order.global_ignore();
    let global_prefix = order.global_prefix();

    for (file_entry_index, file_entry) in order.blend.files.iter().enumerate() {
        if !file_entry.should_apply(&ctx.metadata.os, &ctx.metadata.arch, &ctx.metadata.hostname) {
            if ctx.verbose {
                log::info(&format!(
                    "Skipping file {} (when condition not met)",
                    file_entry.name,
                ));
            }
            continue;
        }

        let mut ignore_keys: Vec<String> = global_ignore.to_vec();
        ignore_keys.extend(file_entry.ignore.iter().cloned());

        for target_path in file_entry.target_paths(global_prefix) {
            let expanded_target = ctx.expand_path(&target_path);
            let result = compose::build_file_entry_pub(
                ctx,
                &order_dir,
                file_entry,
                expanded_target,
                ignore_keys.clone(),
            )?;
            results.push((result, file_entry_index));
        }
    }

    Ok(results)
}
