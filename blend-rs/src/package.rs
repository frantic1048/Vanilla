use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

const EXCLUDED_DIRS: &[&str] = &["darwin-system", "root", "screenshots"];

pub fn discover_packages(packages_dir: &Path) -> HashSet<String> {
    let mut packages = HashSet::new();

    let Ok(entries) = std::fs::read_dir(packages_dir) else {
        return packages;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        // Skip excluded directories
        if EXCLUDED_DIRS.contains(&name) {
            continue;
        }

        // Skip hidden directories
        if name.starts_with('.') {
            continue;
        }

        // Skip git-ignored directories
        if is_git_ignored(&path) {
            continue;
        }

        packages.insert(name.to_string());
    }

    packages
}

fn is_git_ignored(path: &Path) -> bool {
    Command::new("git")
        .args(["check-ignore", "-q"])
        .arg(path)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
