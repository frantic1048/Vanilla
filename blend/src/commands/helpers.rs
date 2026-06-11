use console::style;

use crate::compose::{BuildResult, discover_orders};
use crate::context::Context;
use crate::diff::{DiffResult, FileDiffResult, diff_configs, diff_directory};
use crate::nickel;

pub fn select_orders(ctx: &Context, orders: &[String]) -> Vec<String> {
    let mut selected: Vec<String> = if orders.is_empty() {
        discover_orders(&ctx.orders_dir).into_iter().collect()
    } else {
        orders.to_vec()
    };
    selected.sort();
    selected
}

/// For directory-style entries: walk the source dir and check whether any
/// per-file target is itself a symlink. Returns true even when resolved
/// content matches, so callers can flag legacy stow leftovers within an
/// otherwise in-sync directory.
pub fn dir_has_inner_symlinks(source_dir: &std::path::Path, target_dir: &std::path::Path) -> bool {
    diff_directory(source_dir, target_dir, &[])
        .iter()
        .any(|f| f.target_is_symlink)
}

/// True when the only differences in a directory entry are inner-file
/// symlinks with matching resolved content. Push under this condition
/// rewrites file types only — no content changes — so sync should
/// auto-redeploy rather than prompt the user.
pub fn only_structural_symlink_changes(result: &BuildResult) -> bool {
    let Some(src) = result.source_path.as_ref() else {
        return false;
    };
    if !src.is_dir() {
        return false;
    }
    let file_diffs = compute_dir_file_diffs(result);
    if file_diffs.is_empty() {
        return false;
    }
    let mut saw_symlink = false;
    for f in &file_diffs {
        if !f.has_changes {
            continue;
        }
        if f.source_only || !f.target_is_symlink || !f.diff_output.is_empty() {
            return false;
        }
        saw_symlink = true;
    }
    saw_symlink
}

/// Check if a target path or any of its parent components is a symlink
/// when the order entry does NOT want a symlink.
pub fn target_is_unexpected_symlink(target: &std::path::Path, is_symlink_entry: bool) -> bool {
    if is_symlink_entry {
        return false;
    }
    // Check the target itself
    if let Ok(meta) = std::fs::symlink_metadata(target)
        && meta.file_type().is_symlink()
    {
        return true;
    }
    // Check parent path components (e.g., ~/.config/skhd is a symlink,
    // target is ~/.config/skhd/skhdrc which resolves through it)
    for ancestor in target.ancestors().skip(1) {
        if ancestor == std::path::Path::new("/") || ancestor == std::path::Path::new("") {
            break;
        }
        if let Ok(meta) = std::fs::symlink_metadata(ancestor) {
            if meta.file_type().is_symlink() {
                return true;
            }
            // Stop at first real existing ancestor
            break;
        }
    }
    false
}

/// Compute the diff between a build result and the deployed file
pub fn compute_diff_for_result(result: &BuildResult) -> DiffResult {
    if result.is_symlink {
        return DiffResult::no_changes();
    }

    if !result.target.exists() {
        return DiffResult::no_changes();
    }

    if result.is_plaintext {
        if let Some(source_path) = &result.source_path {
            if source_path.is_dir() {
                // For directories, compute an aggregate diff from per-file results.
                // diff_directory only reports files present in the source.
                let file_diffs = diff_directory(source_path, &result.target, &result.ignore_keys);
                return aggregate_dir_diff(&file_diffs);
            }
            if let (Ok(source_content), Ok(deployed)) = (
                std::fs::read_to_string(source_path),
                std::fs::read_to_string(&result.target),
            ) {
                return diff_configs(
                    nickel::Format::Plaintext,
                    &source_content,
                    &deployed,
                    &result.ignore_keys,
                );
            }
        }
        DiffResult::no_changes()
    } else if let Ok(deployed) = std::fs::read_to_string(&result.target) {
        diff_configs(
            result.format,
            &result.content,
            &deployed,
            &result.ignore_keys,
        )
    } else {
        DiffResult::no_changes()
    }
}

/// Compute per-file diffs for a directory build result, filtering out
/// "target only" files that are managed by other entries in the same order.
pub fn compute_dir_file_diffs(result: &BuildResult) -> Vec<FileDiffResult> {
    if let Some(source_path) = &result.source_path
        && source_path.is_dir()
    {
        return diff_directory(source_path, &result.target, &result.ignore_keys);
    }
    Vec::new()
}

/// Aggregate per-file diffs into a single DiffResult (for sync compatibility)
pub fn aggregate_dir_diff(file_diffs: &[FileDiffResult]) -> DiffResult {
    let any_changes = file_diffs.iter().any(|f| f.has_changes);
    if !any_changes {
        return DiffResult::no_changes();
    }

    let mut output_lines = Vec::new();
    for f in file_diffs {
        let path_str = f.rel_path.display();
        if f.source_only {
            output_lines.push(format!(
                "{} {}",
                style("<< Source").blue(),
                style(format!("{} (missing from Target)", path_str)).blue()
            ));
        } else if f.has_changes {
            let annotation = match (f.target_is_symlink, !f.diff_output.is_empty()) {
                (true, true) => "unexpected symlink, modified",
                (true, false) => "unexpected symlink",
                (false, _) => "modified",
            };
            output_lines.push(format!(
                "{} {}",
                style("\u{2260}").yellow(),
                style(format!("{} ({})", path_str, annotation)).yellow()
            ));
            if !f.diff_output.is_empty() {
                for line in f.diff_output.lines() {
                    output_lines.push(format!("  {}", line));
                }
            }
        }
    }

    DiffResult::with_changes(output_lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    #[test]
    fn test_aggregate_dir_diff_all_in_sync() {
        let diffs = vec![
            FileDiffResult {
                rel_path: PathBuf::from("a.txt"),
                has_changes: false,
                source_only: false,
                diff_output: String::new(),
                target_is_symlink: false,
            },
            FileDiffResult {
                rel_path: PathBuf::from("b.txt"),
                has_changes: false,
                source_only: false,
                diff_output: String::new(),
                target_is_symlink: false,
            },
        ];
        let result = aggregate_dir_diff(&diffs);
        assert!(!result.has_changes);
    }

    #[test]
    fn test_aggregate_dir_diff_some_changed() {
        let diffs = vec![FileDiffResult {
            rel_path: PathBuf::from("modified.txt"),
            has_changes: true,
            source_only: false,
            diff_output: "diff".to_string(),
            target_is_symlink: false,
        }];
        let result = aggregate_dir_diff(&diffs);
        assert!(result.has_changes);
        assert!(result.output.contains("modified.txt"));
    }

    #[test]
    fn test_aggregate_dir_diff_source_only_shows_source_marker() {
        let diffs = vec![FileDiffResult {
            rel_path: PathBuf::from("new_file.txt"),
            has_changes: true,
            source_only: true,
            diff_output: String::new(),
            target_is_symlink: false,
        }];
        let result = aggregate_dir_diff(&diffs);
        let plain = console::strip_ansi_codes(&result.output);
        assert!(result.has_changes);
        assert!(plain.contains("<< Source new_file.txt"));
    }

    #[test]
    fn test_aggregate_dir_diff_modified_shows_neq() {
        let diffs = vec![FileDiffResult {
            rel_path: PathBuf::from("changed.conf"),
            has_changes: true,
            source_only: false,
            diff_output: "line diff".to_string(),
            target_is_symlink: false,
        }];
        let result = aggregate_dir_diff(&diffs);
        let plain = console::strip_ansi_codes(&result.output);
        assert!(result.has_changes);
        assert!(plain.contains("\u{2260} changed.conf"));
    }

    #[test]
    fn test_aggregate_dir_diff_empty_input() {
        let result = aggregate_dir_diff(&[]);
        assert!(!result.has_changes);
    }

    #[test]
    fn test_aggregate_dir_diff_unexpected_symlink_matching_content() {
        // Inner-file symlink with matching resolved content: surface
        // the type mismatch even though there is no textual diff.
        let diffs = vec![FileDiffResult {
            rel_path: PathBuf::from(".prototools"),
            has_changes: true,
            source_only: false,
            diff_output: String::new(),
            target_is_symlink: true,
        }];
        let result = aggregate_dir_diff(&diffs);
        let plain = console::strip_ansi_codes(&result.output);
        assert!(result.has_changes);
        assert!(
            plain.contains(".prototools") && plain.contains("unexpected symlink"),
            "expected unexpected-symlink annotation, got:\n{plain}"
        );
    }

    #[test]
    fn test_aggregate_dir_diff_unexpected_symlink_with_content_diff() {
        let diffs = vec![FileDiffResult {
            rel_path: PathBuf::from(".prototools"),
            has_changes: true,
            source_only: false,
            diff_output: ">> Target: old\n<< Source: new".to_string(),
            target_is_symlink: true,
        }];
        let result = aggregate_dir_diff(&diffs);
        let plain = console::strip_ansi_codes(&result.output);
        assert!(plain.contains("unexpected symlink"));
        assert!(plain.contains(">> Target: old"));
    }

    #[cfg(unix)]
    #[test]
    fn test_dir_has_inner_symlinks_detects_symlink_target() {
        // Per-file detection: for directory entries we must walk into the
        // target dir, not just check the dir's own type.
        let source = TempDir::new().unwrap();
        let target = TempDir::new().unwrap();
        let backing = TempDir::new().unwrap();
        std::fs::write(source.path().join("file"), "x").unwrap();
        std::fs::write(backing.path().join("file"), "x").unwrap();
        std::os::unix::fs::symlink(backing.path().join("file"), target.path().join("file"))
            .unwrap();
        assert!(dir_has_inner_symlinks(source.path(), target.path()));
    }

    #[cfg(unix)]
    #[test]
    fn test_dir_has_inner_symlinks_false_when_all_regular() {
        let source = TempDir::new().unwrap();
        let target = TempDir::new().unwrap();
        std::fs::write(source.path().join("file"), "x").unwrap();
        std::fs::write(target.path().join("file"), "x").unwrap();
        assert!(!dir_has_inner_symlinks(source.path(), target.path()));
    }

    #[cfg(unix)]
    #[test]
    fn test_dir_has_inner_symlinks_only_inspects_files_present_in_source() {
        // A symlink that exists only in target (no source counterpart)
        // is unmanaged by blend and must NOT trigger the warning, so the
        // detection stays scoped to files originating from the order.
        let source = TempDir::new().unwrap();
        let target = TempDir::new().unwrap();
        let backing = TempDir::new().unwrap();
        std::fs::write(source.path().join("managed"), "x").unwrap();
        std::fs::write(target.path().join("managed"), "x").unwrap();
        std::fs::write(backing.path().join("stray"), "y").unwrap();
        std::os::unix::fs::symlink(backing.path().join("stray"), target.path().join("stray"))
            .unwrap();
        assert!(!dir_has_inner_symlinks(source.path(), target.path()));
    }

    #[test]
    fn test_aggregate_dir_diff_mixed_indicators() {
        let diffs = vec![
            FileDiffResult {
                rel_path: PathBuf::from("added.txt"),
                has_changes: true,
                source_only: true,
                diff_output: String::new(),
                target_is_symlink: false,
            },
            FileDiffResult {
                rel_path: PathBuf::from("modified.txt"),
                has_changes: true,
                source_only: false,
                diff_output: "diff".to_string(),
                target_is_symlink: false,
            },
            FileDiffResult {
                rel_path: PathBuf::from("stable.txt"),
                has_changes: false,
                source_only: false,
                diff_output: String::new(),
                target_is_symlink: false,
            },
        ];
        let result = aggregate_dir_diff(&diffs);
        let plain = console::strip_ansi_codes(&result.output);
        assert!(result.has_changes);
        assert!(plain.contains("<< Source added.txt"));
        assert!(plain.contains("\u{2260} modified.txt"));
        assert!(!plain.contains("stable.txt"));
    }

    #[test]
    fn test_compute_dir_file_diffs_non_directory_source() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("plain.txt");
        std::fs::write(&file_path, "content").unwrap();
        let result = BuildResult {
            target: temp.path().join("target"),
            content: String::new(),
            is_plaintext: true,
            source_path: Some(file_path),
            name: "plain.txt".to_string(),
            format: nickel::Format::Plaintext,
            ignore_keys: vec![],
            is_symlink: false,
            canonical_source: None,
            exclude_patterns: vec![],
            local_dir: None,
            immutable: false,
        };
        assert!(compute_dir_file_diffs(&result).is_empty());
    }

    #[test]
    fn test_compute_dir_file_diffs_no_source_path() {
        let temp = TempDir::new().unwrap();
        let result = BuildResult {
            target: temp.path().to_path_buf(),
            content: "rendered".to_string(),
            is_plaintext: false,
            source_path: None,
            name: "config.toml".to_string(),
            format: nickel::Format::Toml,
            ignore_keys: vec![],
            is_symlink: false,
            canonical_source: None,
            exclude_patterns: vec![],
            local_dir: None,
            immutable: false,
        };
        assert!(compute_dir_file_diffs(&result).is_empty());
    }

    #[test]
    fn test_compute_diff_for_result_uses_stored_format() {
        let temp = TempDir::new().unwrap();
        let target = temp.path().join(".npmrc");
        std::fs::write(&target, "prefix=/target\nprefix-extra=target\n").unwrap();

        let result = BuildResult {
            target,
            content: "prefix=/source\nprefix-extra=source\n".to_string(),
            is_plaintext: false,
            source_path: None,
            name: ".npmrc".to_string(),
            format: nickel::Format::EqualsRecordLines,
            ignore_keys: vec!["prefix".to_string()],
            is_symlink: false,
            canonical_source: None,
            exclude_patterns: vec![],
            local_dir: None,
            immutable: false,
        };

        let diff = compute_diff_for_result(&result);
        let plain = console::strip_ansi_codes(&diff.output);
        assert!(diff.has_changes);
        assert!(plain.contains("prefix-extra"));
        assert!(!plain.contains("prefix:"));
    }

    #[test]
    fn test_compute_dir_file_diffs_directory_source() {
        let source = TempDir::new().unwrap();
        let target = TempDir::new().unwrap();
        let src_dir = source.path().join("conf_dir");
        let tgt_dir = target.path().join("deployed_dir");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::create_dir_all(&tgt_dir).unwrap();
        std::fs::write(src_dir.join("a.conf"), "key=1\n").unwrap();
        std::fs::write(tgt_dir.join("a.conf"), "key=1\n").unwrap();
        std::fs::write(src_dir.join("b.conf"), "new\n").unwrap();
        let result = BuildResult {
            target: tgt_dir,
            content: String::new(),
            is_plaintext: true,
            source_path: Some(src_dir),
            name: "conf_dir".to_string(),
            format: nickel::Format::Plaintext,
            ignore_keys: vec![],
            is_symlink: false,
            canonical_source: None,
            exclude_patterns: vec![],
            local_dir: None,
            immutable: false,
        };
        let diffs = compute_dir_file_diffs(&result);
        assert_eq!(diffs.len(), 2);
        let a = diffs
            .iter()
            .find(|d| d.rel_path.as_path() == Path::new("a.conf"))
            .unwrap();
        assert!(!a.has_changes);
        let b = diffs
            .iter()
            .find(|d| d.rel_path.as_path() == Path::new("b.conf"))
            .unwrap();
        assert!(b.has_changes);
        assert!(b.source_only);
    }

    #[test]
    fn test_compute_dir_file_diffs_respects_ignore_keys() {
        let source = TempDir::new().unwrap();
        let target = TempDir::new().unwrap();
        let src_dir = source.path().join("dir");
        let tgt_dir = target.path().join("dir");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::create_dir_all(&tgt_dir).unwrap();
        std::fs::write(src_dir.join("settings"), "fixed=1\nrandom=abc\n").unwrap();
        std::fs::write(tgt_dir.join("settings"), "fixed=1\nrandom=xyz\n").unwrap();
        let result = BuildResult {
            target: tgt_dir,
            content: String::new(),
            is_plaintext: true,
            source_path: Some(src_dir),
            name: "dir".to_string(),
            format: nickel::Format::Plaintext,
            ignore_keys: vec!["^random".to_string()],
            is_symlink: false,
            canonical_source: None,
            exclude_patterns: vec![],
            local_dir: None,
            immutable: false,
        };
        let diffs = compute_dir_file_diffs(&result);
        assert_eq!(diffs.len(), 1);
        assert!(!diffs[0].has_changes);
    }
}
