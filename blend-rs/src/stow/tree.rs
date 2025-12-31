use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::ignore::IgnorePatterns;
use super::StowAction;

/// Check if a path is safely contained within a base directory.
/// Returns the canonicalized path if safe, None if path escapes base.
fn safe_path_within(base: &Path, relative: &Path) -> Option<PathBuf> {
    // Normalize the relative path by removing any .. components
    let mut result = base.to_path_buf();
    for component in relative.components() {
        match component {
            std::path::Component::Normal(c) => result.push(c),
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                // Trying to escape - check if we're still within base
                if !result.pop() || !result.starts_with(base) {
                    return None;
                }
            }
            std::path::Component::Prefix(_) | std::path::Component::RootDir => {
                // Absolute path components in relative path - reject
                return None;
            }
        }
    }
    // Final check: ensure we haven't escaped
    if result.starts_with(base) {
        Some(result)
    } else {
        None
    }
}

/// Stow a package directory to a target directory.
/// Returns a list of actions to perform.
pub fn stow_package(pkg_dir: &Path, target_dir: &Path, ignore: &IgnorePatterns) -> Vec<StowAction> {
    use std::collections::HashSet;

    let mut actions = Vec::new();
    // Track directories we've decided to symlink entirely (folded)
    // so we can skip their children
    let mut folded_dirs: HashSet<PathBuf> = HashSet::new();

    // Walk the package directory
    for entry in WalkDir::new(pkg_dir)
        .min_depth(1)
        .into_iter()
        .filter_entry(|e| !ignore.should_ignore(e.path()))
        .filter_map(|e| e.ok())
    {
        let source_path = entry.path();

        // Skip if this path is under a directory we've already folded
        if folded_dirs.iter().any(|d| source_path.starts_with(d)) {
            continue;
        }

        let relative = source_path
            .strip_prefix(pkg_dir)
            .expect("WalkDir entry should have pkg_dir prefix");

        // Validate target path doesn't escape target directory (path traversal protection)
        let target_path = match safe_path_within(target_dir, relative) {
            Some(p) => p,
            None => {
                actions.push(StowAction::Conflict {
                    target: target_dir.join(relative),
                    reason: "path traversal attempt detected".into(),
                });
                continue;
            }
        };

        if entry.file_type().is_dir() {
            // For directories, check if we can fold (symlink entire dir)
            // or need to recurse
            if !target_path.exists() {
                // Target doesn't exist - we can fold (create symlink to entire dir)
                actions.push(StowAction::CreateSymlink {
                    source: source_path.to_path_buf(),
                    target: target_path,
                });
                // Mark this directory as folded so we skip its children
                folded_dirs.insert(source_path.to_path_buf());
                continue;
            } else if target_path.is_symlink() {
                // Check if it's our symlink
                if is_our_symlink(&target_path, source_path) {
                    // Already linked, mark as folded and skip children
                    folded_dirs.insert(source_path.to_path_buf());
                    continue;
                } else {
                    // Conflict: symlink exists pointing elsewhere
                    actions.push(StowAction::Conflict {
                        target: target_path,
                        reason: "symlink exists pointing to different location".into(),
                    });
                    folded_dirs.insert(source_path.to_path_buf());
                    continue;
                }
            } else if target_path.is_dir() {
                // Target is a real directory, we need to unfold and recurse
                // WalkDir will handle recursion automatically
                continue;
            } else {
                // Target exists as a file - conflict
                actions.push(StowAction::Conflict {
                    target: target_path,
                    reason: "file exists where directory expected".into(),
                });
                continue;
            }
        } else if entry.file_type().is_file() || entry.file_type().is_symlink() {
            // For files, create symlink
            if !target_path.exists() {
                // Ensure parent directory exists
                if let Some(parent) = target_path.parent() {
                    if !parent.exists() {
                        actions.push(StowAction::CreateDirectory(parent.to_path_buf()));
                    }
                }
                actions.push(StowAction::CreateSymlink {
                    source: source_path.to_path_buf(),
                    target: target_path,
                });
            } else if target_path.is_symlink() {
                if is_our_symlink(&target_path, source_path) {
                    // Already linked correctly
                    continue;
                } else {
                    actions.push(StowAction::Conflict {
                        target: target_path,
                        reason: "symlink exists pointing to different location".into(),
                    });
                }
            } else {
                // Regular file exists - conflict
                actions.push(StowAction::Conflict {
                    target: target_path,
                    reason: "file already exists".into(),
                });
            }
        }
    }

    // Deduplicate and optimize actions
    optimize_actions(actions)
}

/// Unstow a package - remove symlinks pointing to the package
pub fn unstow_package(pkg_dir: &Path, target_dir: &Path) -> Vec<StowAction> {
    let mut actions = Vec::new();

    for entry in WalkDir::new(pkg_dir)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let source_path = entry.path();
        let relative = source_path
            .strip_prefix(pkg_dir)
            .expect("WalkDir entry should have pkg_dir prefix");

        // Use safe path calculation for consistency (skip invalid paths)
        let Some(target_path) = safe_path_within(target_dir, relative) else {
            continue;
        };

        if target_path.is_symlink() && is_our_symlink(&target_path, source_path) {
            actions.push(StowAction::RemoveSymlink(target_path));
        }
    }

    // TODO: Add cleanup of empty directories
    actions
}

/// Check if target symlink points to source
fn is_our_symlink(target: &Path, source: &Path) -> bool {
    if !target.is_symlink() {
        return false;
    }

    let Ok(link_dest) = std::fs::read_link(target) else {
        return false;
    };

    // Resolve relative symlink
    let resolved = if link_dest.is_absolute() {
        link_dest
    } else {
        target
            .parent()
            .map(|p| p.join(&link_dest))
            .unwrap_or(link_dest)
    };

    // Compare canonical paths
    let Ok(resolved_canonical) = resolved.canonicalize() else {
        return false;
    };
    let Ok(source_canonical) = source.canonicalize() else {
        return false;
    };

    resolved_canonical == source_canonical
}

/// Optimize actions by removing redundant ones
fn optimize_actions(actions: Vec<StowAction>) -> Vec<StowAction> {
    use std::collections::HashSet;
    use std::path::PathBuf;

    // First pass: collect all symlink targets (these represent "folded" directories)
    let symlinked_dirs: HashSet<PathBuf> = actions
        .iter()
        .filter_map(|action| {
            if let StowAction::CreateSymlink { target, source } = action {
                if source.is_dir() {
                    return Some(target.clone());
                }
            }
            None
        })
        .collect();

    let mut seen_dirs: HashSet<PathBuf> = HashSet::new();
    let mut result = Vec::new();

    for action in actions {
        match &action {
            StowAction::CreateDirectory(path) => {
                // Skip if this directory is under a symlinked directory
                let under_symlink = symlinked_dirs.iter().any(|s| path.starts_with(s));
                if under_symlink {
                    continue;
                }
                if !seen_dirs.contains(path) {
                    seen_dirs.insert(path.clone());
                    result.push(action);
                }
            }
            StowAction::CreateSymlink { target, .. } => {
                // Skip if we're already creating a symlink for a parent directory
                let under_symlink = symlinked_dirs.iter().any(|s| target.starts_with(s) && target != s);
                if !under_symlink {
                    result.push(action);
                }
            }
            _ => result.push(action),
        }
    }

    result
}
