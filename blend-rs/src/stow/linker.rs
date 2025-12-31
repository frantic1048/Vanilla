use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Component, Path, PathBuf};

use super::StowAction;

/// Execute a list of stow actions
pub fn execute_actions(actions: Vec<StowAction>) {
    for action in actions {
        if let Err(e) = execute_action(&action) {
            eprintln!("Error executing {:?}: {}", action, e);
        }
    }
}

fn execute_action(action: &StowAction) -> std::io::Result<()> {
    match action {
        StowAction::CreateSymlink { source, target } => {
            create_symlink(source, target)?;
        }
        StowAction::RemoveSymlink(path) => {
            fs::remove_file(path)?;
        }
        StowAction::CreateDirectory(path) => {
            fs::create_dir_all(path)?;
        }
        StowAction::Conflict { .. } => {
            // Conflicts are reported but not executed
        }
    }
    Ok(())
}

fn create_symlink(source: &Path, target: &Path) -> std::io::Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = target.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    // Only remove symlink if it's broken or already points to our source
    if target.is_symlink() {
        let should_remove = match std::fs::read_link(target) {
            Ok(link_dest) => {
                // Resolve relative symlink
                let resolved = if link_dest.is_absolute() {
                    link_dest
                } else {
                    target
                        .parent()
                        .map(|p| p.join(&link_dest))
                        .unwrap_or(link_dest)
                };
                // Remove if broken (target doesn't exist) or points to our source
                !resolved.exists() || resolved.canonicalize().ok() == source.canonicalize().ok()
            }
            Err(_) => false, // Can't read link, don't remove
        };
        if should_remove {
            fs::remove_file(target)?;
        } else {
            // Symlink exists pointing elsewhere - this is a conflict
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "symlink exists pointing to different location",
            ));
        }
    }

    // Create relative symlink if possible
    let link_path = make_relative_path(source, target);
    symlink(&link_path, target)
}

/// Make a relative path from target to source
fn make_relative_path(source: &Path, target: &Path) -> PathBuf {
    // Get the directory containing the target
    let target_dir = target.parent().unwrap_or(Path::new(""));

    // Try to make source relative to target_dir
    if let (Ok(source_abs), Ok(target_dir_abs)) =
        (source.canonicalize(), target_dir.canonicalize())
    {
        if let Some(relative) = diff_paths(&source_abs, &target_dir_abs) {
            return relative;
        }
    }

    // Fallback to absolute path
    source.to_path_buf()
}

/// Compute relative path from `base` to `path`
/// Returns None if paths are on different prefixes (e.g., different drives on Windows)
fn diff_paths(path: &Path, base: &Path) -> Option<PathBuf> {
    let path_components: Vec<_> = path.components().collect();
    let base_components: Vec<_> = base.components().collect();

    // Find common prefix length
    let common_len = path_components
        .iter()
        .zip(base_components.iter())
        .take_while(|(a, b)| a == b)
        .count();

    // If no common prefix at all, can't compute relative path
    if common_len == 0 {
        // Check if both start with root
        match (path_components.first(), base_components.first()) {
            (Some(Component::RootDir), Some(Component::RootDir)) => {}
            _ => return None,
        }
    }

    let mut result = PathBuf::new();

    // Add ".." for each remaining component in base
    for _ in common_len..base_components.len() {
        result.push("..");
    }

    // Add remaining components from path
    for component in &path_components[common_len..] {
        result.push(component);
    }

    Some(result)
}
