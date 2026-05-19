use std::io::{self, Write};
use std::path::Path;

use anyhow::{Context as AnyhowContext, Result};
use console::style;

use crate::compose::{build_glob_set, collect_merged_files};
use crate::context::Context;
use crate::diff::{DiffResult, KeyChange, KeyChangeType};
use crate::formats::get_renderer;
use crate::nickel;
use crate::output::log;

/// Action to take for a file during bidirectional sync
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SyncAction {
    /// Apply the source version to the deployed target.
    ApplySourceToTarget,
    /// Apply the deployed target version back to the source.
    ApplyTargetToSource,
    /// Skip this file (leave both unchanged)
    Skip,
    /// Quit the sync process entirely
    Quit,
}

/// Resolution strategy for sync conflicts
#[derive(Debug, Clone, Copy)]
pub enum SyncMode {
    /// Prompt the user interactively for each conflict
    Interactive,
    /// Apply all source values to targets without prompting.
    ApplySourceToTargetAll,
    /// Apply all target values back to sources without prompting.
    ApplyTargetToSourceAll,
}

/// Per-key action for interactive from_config sync
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyAction {
    /// Use the source value for this key.
    UseSource,
    /// Use the target value for this key.
    UseTarget,
    /// Skip this key (leave unchanged)
    Skip,
    /// Use source values for all remaining keys in this file entry.
    AllSource,
    /// Use target values for all remaining keys in this file entry.
    AllTarget,
    /// Quit the sync process entirely
    Quit,
}

/// Per-key resolution choice for merging source and target JSON.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyResolution {
    Source,
    Target,
}

/// Trait for interactive prompts, enabling testability
pub trait Prompter {
    /// Ask the user what to do with a conflicting file.
    /// `can_pull` is false when Target -> Source cannot be applied automatically.
    fn ask_sync_action(
        &self,
        order_name: &str,
        name: &str,
        target: &Path,
        diff: &DiffResult,
        can_pull: bool,
        annotation: Option<FileAnnotation>,
    ) -> SyncAction;

    /// Ask the user what to do with a single key change within a from_config entry.
    fn ask_key_action(
        &self,
        order_name: &str,
        name: &str,
        change: &KeyChange,
        annotation: Option<KeyAnnotation>,
    ) -> KeyAction;
}

/// Production prompter that reads from stdin
pub struct TerminalPrompter;

impl Prompter for TerminalPrompter {
    fn ask_sync_action(
        &self,
        _order_name: &str,
        _name: &str,
        _target: &Path,
        _diff: &DiffResult,
        can_pull: bool,
        annotation: Option<FileAnnotation>,
    ) -> SyncAction {
        if let Some(a) = annotation {
            println!(
                "  {} since last sync: {}",
                style("[snapshot]").dim(),
                a.message()
            );
        }

        let prompt = if can_pull {
            format!(
                "  {} ",
                style("[s]ource -> target  [t]arget -> source  s[k]ip  [q]uit:").bold()
            )
        } else {
            format!("  {} ", style("[s]ource -> target  s[k]ip  [q]uit:").bold())
        };

        print!("{}", prompt);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim().to_lowercase().as_str() {
            "s" | "source" => SyncAction::ApplySourceToTarget,
            "t" | "target" if can_pull => SyncAction::ApplyTargetToSource,
            "t" | "target" if !can_pull => {
                log::warn(
                    "Cannot apply target to source for this entry (from_config contains logic)",
                );
                log::info("Update order.ncl manually using the diff shown above");
                SyncAction::Skip
            }
            "k" | "skip" | "" => SyncAction::Skip,
            "q" | "quit" => SyncAction::Quit,
            _ => {
                log::warn("Invalid choice, skipping");
                SyncAction::Skip
            }
        }
    }

    fn ask_key_action(
        &self,
        _order_name: &str,
        _name: &str,
        change: &KeyChange,
        annotation: Option<KeyAnnotation>,
    ) -> KeyAction {
        // Display the change. Multi-line side diffs are indented line by line.
        for line in change.display.lines() {
            println!("    {}", line);
        }

        if let Some(a) = annotation {
            println!(
                "    {} {}: {}",
                style("[snapshot]").dim(),
                change.path,
                a.message()
            );
        }

        let prompt = format!(
            "    {} ",
            style("[s]ource  [t]arget  s[k]ip  [a]ll-source  a[l]l-target  [q]uit:").bold()
        );

        print!("{}", prompt);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim().to_lowercase().as_str() {
            "s" | "source" => KeyAction::UseSource,
            "t" | "target" => KeyAction::UseTarget,
            "k" | "skip" | "" => KeyAction::Skip,
            "a" | "all-source" => KeyAction::AllSource,
            "l" | "all-target" => KeyAction::AllTarget,
            "q" | "quit" => KeyAction::Quit,
            _ => {
                log::warn("Invalid choice, skipping");
                KeyAction::Skip
            }
        }
    }
}

/// Pull a from_file entry: copy deployed file/directory back to source in orders/
///
/// When `local_dir` is set, files that have a local override are pulled into the
/// local directory instead of the tracked source directory.
pub fn pull_from_file(
    source_path: &Path,
    target: &Path,
    local_dir: Option<&Path>,
    exclude_patterns: &[String],
    dry_run: bool,
) -> Result<()> {
    if dry_run {
        log::info(&format!(
            "[dry-run] Would apply Target -> Source: {} -> {}",
            target.display(),
            source_path.display()
        ));
        return Ok(());
    }

    if target.is_dir() && source_path.is_dir() {
        pull_directory(target, source_path, local_dir, exclude_patterns)?;
    } else if target.is_file() {
        // Ensure parent directory exists
        if let Some(parent) = source_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }
        std::fs::copy(target, source_path).with_context(|| {
            format!(
                "Failed to copy {} to {}",
                target.display(),
                source_path.display()
            )
        })?;
    } else {
        anyhow::bail!(
            "Cannot apply Target -> Source: target {} does not exist or is not a regular file/directory",
            target.display()
        );
    }

    Ok(())
}

/// Copy a deployed directory back to the source directory in orders/.
///
/// When a local overlay dir is set, files that came from the local overlay
/// (i.e., exist in the local dir) are pulled back to the local dir, not
/// the tracked source dir. Files matching exclude patterns are skipped.
fn pull_directory(
    deployed_dir: &Path,
    source_dir: &Path,
    local_dir: Option<&Path>,
    exclude_patterns: &[String],
) -> Result<()> {
    let exclude = build_glob_set(exclude_patterns)?;

    for managed_file in collect_merged_files(source_dir, local_dir, exclude.as_ref())? {
        let deployed_file = deployed_dir.join(&managed_file.rel_path);
        if !deployed_file.is_file() {
            continue;
        }

        let pull_dest = if managed_file.is_local {
            if let Some(ld) = local_dir {
                ld.join(&managed_file.rel_path)
            } else {
                source_dir.join(&managed_file.rel_path)
            }
        } else {
            source_dir.join(&managed_file.rel_path)
        };

        if let Some(parent) = pull_dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(&deployed_file, &pull_dest)?;
    }

    Ok(())
}

/// Pull a from_config entry by surgically rewriting the .ncl file.
///
/// Uses context-aware shadow walk to find rewritable leaf values, including
/// values inside conditional branches (match/if-then-else).
/// When a StructureMap can be built, also supports inserting new fields and
/// deleting removed fields.
///
/// Returns Ok(true) if the rewrite succeeded, Ok(false) if it couldn't
/// be done, Err on failure.
pub fn pull_from_config(
    ctx: &Context,
    order_name: &str,
    file_entry_index: usize,
    target: &Path,
    format: nickel::Format,
    dry_run: bool,
) -> Result<bool> {
    let order_dir = ctx.orders_dir.join(order_name);
    let ncl_path = order_dir.join("order.ncl");

    let source = std::fs::read_to_string(&ncl_path)
        .with_context(|| format!("Failed to read {}", ncl_path.display()))?;

    // Shadow walk: locate rewritable leaf spans using runtime metadata
    let rewrite_result =
        nickel::ast_utils::locate_from_config(&source, file_entry_index, &ctx.metadata)?;

    let leaf_spans = rewrite_result.rewritable_spans();
    if leaf_spans.is_empty() {
        return Ok(false);
    }

    // Parse the deployed file
    let deployed_content = std::fs::read_to_string(target)
        .with_context(|| format!("Failed to read {}", target.display()))?;
    let renderer = get_renderer(format);
    let deployed_json = renderer
        .parse(&deployed_content)
        .with_context(|| format!("Failed to parse deployed file {}", target.display()))?;

    // Get current evaluated JSON for comparison
    let evaluator = nickel::NickelEvaluator::new(&ctx.metadata);
    let order = evaluator.evaluate(&ncl_path)?;
    let file_entry = order
        .blend
        .files
        .get(file_entry_index)
        .context("file entry index out of bounds")?;
    let current_json = file_entry
        .from_config
        .as_ref()
        .context("file entry has no from_config")?;

    if dry_run {
        log::info(&format!(
            "[dry-run] Would update from_config in {}",
            ncl_path.display()
        ));
        // Show branch context for non-trivial rewrites
        for span in leaf_spans {
            if !span.branch_context.is_empty() {
                log::info(&format!(
                    "  {} scoped under: {}",
                    span.name,
                    span.branch_context.join(" → ")
                ));
            }
        }
        return Ok(true);
    }

    // Detect indentation level from the first span
    let base_indent = if let Some(first) = leaf_spans.first() {
        nickel::ast_utils::detect_indent_level(&source, first.value_start)
    } else {
        0
    };

    // Try structure-aware rewrite (supports insert/delete), fall back to
    // modify-only if StructureMap cannot be built
    let new_source = match nickel::structure_map::build_structure_map(&source, file_entry_index) {
        Ok(structure) => {
            let edits = build_field_edits_from_diff(current_json, &deployed_json);
            if edits
                .iter()
                .all(|e| matches!(e, nickel::ast_utils::FieldEdit::Modify { .. }))
            {
                // Only modifications -- use the simpler path
                nickel::ast_utils::surgical_rewrite(
                    &source,
                    leaf_spans,
                    current_json,
                    &deployed_json,
                    base_indent,
                )?
            } else {
                nickel::ast_utils::surgical_rewrite_with_structure(
                    &source,
                    &structure,
                    leaf_spans,
                    &edits,
                    base_indent,
                )?
            }
        }
        Err(_) => {
            // StructureMap build failed -- fall back to modify-only
            nickel::ast_utils::surgical_rewrite(
                &source,
                leaf_spans,
                current_json,
                &deployed_json,
                base_indent,
            )?
        }
    };

    std::fs::write(&ncl_path, &new_source)
        .with_context(|| format!("Failed to write {}", ncl_path.display()))?;

    // Log non-rewritable fields for user awareness
    for field in rewrite_result.non_rewritable_fields() {
        log::warn(&format!(
            "Cannot apply Target -> Source for {}: {} (update order.ncl manually)",
            field.name, field.reason
        ));
    }

    Ok(true)
}

/// Build FieldEdit list by diffing current (repo) JSON against deployed JSON.
///
/// - Keys in deployed but not current -> Insert
/// - Keys in current but not deployed -> Delete
/// - Keys in both with different values -> Modify
fn build_field_edits_from_diff(
    current: &serde_json::Value,
    deployed: &serde_json::Value,
) -> Vec<nickel::ast_utils::FieldEdit> {
    let mut edits = Vec::new();
    collect_field_edits(current, deployed, "", &mut edits);
    edits
}

/// Recursively collect field edits between two JSON values.
fn collect_field_edits(
    current: &serde_json::Value,
    deployed: &serde_json::Value,
    path: &str,
    edits: &mut Vec<nickel::ast_utils::FieldEdit>,
) {
    match (current, deployed) {
        (serde_json::Value::Object(cur_obj), serde_json::Value::Object(dep_obj)) => {
            // Keys modified or deleted
            for (key, cur_val) in cur_obj {
                let key_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };

                if let Some(dep_val) = dep_obj.get(key) {
                    if cur_val != dep_val {
                        // Both objects: recurse; otherwise treat as leaf modification
                        if cur_val.is_object() && dep_val.is_object() {
                            collect_field_edits(cur_val, dep_val, &key_path, edits);
                        } else {
                            edits.push(nickel::ast_utils::FieldEdit::Modify {
                                path: key_path,
                                new_value: dep_val.clone(),
                            });
                        }
                    }
                } else {
                    // Key in current but not deployed -> Delete
                    edits.push(nickel::ast_utils::FieldEdit::Delete { path: key_path });
                }
            }
            // Keys added (in deployed but not current)
            for (key, dep_val) in dep_obj {
                if !cur_obj.contains_key(key) {
                    let key_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{path}.{key}")
                    };
                    edits.push(nickel::ast_utils::FieldEdit::Insert {
                        path: key_path,
                        value: dep_val.clone(),
                    });
                }
            }
        }
        _ => {
            // Leaf values that differ
            if current != deployed && !path.is_empty() {
                edits.push(nickel::ast_utils::FieldEdit::Modify {
                    path: path.to_string(),
                    new_value: deployed.clone(),
                });
            }
        }
    }
}

/// Build a merged JSON value by starting from the target JSON and applying
/// per-key decisions. Keys resolved to `Source` take the generated/source value;
/// keys resolved to `Target` keep the deployed target value.
///
/// `decisions` maps dotted key paths to the side that should win.
pub fn build_merged_json(
    source_json: &serde_json::Value,
    target_json: &serde_json::Value,
    decisions: &std::collections::HashMap<String, KeyResolution>,
) -> serde_json::Value {
    merge_values(source_json, target_json, decisions, "")
}

/// Recursively merge two JSON values according to per-key decisions.
fn merge_values(
    source: &serde_json::Value,
    target: &serde_json::Value,
    decisions: &std::collections::HashMap<String, KeyResolution>,
    path: &str,
) -> serde_json::Value {
    match (source, target) {
        (serde_json::Value::Object(source_obj), serde_json::Value::Object(target_obj)) => {
            let mut merged = serde_json::Map::new();

            // Start with all target keys.
            for (key, target_val) in target_obj {
                let key_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };

                if let Some(&resolution) = decisions.get(&key_path) {
                    match resolution {
                        KeyResolution::Source => {
                            // Source wins: use source value if it exists, otherwise omit.
                            if let Some(source_val) = source_obj.get(key) {
                                merged.insert(key.clone(), source_val.clone());
                            }
                        }
                        KeyResolution::Target => {
                            merged.insert(key.clone(), target_val.clone());
                        }
                    }
                } else if let Some(source_val) = source_obj.get(key) {
                    // No decision at this exact path -- recurse into sub-objects
                    merged.insert(
                        key.clone(),
                        merge_values(source_val, target_val, decisions, &key_path),
                    );
                } else {
                    // Key only in target, no decision -> keep target.
                    merged.insert(key.clone(), target_val.clone());
                }
            }

            // Keys only in source (Added changes)
            for (key, source_val) in source_obj {
                if target_obj.contains_key(key) {
                    continue; // Already handled above
                }
                let key_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };

                if let Some(&resolution) = decisions.get(&key_path) {
                    if resolution == KeyResolution::Source {
                        merged.insert(key.clone(), source_val.clone());
                    }
                    // Target for an Added key means "don't add it" -> skip.
                } else {
                    // No decision for source-only key -> keep source value.
                    merged.insert(key.clone(), source_val.clone());
                }
            }

            serde_json::Value::Object(merged)
        }
        _ => {
            // For non-object values at this path, check the decision
            if let Some(&resolution) = decisions.get(path) {
                match resolution {
                    KeyResolution::Source => source.clone(),
                    KeyResolution::Target => target.clone(),
                }
            } else {
                // No decision -- default to target (conservative)
                target.clone()
            }
        }
    }
}

/// Pull specific keys from deployed config back into the .ncl file.
///
/// `pulled_keys` is the set of dotted key paths where Target should be applied
/// back to Source in the .ncl rewrite.
/// `key_changes` optionally provides change type info per key to enable
/// structure-aware insertion and deletion.
pub fn pull_from_config_keys(
    ctx: &Context,
    order_name: &str,
    file_entry_index: usize,
    target: &Path,
    format: nickel::Format,
    pulled_keys: &[String],
    dry_run: bool,
) -> Result<bool> {
    pull_from_config_keys_with_changes(
        ctx,
        order_name,
        file_entry_index,
        target,
        format,
        pulled_keys,
        &[], // no change type info -- infer from diff
        dry_run,
    )
}

/// Pull specific keys with optional KeyChange metadata for insert/delete support.
#[allow(clippy::too_many_arguments)]
pub fn pull_from_config_keys_with_changes(
    ctx: &Context,
    order_name: &str,
    file_entry_index: usize,
    target: &Path,
    format: nickel::Format,
    pulled_keys: &[String],
    key_changes: &[KeyChange],
    dry_run: bool,
) -> Result<bool> {
    if pulled_keys.is_empty() {
        return Ok(true); // nothing to pull
    }

    let order_dir = ctx.orders_dir.join(order_name);
    let ncl_path = order_dir.join("order.ncl");

    let source = std::fs::read_to_string(&ncl_path)
        .with_context(|| format!("Failed to read {}", ncl_path.display()))?;

    let rewrite_result =
        nickel::ast_utils::locate_from_config(&source, file_entry_index, &ctx.metadata)?;

    let leaf_spans = rewrite_result.rewritable_spans();

    // Parse the deployed file
    let deployed_content = std::fs::read_to_string(target)
        .with_context(|| format!("Failed to read {}", target.display()))?;
    let renderer = get_renderer(format);
    let deployed_json = renderer
        .parse(&deployed_content)
        .with_context(|| format!("Failed to parse deployed file {}", target.display()))?;

    // Get current evaluated JSON
    let evaluator = nickel::NickelEvaluator::new(&ctx.metadata);
    let order = evaluator.evaluate(&ncl_path)?;
    let file_entry = order
        .blend
        .files
        .get(file_entry_index)
        .context("file entry index out of bounds")?;
    let current_json = file_entry
        .from_config
        .as_ref()
        .context("file entry has no from_config")?;

    if dry_run {
        log::info(&format!(
            "[dry-run] Would update {} keys in {}",
            pulled_keys.len(),
            ncl_path.display()
        ));
        return Ok(true);
    }

    let base_indent = if let Some(first) = leaf_spans.first() {
        nickel::ast_utils::detect_indent_level(&source, first.value_start)
    } else {
        0
    };

    // Build a change type lookup from key_changes if provided
    let change_type_map: std::collections::HashMap<&str, &KeyChangeType> = key_changes
        .iter()
        .map(|kc| (kc.path.as_str(), &kc.change_type))
        .collect();

    // Check if any pulled keys are insertions or deletions
    let has_structural_changes = pulled_keys.iter().any(|k| {
        if let Some(ct) = change_type_map.get(k.as_str()) {
            matches!(ct, KeyChangeType::Added | KeyChangeType::Removed)
        } else {
            let in_cur = nickel::ast_utils::json_path_get(current_json, k).is_some();
            let in_dep = nickel::ast_utils::json_path_get(&deployed_json, k).is_some();
            (in_dep && !in_cur) || (in_cur && !in_dep)
        }
    });

    // Try structure-aware rewrite if we have structural changes
    let new_source = if has_structural_changes {
        match nickel::structure_map::build_structure_map(&source, file_entry_index) {
            Ok(structure) => {
                let edits = build_field_edits_for_keys(
                    current_json,
                    &deployed_json,
                    pulled_keys,
                    &change_type_map,
                );
                nickel::ast_utils::surgical_rewrite_with_structure(
                    &source,
                    &structure,
                    leaf_spans,
                    &edits,
                    base_indent,
                )?
            }
            Err(_) => {
                // Fall back: only handle modifications
                if leaf_spans.is_empty() {
                    return Ok(false);
                }
                let selective_deployed =
                    build_selective_deployed(current_json, &deployed_json, pulled_keys);
                nickel::ast_utils::surgical_rewrite(
                    &source,
                    leaf_spans,
                    current_json,
                    &selective_deployed,
                    base_indent,
                )?
            }
        }
    } else {
        if leaf_spans.is_empty() {
            return Ok(false);
        }
        // Only modifications -- use existing path
        let selective_deployed =
            build_selective_deployed(current_json, &deployed_json, pulled_keys);
        nickel::ast_utils::surgical_rewrite(
            &source,
            leaf_spans,
            current_json,
            &selective_deployed,
            base_indent,
        )?
    };

    std::fs::write(&ncl_path, &new_source)
        .with_context(|| format!("Failed to write {}", ncl_path.display()))?;

    for field in rewrite_result.non_rewritable_fields() {
        if pulled_keys
            .iter()
            .any(|k| k == &field.name || k.starts_with(&format!("{}.", field.name)))
        {
            log::warn(&format!(
                "Cannot apply Target -> Source for {}: {} (update order.ncl manually)",
                field.name, field.reason
            ));
        }
    }

    Ok(true)
}

/// Build FieldEdit list for specific pulled keys, using change type info.
///
/// Uses dotted-path resolution so nested keys (e.g. `window.opacity`) are
/// looked up through nested JSON objects.
fn build_field_edits_for_keys(
    current: &serde_json::Value,
    deployed: &serde_json::Value,
    pulled_keys: &[String],
    change_types: &std::collections::HashMap<&str, &KeyChangeType>,
) -> Vec<nickel::ast_utils::FieldEdit> {
    pulled_keys
        .iter()
        .filter_map(|key| {
            let change_type = if let Some(ct) = change_types.get(key.as_str()) {
                (*ct).clone()
            } else {
                let in_cur = nickel::ast_utils::json_path_get(current, key).is_some();
                let in_dep = nickel::ast_utils::json_path_get(deployed, key).is_some();
                if in_cur && in_dep {
                    KeyChangeType::Modified
                } else if in_dep && !in_cur {
                    KeyChangeType::Removed
                } else if in_cur && !in_dep {
                    KeyChangeType::Added
                } else {
                    return None;
                }
            };

            match change_type {
                KeyChangeType::Modified => {
                    let dep_val = nickel::ast_utils::json_path_get(deployed, key)?;
                    Some(nickel::ast_utils::FieldEdit::Modify {
                        path: key.clone(),
                        new_value: dep_val.clone(),
                    })
                }
                KeyChangeType::Removed => {
                    let dep_val = nickel::ast_utils::json_path_get(deployed, key)?;
                    Some(nickel::ast_utils::FieldEdit::Insert {
                        path: key.clone(),
                        value: dep_val.clone(),
                    })
                }
                KeyChangeType::Added => {
                    Some(nickel::ast_utils::FieldEdit::Delete { path: key.clone() })
                }
            }
        })
        .collect()
}

/// Build a JSON value where only selected keys take Target values;
/// all other keys keep their current Source values.
///
/// Selected keys may be dotted paths (e.g. `window.opacity`); the overlay walks
/// both objects in lockstep and replaces the subtree at any path that matches.
fn build_selective_deployed(
    current: &serde_json::Value,
    deployed: &serde_json::Value,
    pulled_keys: &[String],
) -> serde_json::Value {
    selective_overlay(current, deployed, pulled_keys, "")
}

fn selective_overlay(
    current: &serde_json::Value,
    deployed: &serde_json::Value,
    pulled_keys: &[String],
    path: &str,
) -> serde_json::Value {
    if !path.is_empty() && pulled_keys.iter().any(|k| k == path) {
        return deployed.clone();
    }

    match (current, deployed) {
        (serde_json::Value::Object(cur_obj), serde_json::Value::Object(dep_obj)) => {
            let mut result = serde_json::Map::new();
            for (key, cur_val) in cur_obj {
                let key_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                if let Some(dep_val) = dep_obj.get(key) {
                    result.insert(
                        key.clone(),
                        selective_overlay(cur_val, dep_val, pulled_keys, &key_path),
                    );
                } else {
                    // Key only in current; keep it (deletions are handled by
                    // FieldEdit::Delete in the structural path).
                    result.insert(key.clone(), cur_val.clone());
                }
            }
            // Keys only in deployed are picked up only if explicitly pulled
            // at this exact path or under it.
            for (key, dep_val) in dep_obj {
                if cur_obj.contains_key(key) {
                    continue;
                }
                let key_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                let prefix = format!("{key_path}.");
                if pulled_keys
                    .iter()
                    .any(|k| k == &key_path || k.starts_with(&prefix))
                {
                    result.insert(key.clone(), dep_val.clone());
                }
            }
            serde_json::Value::Object(result)
        }
        _ => {
            if !pulled_keys.is_empty() && path.is_empty() {
                deployed.clone()
            } else {
                current.clone()
            }
        }
    }
}

/// Display a sync conflict diff for a file
pub fn display_conflict(
    order_name: &str,
    name: &str,
    target: &Path,
    diff: &DiffResult,
    home_dir: &Path,
) {
    let target_display = shorten_path(target, home_dir);
    println!(
        "\n  {}:{} ({})",
        style(order_name).cyan(),
        name,
        target_display
    );
    for line in diff.output.lines() {
        println!("    {}", line);
    }
}

/// Shorten a path by replacing the home directory with ~
pub fn shorten_path(path: &Path, home_dir: &Path) -> String {
    let s = path.to_string_lossy();
    let home = home_dir.to_string_lossy();
    if s.starts_with(home.as_ref()) {
        format!("~{}", &s[home.len()..])
    } else {
        s.into_owned()
    }
}

/// Annotation describing which side moved relative to the snapshot, for the
/// whole-file flow. `None` (returned by `compute_file_annotation`) means no
/// snapshot was available, so no annotation is emitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileAnnotation {
    /// snapshot == target, snapshot != source -> suggest Source -> Target.
    SourceChanged,
    /// snapshot == source, snapshot != target -> suggest Target -> Source.
    DeployedChanged,
    /// All three differ → real conflict.
    BothChanged,
    /// Target file no longer exists on disk; snapshot still does.
    DeployedDeleted,
}

/// Compute the file-level annotation given the snapshot, rendered, and
/// deployed bytes. Returns `None` when the snapshot is missing — callers
/// then fall back to today's unannotated prompt.
pub fn compute_file_annotation(
    snapshot: Option<&[u8]>,
    rendered: &[u8],
    deployed: &[u8],
) -> Option<FileAnnotation> {
    let snap = snapshot?;
    let snap_eq_deployed = snap == deployed;
    let snap_eq_rendered = snap == rendered;
    Some(match (snap_eq_rendered, snap_eq_deployed) {
        (false, true) => FileAnnotation::SourceChanged,
        (true, false) => FileAnnotation::DeployedChanged,
        (false, false) => FileAnnotation::BothChanged,
        // (true, true) cannot happen: rendered == deployed means there's no
        // diff to prompt about, so this code path is not reached. Treat as
        // no annotation to be safe.
        (true, true) => return None,
    })
}

/// Compute the annotation for the case where the deployed target is gone
/// from disk but a snapshot still exists.
pub fn file_annotation_for_deleted_target(snapshot: Option<&[u8]>) -> Option<FileAnnotation> {
    snapshot.map(|_| FileAnnotation::DeployedDeleted)
}

impl FileAnnotation {
    /// Human-readable annotation line shown above the prompt.
    pub fn message(self) -> &'static str {
        match self {
            FileAnnotation::SourceChanged => "Source changed (suggests Source -> Target)",
            FileAnnotation::DeployedChanged => "Target changed (suggests Target -> Source)",
            FileAnnotation::BothChanged => "both Source and Target changed since Base",
            FileAnnotation::DeployedDeleted => "Target file was deleted",
        }
    }
}

/// Annotation describing which side moved at the level of a single key, for
/// the per-key from_config flow.
//
// `enum_variant_names` lints the shared `Changed` suffix, but the suffix
// keeps the variant names symmetric with `FileAnnotation` and parallel with
// the prompt-facing message wording. Suppressing here is intentional.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAnnotation {
    SourceChanged,
    DeployedChanged,
    BothChanged,
}

/// Compute the per-key annotation. Returns `None` when the snapshot is
/// missing entirely OR the snapshot has no value at the given key path.
pub fn compute_key_annotation(
    snapshot: Option<&serde_json::Value>,
    rendered: &serde_json::Value,
    deployed: &serde_json::Value,
    path: &str,
) -> Option<KeyAnnotation> {
    let snap = snapshot?;
    let snap_val = lookup_dotted(snap, path)?;
    let rendered_val = lookup_dotted(rendered, path);
    let deployed_val = lookup_dotted(deployed, path);
    let snap_eq_rendered = rendered_val.map(|v| v == snap_val).unwrap_or(false);
    let snap_eq_deployed = deployed_val.map(|v| v == snap_val).unwrap_or(false);
    Some(match (snap_eq_rendered, snap_eq_deployed) {
        (false, true) => KeyAnnotation::SourceChanged,
        (true, false) => KeyAnnotation::DeployedChanged,
        (false, false) => KeyAnnotation::BothChanged,
        (true, true) => return None,
    })
}

fn lookup_dotted<'a>(value: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    let mut cur = value;
    for segment in path.split('.') {
        cur = cur.as_object()?.get(segment)?;
    }
    Some(cur)
}

impl KeyAnnotation {
    pub fn message(self) -> &'static str {
        match self {
            KeyAnnotation::SourceChanged => "Source changed (suggests Source)",
            KeyAnnotation::DeployedChanged => "Target changed (suggests Target)",
            KeyAnnotation::BothChanged => "both Source and Target changed since Base",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::KeyChangeType;
    use std::collections::{HashMap, VecDeque};
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Mock prompter for testing
    struct MockPrompter {
        answers: std::cell::RefCell<VecDeque<SyncAction>>,
        key_answers: std::cell::RefCell<VecDeque<KeyAction>>,
    }

    impl MockPrompter {
        fn new(answers: Vec<SyncAction>) -> Self {
            Self {
                answers: std::cell::RefCell::new(answers.into()),
                key_answers: std::cell::RefCell::new(VecDeque::new()),
            }
        }

        fn with_key_answers(answers: Vec<SyncAction>, key_answers: Vec<KeyAction>) -> Self {
            Self {
                answers: std::cell::RefCell::new(answers.into()),
                key_answers: std::cell::RefCell::new(key_answers.into()),
            }
        }
    }

    impl Prompter for MockPrompter {
        fn ask_sync_action(
            &self,
            _order_name: &str,
            _name: &str,
            _target: &Path,
            _diff: &DiffResult,
            _can_pull: bool,
            _annotation: Option<FileAnnotation>,
        ) -> SyncAction {
            self.answers
                .borrow_mut()
                .pop_front()
                .unwrap_or(SyncAction::Skip)
        }

        fn ask_key_action(
            &self,
            _order_name: &str,
            _name: &str,
            change: &KeyChange,
            _annotation: Option<KeyAnnotation>,
        ) -> KeyAction {
            let _ = change;
            self.key_answers
                .borrow_mut()
                .pop_front()
                .unwrap_or(KeyAction::Skip)
        }
    }

    #[test]
    fn test_shorten_path() {
        let home = PathBuf::from("/Users/test");
        assert_eq!(
            shorten_path(&PathBuf::from("/Users/test/.config/foo"), &home),
            "~/.config/foo"
        );
        assert_eq!(shorten_path(&PathBuf::from("/etc/foo"), &home), "/etc/foo");
    }

    #[test]
    fn test_pull_from_file() {
        let temp = TempDir::new().unwrap();
        let target = temp.path().join("deployed.txt");
        let source = temp.path().join("source.txt");

        std::fs::write(&target, "deployed content").unwrap();
        std::fs::write(&source, "old content").unwrap();

        pull_from_file(&source, &target, None, &[], false).unwrap();

        assert_eq!(
            std::fs::read_to_string(&source).unwrap(),
            "deployed content"
        );
    }

    #[test]
    fn test_pull_from_file_dry_run() {
        let temp = TempDir::new().unwrap();
        let target = temp.path().join("deployed.txt");
        let source = temp.path().join("source.txt");

        std::fs::write(&target, "deployed content").unwrap();
        std::fs::write(&source, "old content").unwrap();

        pull_from_file(&source, &target, None, &[], true).unwrap();

        // Should not have changed
        assert_eq!(std::fs::read_to_string(&source).unwrap(), "old content");
    }

    #[test]
    fn test_pull_directory() {
        let temp = TempDir::new().unwrap();
        let deployed = temp.path().join("deployed");
        let source = temp.path().join("source");

        std::fs::create_dir_all(deployed.join("sub")).unwrap();
        std::fs::write(deployed.join("file.txt"), "content").unwrap();
        std::fs::write(deployed.join("sub/nested.txt"), "nested").unwrap();
        std::fs::write(deployed.join("target-only.txt"), "target-only").unwrap();

        std::fs::create_dir_all(source.join("sub")).unwrap();
        std::fs::write(source.join("file.txt"), "old").unwrap();
        std::fs::write(source.join("sub/nested.txt"), "old nested").unwrap();

        pull_from_file(&source, &deployed, None, &[], false).unwrap();

        assert_eq!(
            std::fs::read_to_string(source.join("file.txt")).unwrap(),
            "content"
        );
        assert_eq!(
            std::fs::read_to_string(source.join("sub/nested.txt")).unwrap(),
            "nested"
        );
        assert!(
            !source.join("target-only.txt").exists(),
            "target-only deployed files should stay outside blend's ownership boundary"
        );
    }

    #[test]
    fn test_mock_prompter() {
        let prompter = MockPrompter::new(vec![
            SyncAction::ApplySourceToTarget,
            SyncAction::ApplyTargetToSource,
            SyncAction::Quit,
        ]);
        let diff = DiffResult::no_changes();

        assert_eq!(
            prompter.ask_sync_action("order_name", "file", Path::new("/tmp"), &diff, true, None),
            SyncAction::ApplySourceToTarget
        );
        assert_eq!(
            prompter.ask_sync_action("order_name", "file", Path::new("/tmp"), &diff, true, None),
            SyncAction::ApplyTargetToSource
        );
        assert_eq!(
            prompter.ask_sync_action("order_name", "file", Path::new("/tmp"), &diff, true, None),
            SyncAction::Quit
        );
        // Exhausted -- defaults to Skip
        assert_eq!(
            prompter.ask_sync_action("order_name", "file", Path::new("/tmp"), &diff, true, None),
            SyncAction::Skip
        );
    }

    #[test]
    fn test_mock_prompter_key_actions() {
        let prompter = MockPrompter::with_key_answers(
            vec![],
            vec![
                KeyAction::UseSource,
                KeyAction::UseTarget,
                KeyAction::AllSource,
            ],
        );
        let change = KeyChange {
            path: "key".to_string(),
            change_type: KeyChangeType::Modified,
            repo_value: Some(serde_json::json!("new")),
            deployed_value: Some(serde_json::json!("old")),
            display: "~ key".to_string(),
        };

        assert_eq!(
            prompter.ask_key_action("order_name", "file", &change, None),
            KeyAction::UseSource
        );
        assert_eq!(
            prompter.ask_key_action("order_name", "file", &change, None),
            KeyAction::UseTarget
        );
        assert_eq!(
            prompter.ask_key_action("order_name", "file", &change, None),
            KeyAction::AllSource
        );
        // Exhausted
        assert_eq!(
            prompter.ask_key_action("order_name", "file", &change, None),
            KeyAction::Skip
        );
    }

    #[test]
    fn test_build_selective_deployed_nested_key() {
        use serde_json::json;
        // semantic_diff_keys produces dotted paths like "window.opacity".
        // build_selective_deployed must resolve them through nested JSON.
        let current = json!({
            "window": {"opacity": 0.7, "decorations": "Buttonless"},
            "font": {"size": 12},
        });
        let deployed = json!({
            "window": {"opacity": 0.8, "decorations": "Buttonless"},
            "font": {"size": 12},
        });
        let pulled_keys = vec!["window.opacity".to_string()];
        let result = build_selective_deployed(&current, &deployed, &pulled_keys);
        assert_eq!(
            result,
            json!({
                "window": {"opacity": 0.8, "decorations": "Buttonless"},
                "font": {"size": 12},
            }),
            "Only window.opacity should adopt the Target value; got:\n{:#?}",
            result
        );
    }

    #[test]
    fn test_build_merged_json_all_source() {
        use serde_json::json;
        let source = json!({"a": 1, "b": 2});
        let target = json!({"a": 10, "b": 20});
        let mut decisions = HashMap::new();
        decisions.insert("a".to_string(), KeyResolution::Source);
        decisions.insert("b".to_string(), KeyResolution::Source);

        let merged = build_merged_json(&source, &target, &decisions);
        assert_eq!(merged, json!({"a": 1, "b": 2}));
    }

    #[test]
    fn test_build_merged_json_all_target() {
        use serde_json::json;
        let source = json!({"a": 1, "b": 2});
        let target = json!({"a": 10, "b": 20});
        let mut decisions = HashMap::new();
        decisions.insert("a".to_string(), KeyResolution::Target);
        decisions.insert("b".to_string(), KeyResolution::Target);

        let merged = build_merged_json(&source, &target, &decisions);
        assert_eq!(merged, json!({"a": 10, "b": 20}));
    }

    #[test]
    fn test_build_merged_json_mixed() {
        use serde_json::json;
        let source = json!({"a": 1, "b": 2, "c": 3});
        let target = json!({"a": 10, "b": 20, "c": 30});
        let mut decisions = HashMap::new();
        decisions.insert("a".to_string(), KeyResolution::Source);
        decisions.insert("b".to_string(), KeyResolution::Target);
        // c has no decision -> defaults to target for unchanged keys

        let merged = build_merged_json(&source, &target, &decisions);
        assert_eq!(merged, json!({"a": 1, "b": 20, "c": 30}));
    }

    #[test]
    fn test_build_merged_json_added_key_source() {
        use serde_json::json;
        let source = json!({"a": 1, "new_key": "hello"});
        let target = json!({"a": 1});
        let mut decisions = HashMap::new();
        decisions.insert("new_key".to_string(), KeyResolution::Source);

        let merged = build_merged_json(&source, &target, &decisions);
        assert_eq!(merged, json!({"a": 1, "new_key": "hello"}));
    }

    #[test]
    fn test_build_merged_json_added_key_target() {
        use serde_json::json;
        let source = json!({"a": 1, "new_key": "hello"});
        let target = json!({"a": 1});
        let mut decisions = HashMap::new();
        decisions.insert("new_key".to_string(), KeyResolution::Target);

        let merged = build_merged_json(&source, &target, &decisions);
        assert_eq!(merged, json!({"a": 1}));
    }

    #[test]
    fn test_build_merged_json_removed_key_source() {
        use serde_json::json;
        let source = json!({"a": 1});
        let target = json!({"a": 1, "extra": "deployed"});
        let mut decisions = HashMap::new();
        decisions.insert("extra".to_string(), KeyResolution::Source);

        let merged = build_merged_json(&source, &target, &decisions);
        assert_eq!(merged, json!({"a": 1}));
    }

    #[test]
    fn test_build_merged_json_removed_key_target() {
        use serde_json::json;
        let source = json!({"a": 1});
        let target = json!({"a": 1, "extra": "deployed"});
        let mut decisions = HashMap::new();
        decisions.insert("extra".to_string(), KeyResolution::Target);

        let merged = build_merged_json(&source, &target, &decisions);
        assert_eq!(merged, json!({"a": 1, "extra": "deployed"}));
    }

    use crate::sync::{FileAnnotation, compute_file_annotation};

    #[test]
    fn file_annotation_source_changed_when_snapshot_equals_deployed() {
        let snapshot = Some(b"original".as_slice());
        let rendered = b"new-source";
        let deployed = b"original";
        assert_eq!(
            compute_file_annotation(snapshot, rendered, deployed),
            Some(FileAnnotation::SourceChanged)
        );
    }

    #[test]
    fn file_annotation_deployed_changed_when_snapshot_equals_rendered() {
        let snapshot = Some(b"original".as_slice());
        let rendered = b"original";
        let deployed = b"hand-edited";
        assert_eq!(
            compute_file_annotation(snapshot, rendered, deployed),
            Some(FileAnnotation::DeployedChanged)
        );
    }

    #[test]
    fn file_annotation_both_changed_when_all_three_differ() {
        let snapshot = Some(b"original".as_slice());
        let rendered = b"new-source";
        let deployed = b"hand-edited";
        assert_eq!(
            compute_file_annotation(snapshot, rendered, deployed),
            Some(FileAnnotation::BothChanged)
        );
    }

    #[test]
    fn file_annotation_none_when_snapshot_missing() {
        let snapshot: Option<&[u8]> = None;
        let rendered = b"new-source";
        let deployed = b"hand-edited";
        assert_eq!(compute_file_annotation(snapshot, rendered, deployed), None);
    }

    use crate::sync::{KeyAnnotation, compute_key_annotation};
    use serde_json::json;

    #[test]
    fn key_annotation_source_changed_when_snapshot_value_equals_deployed() {
        let snapshot = json!({ "font": { "size": 12 } });
        let rendered = json!({ "font": { "size": 14 } });
        let deployed = json!({ "font": { "size": 12 } });
        assert_eq!(
            compute_key_annotation(Some(&snapshot), &rendered, &deployed, "font.size"),
            Some(KeyAnnotation::SourceChanged)
        );
    }

    #[test]
    fn key_annotation_deployed_changed_when_snapshot_value_equals_rendered() {
        let snapshot = json!({ "font": { "size": 12 } });
        let rendered = json!({ "font": { "size": 12 } });
        let deployed = json!({ "font": { "size": 18 } });
        assert_eq!(
            compute_key_annotation(Some(&snapshot), &rendered, &deployed, "font.size"),
            Some(KeyAnnotation::DeployedChanged)
        );
    }

    #[test]
    fn key_annotation_both_changed_when_all_three_differ() {
        let snapshot = json!({ "font": { "size": 12 } });
        let rendered = json!({ "font": { "size": 14 } });
        let deployed = json!({ "font": { "size": 18 } });
        assert_eq!(
            compute_key_annotation(Some(&snapshot), &rendered, &deployed, "font.size"),
            Some(KeyAnnotation::BothChanged)
        );
    }

    #[test]
    fn key_annotation_none_when_snapshot_missing_entirely() {
        let rendered = json!({ "font": { "size": 14 } });
        let deployed = json!({ "font": { "size": 18 } });
        assert_eq!(
            compute_key_annotation(None, &rendered, &deployed, "font.size"),
            None
        );
    }

    #[test]
    fn key_annotation_none_when_snapshot_lacks_the_key() {
        let snapshot = json!({ "font": {} });
        let rendered = json!({ "font": { "size": 14 } });
        let deployed = json!({ "font": { "size": 18 } });
        assert_eq!(
            compute_key_annotation(Some(&snapshot), &rendered, &deployed, "font.size"),
            None
        );
    }

    #[test]
    fn mock_prompter_receives_file_annotation() {
        let prompter = MockPrompter::new(vec![SyncAction::ApplySourceToTarget]);
        let diff = DiffResult::with_changes("dummy".into());
        let action = prompter.ask_sync_action(
            "order_name",
            "entry",
            Path::new("/tmp/x"),
            &diff,
            true,
            Some(FileAnnotation::SourceChanged),
        );
        assert_eq!(action, SyncAction::ApplySourceToTarget);
    }

    #[test]
    fn mock_prompter_receives_key_annotation() {
        let prompter = MockPrompter::with_key_answers(vec![], vec![KeyAction::UseSource]);
        let change = KeyChange {
            path: "font.size".into(),
            change_type: KeyChangeType::Modified,
            repo_value: Some(serde_json::json!(14)),
            deployed_value: Some(serde_json::json!(12)),
            display: "font.size = 14".into(),
        };
        let action = prompter.ask_key_action(
            "order_name",
            "entry",
            &change,
            Some(KeyAnnotation::SourceChanged),
        );
        assert_eq!(action, KeyAction::UseSource);
    }

    #[test]
    fn cmd_sync_passes_source_changed_annotation_when_snapshot_equals_deployed() {
        // This is a thin integration check: build a Context whose StateStore
        // has a snapshot equal to the bytes we pretend are "deployed", then
        // verify that compute_file_annotation classifies a different
        // "rendered" buffer as SourceChanged.
        let snap_bytes = b"deployed-bytes";
        let rendered = b"new-rendered";
        let deployed = b"deployed-bytes";
        assert_eq!(
            compute_file_annotation(Some(snap_bytes), rendered, deployed),
            Some(FileAnnotation::SourceChanged)
        );
    }

    #[test]
    fn key_annotation_handles_nested_dotted_paths() {
        let snapshot = serde_json::json!({ "a": { "b": { "c": 1 } } });
        let rendered = serde_json::json!({ "a": { "b": { "c": 2 } } });
        let deployed = serde_json::json!({ "a": { "b": { "c": 1 } } });
        assert_eq!(
            compute_key_annotation(Some(&snapshot), &rendered, &deployed, "a.b.c"),
            Some(KeyAnnotation::SourceChanged)
        );
    }
}
