use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::{Context as AnyhowContext, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use walkdir::WalkDir;

use crate::context::Context;
use crate::formats::get_renderer;
use crate::nickel::{FileEntry, Format, NickelEvaluator, Order};
use crate::output::log;

/// Discover orders in the orders directory
pub fn discover_orders(orders_dir: &Path) -> HashSet<String> {
    let mut orders = HashSet::new();

    let Ok(entries) = std::fs::read_dir(orders_dir) else {
        return orders;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        // Skip hidden directories
        if name.starts_with('.') {
            continue;
        }

        // Check for order.ncl
        if path.join("order.ncl").exists() {
            orders.insert(name.to_string());
        }
    }

    orders
}

/// Result of building a single file entry target
#[derive(Debug)]
pub struct BuildResult {
    /// Target path (expanded)
    pub target: PathBuf,
    /// Rendered content (empty for plaintext)
    pub content: String,
    /// Whether this is a plaintext copy
    pub is_plaintext: bool,
    /// Source path for plaintext copies
    pub source_path: Option<PathBuf>,
    /// Name from FileEntry
    pub name: String,
    /// Effective format from FileEntry (explicit or inferred)
    pub format: Format,
    /// Merged ignore keys (global + per-file)
    pub ignore_keys: Vec<String>,
    /// Whether this entry should be symlinked instead of copied
    pub is_symlink: bool,
    /// Canonical (absolute) source path for symlink entries
    pub canonical_source: Option<PathBuf>,
    /// Glob patterns to exclude when shipping a directory
    pub exclude_patterns: Vec<String>,
    /// Path to the local overlay directory (if set and source is a directory)
    pub local_dir: Option<PathBuf>,
    /// Whether to set the OS immutable flag on the deployed file
    pub immutable: bool,
}

/// Build a single order, returning results for all file entries and targets
pub fn build_order(ctx: &Context, order_name: &str) -> Result<Vec<BuildResult>> {
    let order_dir = ctx.orders_dir.join(order_name);
    let ncl_path = order_dir.join("order.ncl");

    if !ncl_path.exists() {
        return Ok(vec![]);
    }

    let evaluator = NickelEvaluator::new(&ctx.metadata);
    let order = evaluator.evaluate(&ncl_path)?;

    // Check if order should be applied for this system
    if !order.should_apply(&ctx.metadata.os, &ctx.metadata.arch, &ctx.metadata.hostname) {
        return Ok(vec![]);
    }

    let mut results = Vec::new();
    let global_ignore = order.global_ignore();
    let global_prefix = order.global_prefix();

    for file_entry in &order.blend.files {
        // Check per-file condition
        if !file_entry.should_apply(&ctx.metadata.os, &ctx.metadata.arch, &ctx.metadata.hostname) {
            if ctx.verbose {
                log::info(&format!(
                    "Skipping file {} (when condition not met)",
                    file_entry.name,
                ));
            }
            continue;
        }

        // Merge ignore keys: global + per-file
        let mut ignore_keys: Vec<String> = global_ignore.to_vec();
        ignore_keys.extend(file_entry.ignore.iter().cloned());

        // Build for each target prefix (file-level overrides global)
        for target_path in file_entry.target_paths(global_prefix) {
            let expanded_target = ctx.expand_path(&target_path);
            let result =
                build_file_entry(&order_dir, file_entry, expanded_target, ignore_keys.clone())?;
            results.push(result);
        }
    }

    Ok(results)
}

/// Build a single file entry to a specific target (public wrapper)
pub fn build_file_entry_pub(
    _ctx: &Context,
    order_dir: &Path,
    entry: &FileEntry,
    target: PathBuf,
    ignore_keys: Vec<String>,
) -> Result<BuildResult> {
    build_file_entry(order_dir, entry, target, ignore_keys)
}

/// Build a single file entry to a specific target
fn build_file_entry(
    order_dir: &Path,
    entry: &FileEntry,
    target: PathBuf,
    ignore_keys: Vec<String>,
) -> Result<BuildResult> {
    if let Some(file) = &entry.from_file {
        let source_path = order_dir.join(file);
        if !source_path.exists() {
            return Err(anyhow::anyhow!(
                "File entry '{}': source file not found at {}",
                entry.name,
                source_path.display()
            ));
        }

        // Resolve local overlay directory
        let local_dir = if let Some(local) = &entry.local {
            let ld = order_dir.join(local);
            // Auto-create local dir if it doesn't exist
            if !ld.exists() {
                std::fs::create_dir_all(&ld).with_context(|| {
                    format!("Failed to create local overlay directory {}", ld.display())
                })?;
            }
            Some(ld)
        } else {
            None
        };

        if entry.symlink {
            let canonical = source_path.canonicalize().with_context(|| {
                format!(
                    "Failed to canonicalize source path {}",
                    source_path.display()
                )
            })?;
            return Ok(BuildResult {
                target,
                content: String::new(),
                is_plaintext: true,
                source_path: Some(source_path),
                name: entry.name.clone(),
                format: entry.effective_format(),
                ignore_keys,
                is_symlink: true,
                canonical_source: Some(canonical),
                exclude_patterns: entry.exclude.clone(),
                local_dir,
                immutable: entry.immutable,
            });
        }

        return Ok(BuildResult {
            target,
            content: String::new(),
            is_plaintext: true,
            source_path: Some(source_path),
            name: entry.name.clone(),
            format: entry.effective_format(),
            ignore_keys,
            is_symlink: false,
            canonical_source: None,
            exclude_patterns: entry.exclude.clone(),
            local_dir,
            immutable: entry.immutable,
        });
    }

    if let Some(config) = &entry.from_config {
        let format = entry.effective_format();
        let renderer = get_renderer(format);
        let content = renderer.render(config)?;

        return Ok(BuildResult {
            target,
            content,
            is_plaintext: false,
            source_path: None,
            name: entry.name.clone(),
            format,
            ignore_keys,
            is_symlink: false,
            canonical_source: None,
            exclude_patterns: vec![],
            local_dir: None,
            immutable: entry.immutable,
        });
    }

    // Unreachable after resolve_defaults validation
    Err(anyhow::anyhow!(
        "File entry '{}' has neither 'from_file' nor 'from_config'",
        entry.name,
    ))
}

/// If the target path is a symlink, remove it so we write a regular file
/// instead of writing through the symlink to its target.
/// Ensure a directory exists at the given path, removing any symlink that blocks it.
///
/// Walks up from the target path to find any path component that is a symlink
/// (broken or otherwise) and removes it before calling `create_dir_all`.
/// This handles the case where old stow/symlink deployments left behind symlinks
/// at directories that blend now needs to create as real directories.
fn ensure_dir(path: &Path) -> Result<()> {
    for ancestor in path.ancestors() {
        if ancestor == Path::new("") || ancestor == Path::new("/") {
            break;
        }
        match std::fs::symlink_metadata(ancestor) {
            Ok(meta) if meta.file_type().is_symlink() => {
                std::fs::remove_file(ancestor).with_context(|| {
                    format!(
                        "Failed to remove symlink blocking directory creation: {}",
                        ancestor.display()
                    )
                })?;
                break;
            }
            Ok(_) => {
                break;
            }
            Err(_) => {
                continue;
            }
        }
    }
    std::fs::create_dir_all(path)
        .with_context(|| format!("Failed to create directory {}", path.display()))?;
    Ok(())
}

fn remove_symlink_if_exists(path: &Path, dry_run: bool) -> Result<()> {
    match std::fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => {
            if dry_run {
                log::info(&format!("Would remove symlink {}", path.display()));
                return Ok(());
            }
            std::fs::remove_file(path)
                .with_context(|| format!("Failed to remove symlink {}", path.display()))?;
        }
        _ => {}
    }
    Ok(())
}

/// Remove OS immutable flag from a file so it can be overwritten.
fn remove_immutable_flag(path: &Path, warn_on_failure: bool) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        let status = std::process::Command::new("chflags")
            .arg("nouchg")
            .arg(path)
            .status()
            .with_context(|| format!("Failed to run chflags nouchg on {}", path.display()))?;
        if warn_on_failure && !status.success() {
            log::warn(&format!(
                "chflags nouchg failed on {} (exit {})",
                path.display(),
                status.code().unwrap_or(-1)
            ));
        }
    }

    #[cfg(target_os = "linux")]
    {
        let status = std::process::Command::new("chattr")
            .arg("-i")
            .arg(path)
            .status();
        match status {
            Ok(s) if warn_on_failure && !s.success() => {
                log::warn(&format!(
                    "chattr -i failed on {} (exit {}) — may require root",
                    path.display(),
                    s.code().unwrap_or(-1)
                ));
            }
            Err(e) if warn_on_failure => {
                log::warn(&format!(
                    "Failed to run chattr -i on {}: {} — may require root",
                    path.display(),
                    e
                ));
            }
            _ => {}
        }
    }

    Ok(())
}

/// Set OS immutable flag on a file to prevent modification.
fn set_immutable_flag(path: &Path) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        let status = std::process::Command::new("chflags")
            .arg("uchg")
            .arg(path)
            .status()
            .with_context(|| format!("Failed to run chflags uchg on {}", path.display()))?;
        if !status.success() {
            log::warn(&format!(
                "chflags uchg failed on {} (exit {})",
                path.display(),
                status.code().unwrap_or(-1)
            ));
        }
    }

    #[cfg(target_os = "linux")]
    {
        let status = std::process::Command::new("chattr")
            .arg("+i")
            .arg(path)
            .status();
        match status {
            Ok(s) if !s.success() => {
                log::warn(&format!(
                    "chattr +i failed on {} (exit {}) — may require root",
                    path.display(),
                    s.code().unwrap_or(-1)
                ));
            }
            Err(e) => {
                log::warn(&format!(
                    "Failed to run chattr +i on {}: {} — may require root",
                    path.display(),
                    e
                ));
            }
            _ => {}
        }
    }

    Ok(())
}

/// Set immutable flags on all files within a directory.
fn set_immutable_flag_recursive(dir: &Path) -> Result<()> {
    for entry in WalkDir::new(dir).min_depth(1) {
        let entry = entry?;
        if entry.file_type().is_file() {
            set_immutable_flag(entry.path())?;
        }
    }
    Ok(())
}

/// Remove immutable flags from all files within a directory.
fn remove_immutable_flag_recursive(dir: &Path, warn_on_failure: bool) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in WalkDir::new(dir).min_depth(1) {
        let entry = entry?;
        if entry.file_type().is_file() {
            remove_immutable_flag(entry.path(), warn_on_failure)?;
        }
    }
    Ok(())
}

fn target_is_real_dir(path: &Path) -> bool {
    std::fs::symlink_metadata(path)
        .map(|meta| meta.is_dir() && !meta.file_type().is_symlink())
        .unwrap_or(false)
}

/// Write build result to target
pub fn write_result(result: &BuildResult, dry_run: bool) -> Result<()> {
    if result.is_symlink {
        if let Some(canonical) = &result.canonical_source {
            return create_symlink(canonical, &result.target, dry_run);
        }
        return Err(anyhow::anyhow!(
            "Symlink entry '{}' has no canonical source path",
            result.name
        ));
    }

    // Always try to clear an existing immutable target before writing. This
    // lets a config converge from `immutable = true` back to `false`.
    if result.target.exists() {
        if dry_run {
            if result.immutable {
                log::info(&format!(
                    "Would remove immutable flag from {}",
                    result.target.display()
                ));
            }
        } else if target_is_real_dir(&result.target) {
            remove_immutable_flag_recursive(&result.target, result.immutable)?;
        } else {
            remove_immutable_flag(&result.target, result.immutable)?;
        }
    }

    if result.is_plaintext {
        if let Some(source_path) = &result.source_path {
            if source_path.is_dir() {
                let exclude = build_glob_set(&result.exclude_patterns)?;
                copy_directory(
                    source_path,
                    &result.target,
                    result.local_dir.as_deref(),
                    exclude.as_ref(),
                    dry_run,
                )?;
            } else {
                copy_file(source_path, &result.target, dry_run)?;
            }
        }
    } else {
        if dry_run {
            remove_symlink_if_exists(&result.target, true)?;
            log::info(&format!("Would write to {}", result.target.display()));
            if result.immutable {
                log::info(&format!(
                    "Would set immutable flag on {}",
                    result.target.display()
                ));
            }
            return Ok(());
        }

        // Ensure parent directory exists
        if let Some(parent) = result.target.parent() {
            ensure_dir(parent)?;
        }

        remove_symlink_if_exists(&result.target, false)?;
        std::fs::write(&result.target, &result.content)
            .with_context(|| format!("Failed to write {}", result.target.display()))?;
    }

    // Set immutable flag after successful write
    if result.immutable && !dry_run {
        if result.target.is_dir() {
            set_immutable_flag_recursive(&result.target)?;
        } else {
            set_immutable_flag(&result.target)?;
        }
    }

    Ok(())
}

/// Create a symlink at target pointing to source
fn create_symlink(source: &Path, target: &Path, dry_run: bool) -> Result<()> {
    if dry_run {
        log::info(&format!(
            "Would symlink {} -> {}",
            target.display(),
            source.display()
        ));
        return Ok(());
    }

    // Ensure parent directory exists
    if let Some(parent) = target.parent() {
        ensure_dir(parent)?;
    }

    // Remove existing file, symlink, or directory at target
    if let Ok(meta) = std::fs::symlink_metadata(target) {
        if meta.is_dir() && !meta.file_type().is_symlink() {
            std::fs::remove_dir_all(target)
                .with_context(|| format!("Failed to remove directory {}", target.display()))?;
        } else {
            std::fs::remove_file(target)
                .with_context(|| format!("Failed to remove {}", target.display()))?;
        }
    }

    #[cfg(unix)]
    std::os::unix::fs::symlink(source, target).with_context(|| {
        format!(
            "Failed to create symlink {} -> {}",
            target.display(),
            source.display()
        )
    })?;

    #[cfg(not(unix))]
    return Err(anyhow::anyhow!(
        "Symlinks are only supported on Unix platforms"
    ));

    Ok(())
}

/// Copy a single file to target
fn copy_file(source: &Path, target: &Path, dry_run: bool) -> Result<()> {
    if dry_run {
        remove_symlink_if_exists(target, true)?;
        log::info(&format!(
            "Would copy {} to {}",
            source.display(),
            target.display()
        ));
        return Ok(());
    }

    // Ensure parent directory exists
    if let Some(parent) = target.parent() {
        ensure_dir(parent)?;
    }

    remove_symlink_if_exists(target, false)?;
    std::fs::copy(source, target).with_context(|| {
        format!(
            "Failed to copy {} to {}",
            source.display(),
            target.display()
        )
    })?;

    Ok(())
}

/// Build a GlobSet from a list of glob pattern strings.
/// Returns None if the list is empty.
pub fn build_glob_set(patterns: &[String]) -> Result<Option<GlobSet>> {
    if patterns.is_empty() {
        return Ok(None);
    }
    let mut builder = GlobSetBuilder::new();
    for pat in patterns {
        builder
            .add(Glob::new(pat).with_context(|| format!("Invalid exclude glob pattern: {pat}"))?);
    }
    Ok(Some(builder.build()?))
}

/// Represents a file in a merged directory view (tracked + local overlay).
#[derive(Debug, Clone)]
pub struct MergedFile {
    /// The actual file path on disk to read from
    pub source: PathBuf,
    /// Relative path within the directory
    pub rel_path: PathBuf,
    /// Whether this file comes from the local overlay
    pub is_local: bool,
}

/// Collect merged files from a source directory and optional local overlay,
/// with exclude filtering applied to the merged result.
pub fn collect_merged_files(
    source: &Path,
    local_dir: Option<&Path>,
    exclude: Option<&GlobSet>,
) -> Result<Vec<MergedFile>> {
    // Collect tracked files (no exclude yet -- we apply after merge)
    let mut tracked = HashMap::new();
    if source.exists() {
        for entry in WalkDir::new(source).min_depth(1) {
            let entry = entry?;
            if entry.file_type().is_dir() {
                continue;
            }
            let rel_path = entry.path().strip_prefix(source)?.to_path_buf();
            tracked.insert(rel_path, entry.path().to_path_buf());
        }
    }

    // Collect local overlay files
    let mut local_files = HashMap::new();
    if let Some(ld) = local_dir
        && ld.exists()
    {
        for entry in WalkDir::new(ld).min_depth(1) {
            let entry = entry?;
            if entry.file_type().is_dir() {
                continue;
            }
            let rel_path = entry.path().strip_prefix(ld)?.to_path_buf();
            local_files.insert(rel_path, entry.path().to_path_buf());
        }
    }

    // Merge: local overrides tracked
    let mut merged = Vec::new();
    let mut all_rel_paths: HashSet<PathBuf> = tracked.keys().cloned().collect();
    for k in local_files.keys() {
        all_rel_paths.insert(k.clone());
    }

    let mut sorted_paths: Vec<PathBuf> = all_rel_paths.into_iter().collect();
    sorted_paths.sort();

    for rel_path in sorted_paths {
        // Apply exclude to the merged result
        if let Some(gs) = exclude
            && gs.is_match(&rel_path)
        {
            continue;
        }

        if let Some(local_source) = local_files.get(&rel_path) {
            merged.push(MergedFile {
                source: local_source.clone(),
                rel_path,
                is_local: true,
            });
        } else if let Some(tracked_source) = tracked.get(&rel_path) {
            merged.push(MergedFile {
                source: tracked_source.clone(),
                rel_path,
                is_local: false,
            });
        }
    }

    Ok(merged)
}

/// Copy a source directory to target, with optional local overlay and exclude patterns.
fn copy_directory(
    source: &Path,
    target: &Path,
    local_dir: Option<&Path>,
    exclude: Option<&GlobSet>,
    dry_run: bool,
) -> Result<()> {
    if !source.exists() {
        return Err(anyhow::anyhow!(
            "Source directory does not exist: {}",
            source.display()
        ));
    }

    // If the top-level target is a symlink, remove it first
    remove_symlink_if_exists(target, dry_run)?;

    let merged = collect_merged_files(source, local_dir, exclude)?;

    for mf in &merged {
        let target_path = target.join(&mf.rel_path);

        if dry_run {
            let overlay_note = if mf.is_local { " (local)" } else { "" };
            log::info(&format!(
                "Would copy to {}{}",
                target_path.display(),
                overlay_note
            ));
            continue;
        }

        if let Some(parent) = target_path.parent() {
            ensure_dir(parent)?;
        }
        remove_symlink_if_exists(&target_path, false)?;
        std::fs::copy(&mf.source, &target_path)?;
    }

    Ok(())
}

/// Get the evaluated Order for an order
pub fn get_order(ctx: &Context, order_name: &str) -> Result<Order> {
    let order_dir = ctx.orders_dir.join(order_name);
    let ncl_path = order_dir.join("order.ncl");

    let evaluator = NickelEvaluator::new(&ctx.metadata);
    evaluator.evaluate(&ncl_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};
    use tempfile::TempDir;

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn test_discover_orders() {
        let temp = TempDir::new().unwrap();
        let orders = temp.path();

        // Create order with order.ncl
        let order1 = orders.join("order1");
        std::fs::create_dir(&order1).unwrap();
        std::fs::write(order1.join("order.ncl"), "{}").unwrap();

        // Create order without order.ncl
        let order2 = orders.join("order2");
        std::fs::create_dir(&order2).unwrap();

        let orders = discover_orders(orders);
        assert!(orders.contains("order1"));
        assert!(!orders.contains("order2"));
    }

    #[test]
    fn test_build_glob_set_empty() {
        let result = build_glob_set(&[]).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_build_glob_set_patterns() {
        let patterns = vec!["*.bak".to_string(), ".gitignore".to_string()];
        let gs = build_glob_set(&patterns).unwrap().unwrap();
        assert!(gs.is_match("file.bak"));
        assert!(gs.is_match(".gitignore"));
        assert!(!gs.is_match("file.txt"));
    }

    #[test]
    fn test_build_glob_set_nested_pattern() {
        let patterns = vec!["lib/tmp/**".to_string()];
        let gs = build_glob_set(&patterns).unwrap().unwrap();
        assert!(gs.is_match("lib/tmp/foo.txt"));
        assert!(gs.is_match("lib/tmp/sub/bar.txt"));
        assert!(!gs.is_match("lib/foo.txt"));
    }

    #[test]
    fn test_build_file_entry_preserves_explicit_format() {
        let temp = TempDir::new().unwrap();
        let entry = FileEntry {
            name: ".npmrc".to_string(),
            from_file: None,
            from_config: Some(serde_json::json!({
                "prefix": "~/.local/share/npm",
                "save-exact": "true",
            })),
            prefix: vec![],
            format: Some(Format::EqualsRecordLines),
            ignore: vec![],
            when: None,
            symlink: false,
            exclude: vec![],
            local: None,
            immutable: false,
        };

        let result =
            build_file_entry(temp.path(), &entry, temp.path().join(".npmrc"), vec![]).unwrap();

        assert_eq!(result.format, Format::EqualsRecordLines);
        assert_eq!(result.content, "prefix=~/.local/share/npm\nsave-exact=true");
    }

    #[test]
    fn test_collect_merged_files_no_overlay() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        std::fs::create_dir_all(source.join("sub")).unwrap();
        std::fs::write(source.join("a.txt"), "a").unwrap();
        std::fs::write(source.join("sub/b.txt"), "b").unwrap();

        let merged = collect_merged_files(&source, None, None).unwrap();
        assert_eq!(merged.len(), 2);
        assert!(merged.iter().all(|m| !m.is_local));
    }

    #[test]
    fn test_collect_merged_files_with_exclude() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        std::fs::create_dir_all(&source).unwrap();
        std::fs::write(source.join("keep.txt"), "keep").unwrap();
        std::fs::write(source.join("skip.bak"), "skip").unwrap();
        std::fs::write(source.join(".gitignore"), "ignore").unwrap();

        let patterns = vec!["*.bak".to_string(), ".gitignore".to_string()];
        let gs = build_glob_set(&patterns).unwrap();
        let merged = collect_merged_files(&source, None, gs.as_ref()).unwrap();

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].rel_path, std::path::PathBuf::from("keep.txt"));
    }

    #[test]
    fn test_collect_merged_files_with_local_overlay() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        let local = temp.path().join("local");

        // Source files
        std::fs::create_dir_all(source.join("sub")).unwrap();
        std::fs::write(source.join("tracked.txt"), "tracked").unwrap();
        std::fs::write(source.join("sub/shared.txt"), "from-source").unwrap();

        // Local overlay: overrides sub/shared.txt and adds new.txt
        std::fs::create_dir_all(local.join("sub")).unwrap();
        std::fs::write(local.join("sub/shared.txt"), "from-local").unwrap();
        std::fs::write(local.join("new.txt"), "new-local").unwrap();

        let merged = collect_merged_files(&source, Some(&local), None).unwrap();

        assert_eq!(merged.len(), 3);

        // new.txt (local)
        let new = merged
            .iter()
            .find(|m| m.rel_path == PathBuf::from("new.txt"))
            .unwrap();
        assert!(new.is_local);

        // sub/shared.txt (local override)
        let shared = merged
            .iter()
            .find(|m| m.rel_path == PathBuf::from("sub/shared.txt"))
            .unwrap();
        assert!(shared.is_local);
        assert_eq!(
            std::fs::read_to_string(&shared.source).unwrap(),
            "from-local"
        );

        // tracked.txt (from source)
        let tracked = merged
            .iter()
            .find(|m| m.rel_path == PathBuf::from("tracked.txt"))
            .unwrap();
        assert!(!tracked.is_local);
    }

    #[test]
    fn test_collect_merged_files_exclude_applies_to_local() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        let local = temp.path().join("local");

        std::fs::create_dir_all(&source).unwrap();
        std::fs::write(source.join("a.txt"), "a").unwrap();

        std::fs::create_dir_all(&local).unwrap();
        std::fs::write(local.join("skip.bak"), "skip").unwrap();
        std::fs::write(local.join("keep.txt"), "keep").unwrap();

        let patterns = vec!["*.bak".to_string()];
        let gs = build_glob_set(&patterns).unwrap();
        let merged = collect_merged_files(&source, Some(&local), gs.as_ref()).unwrap();

        // Should have a.txt and keep.txt, but not skip.bak
        assert_eq!(merged.len(), 2);
        assert!(
            !merged
                .iter()
                .any(|m| m.rel_path == PathBuf::from("skip.bak"))
        );
    }

    #[test]
    fn test_copy_directory_with_exclude() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        let target = temp.path().join("target");

        std::fs::create_dir_all(&source).unwrap();
        std::fs::write(source.join("keep.txt"), "keep").unwrap();
        std::fs::write(source.join("skip.bak"), "skip").unwrap();

        let patterns = vec!["*.bak".to_string()];
        let gs = build_glob_set(&patterns).unwrap();

        copy_directory(&source, &target, None, gs.as_ref(), false).unwrap();

        assert!(target.join("keep.txt").exists());
        assert!(!target.join("skip.bak").exists());
    }

    #[test]
    fn test_copy_directory_with_local_overlay() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        let local = temp.path().join("local");
        let target = temp.path().join("target");

        std::fs::create_dir_all(&source).unwrap();
        std::fs::write(source.join("tracked.txt"), "from-source").unwrap();
        std::fs::write(source.join("shared.txt"), "source-version").unwrap();

        std::fs::create_dir_all(&local).unwrap();
        std::fs::write(local.join("shared.txt"), "local-version").unwrap();
        std::fs::write(local.join("extra.txt"), "local-only").unwrap();

        copy_directory(&source, &target, Some(&local), None, false).unwrap();

        assert_eq!(
            std::fs::read_to_string(target.join("tracked.txt")).unwrap(),
            "from-source"
        );
        assert_eq!(
            std::fs::read_to_string(target.join("shared.txt")).unwrap(),
            "local-version"
        );
        assert_eq!(
            std::fs::read_to_string(target.join("extra.txt")).unwrap(),
            "local-only"
        );
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    #[test]
    fn test_write_result_clears_existing_immutable_even_when_desired_false() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_lock().lock().unwrap();
        let temp = TempDir::new().unwrap();
        let bin = temp.path().join("bin");
        let calls = temp.path().join("immutable-calls.log");
        std::fs::create_dir(&bin).unwrap();

        #[cfg(target_os = "macos")]
        let (tool, clear_arg, set_arg) = ("chflags", "nouchg", "uchg");
        #[cfg(target_os = "linux")]
        let (tool, clear_arg, set_arg) = ("chattr", "-i", "+i");

        let tool_path = bin.join(tool);
        std::fs::write(
            &tool_path,
            format!("#!/bin/sh\nprintf '%s\\n' \"$*\" >> {}\n", calls.display()),
        )
        .unwrap();
        let mut perms = std::fs::metadata(&tool_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&tool_path, perms).unwrap();

        let prev_path = std::env::var_os("PATH");
        let new_path = match &prev_path {
            Some(path) => format!("{}:{}", bin.display(), path.to_string_lossy()),
            None => bin.display().to_string(),
        };
        unsafe { std::env::set_var("PATH", new_path) };

        let target = temp.path().join("target.txt");
        std::fs::write(&target, "old").unwrap();

        let result = BuildResult {
            target: target.clone(),
            content: "new".to_string(),
            is_plaintext: false,
            source_path: None,
            name: "target.txt".to_string(),
            format: Format::Plaintext,
            ignore_keys: vec![],
            is_symlink: false,
            canonical_source: None,
            exclude_patterns: vec![],
            local_dir: None,
            immutable: false,
        };

        let write = write_result(&result, false);

        unsafe {
            match prev_path {
                Some(path) => std::env::set_var("PATH", path),
                None => std::env::remove_var("PATH"),
            }
        }

        write.unwrap();
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "new");

        let calls = std::fs::read_to_string(calls).unwrap();
        assert!(
            calls
                .lines()
                .any(|line| line == clear_arg || line.starts_with(&format!("{clear_arg} "))),
            "expected clear immutable call `{clear_arg}`, got: {calls}"
        );
        assert!(
            !calls
                .lines()
                .any(|line| line == set_arg || line.starts_with(&format!("{set_arg} "))),
            "desired immutable=false should not set immutable, got: {calls}"
        );
    }
}
