use anyhow::{Context as _, Result};
use console::style;

use crate::compose::{BuildResult, build_glob_set, collect_merged_files, discover_orders};
use crate::context::Context;
use crate::diff::{DiffResult, FileDiffResult, diff_configs, diff_managed_files};
use crate::fs_node::{NodeKind, node_kind};
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

pub fn expected_node_kind(result: &BuildResult) -> NodeKind {
    if result.is_symlink {
        NodeKind::Symlink
    } else if result
        .source_path
        .as_ref()
        .is_some_and(|source_path| source_path.is_dir())
    {
        NodeKind::Directory
    } else {
        NodeKind::File
    }
}

pub fn target_type_mismatch(result: &BuildResult) -> Result<Option<(NodeKind, NodeKind)>> {
    let expected = expected_node_kind(result);
    let Some(actual) = node_kind(&result.target)
        .with_context(|| format!("could not inspect target {}", result.target.display()))?
    else {
        return Ok(None);
    };
    Ok((expected != actual).then_some((expected, actual)))
}

pub fn result_has_type_mismatch(result: &BuildResult) -> Result<bool> {
    if target_type_mismatch(result)?.is_some() {
        return Ok(true);
    }
    Ok(compute_dir_file_diffs(result)?
        .iter()
        .any(FileDiffResult::has_type_mismatch))
}

pub fn compute_managed_dir_diffs(
    source_dir: &std::path::Path,
    target_dir: &std::path::Path,
    local_dir: Option<&std::path::Path>,
    exclude_patterns: &[String],
    ignore_keys: &[String],
) -> Result<Vec<FileDiffResult>> {
    let exclude = build_glob_set(exclude_patterns)?;
    let merged = collect_merged_files(source_dir, local_dir, exclude.as_ref())?;
    let managed_files: Vec<_> = merged
        .into_iter()
        .map(|file| (file.source, file.rel_path))
        .collect();
    Ok(diff_managed_files(&managed_files, target_dir, ignore_keys)?)
}

/// Compute the diff between a build result and the deployed file
pub fn compute_diff_for_result(result: &BuildResult) -> Result<DiffResult> {
    let target_kind = node_kind(&result.target)
        .with_context(|| format!("could not inspect target {}", result.target.display()))?;
    if target_kind.is_none() {
        return Ok(DiffResult::no_changes());
    }

    if let Some((expected, actual)) = target_type_mismatch(result)? {
        return Ok(DiffResult::with_changes(format!(
            "target type mismatch: expected {expected}, found {actual}"
        )));
    }

    if result.is_symlink {
        let expected = result
            .canonical_source
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "<unknown>".to_string());
        return Ok(match std::fs::read_link(&result.target) {
            Ok(actual) if result.canonical_source.as_ref() == Some(&actual) => {
                DiffResult::no_changes()
            }
            Ok(actual) => DiffResult::with_changes(format!(
                "symlink target mismatch: expected {expected}, found {}",
                actual.display()
            )),
            Err(error) => {
                DiffResult::with_changes(format!("could not read target symlink: {error}"))
            }
        });
    }

    if result.is_plaintext {
        if let Some(source_path) = &result.source_path {
            if source_path.is_dir() {
                let file_diffs = compute_dir_file_diffs(result)?;
                return Ok(aggregate_dir_diff(&file_diffs));
            }
            if let (Ok(source_content), Ok(deployed)) = (
                std::fs::read_to_string(source_path),
                std::fs::read_to_string(&result.target),
            ) {
                return Ok(diff_configs(
                    nickel::Format::Plaintext,
                    &source_content,
                    &deployed,
                    &result.ignore_keys,
                ));
            }
        }
        Ok(DiffResult::no_changes())
    } else if let Ok(deployed) = std::fs::read_to_string(&result.target) {
        Ok(diff_configs(
            result.format,
            &result.content,
            &deployed,
            &result.ignore_keys,
        ))
    } else {
        Ok(DiffResult::no_changes())
    }
}

/// Compute per-file diffs for a directory build result, filtering out
/// "target only" files that are managed by other entries in the same order.
pub fn compute_dir_file_diffs(result: &BuildResult) -> Result<Vec<FileDiffResult>> {
    if expected_node_kind(result) == NodeKind::Directory
        && let Some(source_path) = &result.source_path
    {
        return compute_managed_dir_diffs(
            source_path,
            &result.target,
            result.local_dir.as_deref(),
            &result.exclude_patterns,
            &result.ignore_keys,
        );
    }
    Ok(Vec::new())
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
            let annotation = if let Some(target_kind) = f.target_kind
                && target_kind != f.expected_kind
            {
                format!(
                    "type mismatch: expected {}, found {}",
                    f.expected_kind, target_kind
                )
            } else {
                "modified".to_string()
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
                expected_kind: NodeKind::File,
                target_kind: Some(NodeKind::File),
            },
            FileDiffResult {
                rel_path: PathBuf::from("b.txt"),
                has_changes: false,
                source_only: false,
                diff_output: String::new(),
                expected_kind: NodeKind::File,
                target_kind: Some(NodeKind::File),
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
            expected_kind: NodeKind::File,
            target_kind: Some(NodeKind::File),
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
            expected_kind: NodeKind::File,
            target_kind: None,
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
            expected_kind: NodeKind::File,
            target_kind: Some(NodeKind::File),
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
    fn test_aggregate_dir_diff_type_mismatch() {
        let diffs = vec![FileDiffResult {
            rel_path: PathBuf::from(".prototools"),
            has_changes: true,
            source_only: false,
            diff_output: String::new(),
            expected_kind: NodeKind::File,
            target_kind: Some(NodeKind::Symlink),
        }];
        let result = aggregate_dir_diff(&diffs);
        let plain = console::strip_ansi_codes(&result.output);
        assert!(result.has_changes);
        assert!(
            plain.contains(".prototools")
                && plain.contains("type mismatch: expected file, found symlink"),
            "expected type-mismatch annotation, got:\n{plain}"
        );
    }

    #[test]
    fn test_aggregate_dir_diff_does_not_mix_type_and_content_diffs() {
        let diffs = vec![FileDiffResult {
            rel_path: PathBuf::from(".prototools"),
            has_changes: true,
            source_only: false,
            diff_output: String::new(),
            expected_kind: NodeKind::File,
            target_kind: Some(NodeKind::Symlink),
        }];
        let result = aggregate_dir_diff(&diffs);
        let plain = console::strip_ansi_codes(&result.output);
        assert!(plain.contains("type mismatch: expected file, found symlink"));
        assert!(!plain.contains(">> Target"));
    }

    #[cfg(unix)]
    #[test]
    fn test_managed_dir_diffs_detects_symlink_target() {
        // Per-file detection: for directory entries we must walk into the
        // target dir, not just check the dir's own type.
        let source = TempDir::new().unwrap();
        let target = TempDir::new().unwrap();
        let backing = TempDir::new().unwrap();
        std::fs::write(source.path().join("file"), "x").unwrap();
        std::fs::write(backing.path().join("file"), "x").unwrap();
        std::os::unix::fs::symlink(backing.path().join("file"), target.path().join("file"))
            .unwrap();
        assert!(
            compute_managed_dir_diffs(source.path(), target.path(), None, &[], &[])
                .unwrap()
                .iter()
                .any(FileDiffResult::has_type_mismatch)
        );
    }

    #[cfg(unix)]
    #[test]
    fn test_managed_dir_diffs_has_no_type_mismatch_when_all_regular() {
        let source = TempDir::new().unwrap();
        let target = TempDir::new().unwrap();
        std::fs::write(source.path().join("file"), "x").unwrap();
        std::fs::write(target.path().join("file"), "x").unwrap();
        assert!(
            !compute_managed_dir_diffs(source.path(), target.path(), None, &[], &[])
                .unwrap()
                .iter()
                .any(FileDiffResult::has_type_mismatch)
        );
    }

    #[cfg(unix)]
    #[test]
    fn test_managed_dir_diffs_only_inspects_nodes_present_in_source() {
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
        assert!(
            !compute_managed_dir_diffs(source.path(), target.path(), None, &[], &[])
                .unwrap()
                .iter()
                .any(FileDiffResult::has_type_mismatch)
        );
    }

    #[test]
    fn test_aggregate_dir_diff_mixed_indicators() {
        let diffs = vec![
            FileDiffResult {
                rel_path: PathBuf::from("added.txt"),
                has_changes: true,
                source_only: true,
                diff_output: String::new(),
                expected_kind: NodeKind::File,
                target_kind: None,
            },
            FileDiffResult {
                rel_path: PathBuf::from("modified.txt"),
                has_changes: true,
                source_only: false,
                diff_output: "diff".to_string(),
                expected_kind: NodeKind::File,
                target_kind: Some(NodeKind::File),
            },
            FileDiffResult {
                rel_path: PathBuf::from("stable.txt"),
                has_changes: false,
                source_only: false,
                diff_output: String::new(),
                expected_kind: NodeKind::File,
                target_kind: Some(NodeKind::File),
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
        assert!(compute_dir_file_diffs(&result).unwrap().is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn test_compute_dir_file_diffs_includes_local_overlay_nodes() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        let local = temp.path().join("local");
        let target = temp.path().join("target");
        let backing = temp.path().join("backing");
        std::fs::create_dir_all(&source).unwrap();
        std::fs::create_dir_all(&local).unwrap();
        std::fs::create_dir_all(&target).unwrap();
        std::fs::write(local.join("local.txt"), "local").unwrap();
        std::fs::write(&backing, "local").unwrap();
        std::os::unix::fs::symlink(&backing, target.join("local.txt")).unwrap();

        let result = BuildResult {
            target,
            content: String::new(),
            is_plaintext: true,
            source_path: Some(source),
            name: "target".to_string(),
            format: nickel::Format::Plaintext,
            ignore_keys: vec![],
            is_symlink: false,
            canonical_source: None,
            exclude_patterns: vec![],
            local_dir: Some(local),
            immutable: false,
        };

        let diffs = compute_dir_file_diffs(&result).unwrap();
        assert_eq!(diffs.len(), 1);
        assert!(diffs[0].has_type_mismatch());
        assert!(result_has_type_mismatch(&result).unwrap());
    }

    #[cfg(unix)]
    #[test]
    fn test_compute_dir_file_diffs_ignores_excluded_nodes() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        let target = temp.path().join("target");
        let backing = temp.path().join("backing");
        std::fs::create_dir_all(&source).unwrap();
        std::fs::create_dir_all(&target).unwrap();
        std::fs::write(source.join("excluded.txt"), "source").unwrap();
        std::fs::write(&backing, "source").unwrap();
        std::os::unix::fs::symlink(&backing, target.join("excluded.txt")).unwrap();

        let result = BuildResult {
            target,
            content: String::new(),
            is_plaintext: true,
            source_path: Some(source),
            name: "target".to_string(),
            format: nickel::Format::Plaintext,
            ignore_keys: vec![],
            is_symlink: false,
            canonical_source: None,
            exclude_patterns: vec!["excluded.txt".to_string()],
            local_dir: None,
            immutable: false,
        };

        assert!(compute_dir_file_diffs(&result).unwrap().is_empty());
        assert!(!result_has_type_mismatch(&result).unwrap());
    }

    #[test]
    fn test_compute_dir_file_diffs_propagates_invalid_exclude_pattern() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        let target = temp.path().join("target");
        std::fs::create_dir_all(&source).unwrap();
        std::fs::create_dir_all(&target).unwrap();

        let result = BuildResult {
            target,
            content: String::new(),
            is_plaintext: true,
            source_path: Some(source),
            name: "target".to_string(),
            format: nickel::Format::Plaintext,
            ignore_keys: vec![],
            is_symlink: false,
            canonical_source: None,
            exclude_patterns: vec!["[".to_string()],
            local_dir: None,
            immutable: false,
        };

        let error = compute_dir_file_diffs(&result).unwrap_err();
        assert!(error.to_string().contains("Invalid exclude glob pattern"));
        assert!(result_has_type_mismatch(&result).is_err());
        assert!(compute_diff_for_result(&result).is_err());
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
        assert!(compute_dir_file_diffs(&result).unwrap().is_empty());
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

        let diff = compute_diff_for_result(&result).unwrap();
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
        let diffs = compute_dir_file_diffs(&result).unwrap();
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
        let diffs = compute_dir_file_diffs(&result).unwrap();
        assert_eq!(diffs.len(), 1);
        assert!(!diffs[0].has_changes);
    }
}
