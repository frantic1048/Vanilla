mod semantic;
mod text;

#[allow(unused_imports)]
pub use semantic::{
    KeyChange, KeyChangeType, key_change_with_base_display, semantic_diff, semantic_diff_keys,
    semantic_diff_with_base,
};
pub use text::{text_diff, text_diff_with_base};

use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::formats::get_renderer;
use crate::nickel::{FileEntry, Format};

/// Result of comparing two configs
#[derive(Debug)]
pub struct DiffResult {
    /// Human-readable diff output
    pub output: String,
    /// Whether there are any differences
    pub has_changes: bool,
}

impl DiffResult {
    pub fn no_changes() -> Self {
        Self {
            output: String::new(),
            has_changes: false,
        }
    }

    pub fn with_changes(output: String) -> Self {
        Self {
            output,
            has_changes: true,
        }
    }
}

/// Result of comparing a single file within a directory
#[derive(Debug)]
pub struct FileDiffResult {
    /// Relative path within the directory
    pub rel_path: PathBuf,
    /// Whether the file has changes (or is new/missing)
    pub has_changes: bool,
    /// Diff output if changes exist
    pub diff_output: String,
    /// Whether the file only exists in the source (not yet deployed)
    pub source_only: bool,
    /// Whether the deployed target is a symlink. Set even when resolved
    /// content matches the source, so callers can flag legacy stow leftovers
    /// that need to be replaced with a real file on the next sync.
    pub target_is_symlink: bool,
}

/// Enumerate files in the source directory and compare each against the target.
///
/// Files present only in the target are intentionally ignored — blend only manages
/// files originating from the source side, and walking large deployed directories
/// (e.g. `~/.proto`, `~/.cache`) was the dominant cost of the status command.
pub fn diff_directory(
    source_dir: &Path,
    target_dir: &Path,
    ignore_patterns: &[String],
) -> Vec<FileDiffResult> {
    let mut results = Vec::new();

    if !source_dir.is_dir() {
        return results;
    }

    for entry in WalkDir::new(source_dir).min_depth(1).sort_by_file_name() {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if entry.file_type().is_dir() {
            continue;
        }
        let rel_path = match entry.path().strip_prefix(source_dir) {
            Ok(r) => r.to_path_buf(),
            Err(_) => continue,
        };

        let target_file = target_dir.join(&rel_path);
        if !target_file.exists() {
            results.push(FileDiffResult {
                rel_path,
                has_changes: true,
                diff_output: String::new(),
                source_only: true,
                target_is_symlink: false,
            });
            continue;
        }

        // Inspect the target file type without following symlinks. A target
        // that is a symlink is structurally wrong (orders deploy real files)
        // and must be flagged for redeploy regardless of resolved content.
        let target_is_symlink = std::fs::symlink_metadata(&target_file)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false);

        // Both exist: compare contents
        let source_content = match std::fs::read_to_string(entry.path()) {
            Ok(c) => c,
            Err(_) => {
                // Binary file — compare raw bytes
                let source_bytes = std::fs::read(entry.path()).unwrap_or_default();
                let target_bytes = std::fs::read(&target_file).unwrap_or_default();
                results.push(FileDiffResult {
                    rel_path,
                    has_changes: target_is_symlink || source_bytes != target_bytes,
                    diff_output: String::new(),
                    source_only: false,
                    target_is_symlink,
                });
                continue;
            }
        };
        let target_content = match std::fs::read_to_string(&target_file) {
            Ok(c) => c,
            Err(_) => {
                // Target is binary, source is text — they differ
                results.push(FileDiffResult {
                    rel_path,
                    has_changes: true,
                    diff_output: String::new(),
                    source_only: false,
                    target_is_symlink,
                });
                continue;
            }
        };

        let diff = text_diff(&source_content, &target_content, ignore_patterns);
        results.push(FileDiffResult {
            rel_path,
            has_changes: target_is_symlink || diff.has_changes,
            diff_output: diff.output,
            source_only: false,
            target_is_symlink,
        });
    }

    results
}

/// Check whether a deployed target file is in sync with its source.
///
/// Returns:
/// - `Some(true)` if target matches source (in sync)
/// - `Some(false)` if target differs from source
/// - `None` if comparison is not applicable (directories, missing target, errors)
pub fn check_file_sync(
    order_dir: &Path,
    file_entry: &FileEntry,
    target: &Path,
    global_ignore: &[String],
) -> Option<bool> {
    if file_entry.symlink {
        return None; // Symlinks are checked separately in cmd_status
    }

    if !target.exists() {
        return None;
    }

    let mut ignore_keys: Vec<String> = global_ignore.to_vec();
    ignore_keys.extend(file_entry.ignore.iter().cloned());

    if let Some(file) = &file_entry.from_file {
        // Plaintext source file on disk
        let source_path = order_dir.join(file);
        if source_path.is_dir() {
            // For directories, check all files within
            let file_diffs = diff_directory(&source_path, target, &ignore_keys);
            if file_diffs.is_empty() {
                return None;
            }
            let all_in_sync = file_diffs.iter().all(|f| !f.has_changes);
            return Some(all_in_sync);
        }
        let source_content = std::fs::read_to_string(&source_path).ok()?;
        let deployed = std::fs::read_to_string(target).ok()?;
        let result = diff_configs(Format::Plaintext, &source_content, &deployed, &ignore_keys);
        return Some(!result.has_changes);
    }

    let deployed = std::fs::read_to_string(target).ok()?;

    if let Some(config) = &file_entry.from_config {
        // Structured config rendered from nickel
        let format = file_entry.effective_format();
        let rendered = get_renderer(format).render(config).ok()?;
        let result = diff_configs(format, &rendered, &deployed, &ignore_keys);
        return Some(!result.has_changes);
    }

    None
}

/// Diff two configs based on format
pub fn diff_configs(
    format: Format,
    generated: &str,
    deployed: &str,
    ignore_keys: &[String],
) -> DiffResult {
    match format {
        Format::Toml
        | Format::Json
        | Format::Jsonc
        | Format::Yaml
        | Format::SpaceRecordLines
        | Format::EqualsRecordLines => semantic_diff(format, generated, deployed, ignore_keys),
        Format::SpacePairLines | Format::Plaintext => text_diff(generated, deployed, ignore_keys),
    }
}

/// Diff two configs against a Base snapshot when one is available.
pub fn diff_configs_with_base(
    format: Format,
    generated: &str,
    deployed: &str,
    base: &str,
    ignore_keys: &[String],
) -> DiffResult {
    match format {
        Format::Toml
        | Format::Json
        | Format::Jsonc
        | Format::Yaml
        | Format::SpaceRecordLines
        | Format::EqualsRecordLines => {
            semantic_diff_with_base(format, generated, deployed, base, ignore_keys)
        }
        Format::SpacePairLines | Format::Plaintext => {
            text_diff_with_base(generated, deployed, base, ignore_keys)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_file(base: &Path, relative: &str, content: &str) {
        let path = base.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, content).unwrap();
    }

    #[test]
    fn test_diff_directory_both_identical() {
        let source = TempDir::new().unwrap();
        let target = TempDir::new().unwrap();
        write_file(source.path(), "a.txt", "hello\n");
        write_file(source.path(), "b.txt", "world\n");
        write_file(target.path(), "a.txt", "hello\n");
        write_file(target.path(), "b.txt", "world\n");
        let results = diff_directory(source.path(), target.path(), &[]);
        assert_eq!(results.len(), 2);
        for r in &results {
            assert!(!r.has_changes);
            assert!(!r.source_only);
        }
    }

    #[test]
    fn test_diff_directory_source_only_file() {
        let source = TempDir::new().unwrap();
        let target = TempDir::new().unwrap();
        write_file(source.path(), "only_in_source.txt", "data\n");
        write_file(source.path(), "shared.txt", "same\n");
        write_file(target.path(), "shared.txt", "same\n");
        let results = diff_directory(source.path(), target.path(), &[]);
        assert_eq!(results.len(), 2);
        let source_only = results
            .iter()
            .find(|r| r.rel_path.as_path() == Path::new("only_in_source.txt"))
            .unwrap();
        assert!(source_only.has_changes);
        assert!(source_only.source_only);
    }

    #[test]
    fn test_diff_directory_ignores_extra_target_files() {
        // Files present only in the target are intentionally ignored —
        // blend only manages files that originate from the source side.
        let source = TempDir::new().unwrap();
        let target = TempDir::new().unwrap();
        write_file(source.path(), "shared.txt", "same\n");
        write_file(target.path(), "shared.txt", "same\n");
        write_file(target.path(), "only_in_target.txt", "extra\n");
        let results = diff_directory(source.path(), target.path(), &[]);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].rel_path.as_path(), Path::new("shared.txt"));
        assert!(!results[0].has_changes);
    }

    #[test]
    fn test_diff_directory_file_differs() {
        let source = TempDir::new().unwrap();
        let target = TempDir::new().unwrap();
        write_file(source.path(), "config.txt", "key=new_value\n");
        write_file(target.path(), "config.txt", "key=old_value\n");
        let results = diff_directory(source.path(), target.path(), &[]);
        assert_eq!(results.len(), 1);
        assert!(results[0].has_changes);
        assert!(!results[0].diff_output.is_empty());
    }

    #[test]
    fn test_diff_directory_nested_subdirectories() {
        let source = TempDir::new().unwrap();
        let target = TempDir::new().unwrap();
        write_file(source.path(), "a/b/c.txt", "deep\n");
        write_file(source.path(), "a/d.txt", "shallow\n");
        write_file(target.path(), "a/b/c.txt", "deep\n");
        write_file(target.path(), "a/d.txt", "different\n");
        let results = diff_directory(source.path(), target.path(), &[]);
        assert_eq!(results.len(), 2);
        let deep = results
            .iter()
            .find(|r| r.rel_path.as_path() == Path::new("a/b/c.txt"))
            .unwrap();
        assert!(!deep.has_changes);
        let shallow = results
            .iter()
            .find(|r| r.rel_path.as_path() == Path::new("a/d.txt"))
            .unwrap();
        assert!(shallow.has_changes);
    }

    #[test]
    fn test_diff_directory_empty_directories() {
        let source = TempDir::new().unwrap();
        let target = TempDir::new().unwrap();
        std::fs::create_dir_all(source.path().join("empty_sub")).unwrap();
        std::fs::create_dir_all(target.path().join("empty_sub")).unwrap();
        let results = diff_directory(source.path(), target.path(), &[]);
        assert!(results.is_empty());
    }

    #[test]
    fn test_diff_directory_binary_files_differ() {
        let source = TempDir::new().unwrap();
        let target = TempDir::new().unwrap();
        std::fs::write(source.path().join("image.bin"), [0x00, 0xFF, 0x80, 0x01]).unwrap();
        std::fs::write(target.path().join("image.bin"), [0x00, 0xFF, 0x80, 0x02]).unwrap();
        let results = diff_directory(source.path(), target.path(), &[]);
        assert_eq!(results.len(), 1);
        assert!(results[0].has_changes);
        assert!(results[0].diff_output.is_empty());
    }

    #[test]
    fn test_diff_directory_binary_files_identical() {
        let source = TempDir::new().unwrap();
        let target = TempDir::new().unwrap();
        let bytes = [0x00, 0xFF, 0x80, 0x01];
        std::fs::write(source.path().join("data.bin"), bytes).unwrap();
        std::fs::write(target.path().join("data.bin"), bytes).unwrap();
        let results = diff_directory(source.path(), target.path(), &[]);
        assert_eq!(results.len(), 1);
        assert!(!results[0].has_changes);
    }

    #[test]
    fn test_diff_directory_ignore_patterns() {
        let source = TempDir::new().unwrap();
        let target = TempDir::new().unwrap();
        write_file(source.path(), "conf", "stable=1\nvolatile=aaa\n");
        write_file(target.path(), "conf", "stable=1\nvolatile=zzz\n");
        let results = diff_directory(source.path(), target.path(), &[]);
        assert!(results[0].has_changes);
        let results = diff_directory(source.path(), target.path(), &["^volatile".to_string()]);
        assert!(!results[0].has_changes);
    }

    #[test]
    fn test_diff_directory_source_does_not_exist() {
        // Without a source dir there is nothing to diff against — blend never
        // reports unmanaged target files.
        let target = TempDir::new().unwrap();
        write_file(target.path(), "a.txt", "hello\n");
        let fake_source = target.path().join("nonexistent");
        let results = diff_directory(&fake_source, target.path(), &[]);
        assert!(results.is_empty());
    }

    #[test]
    fn test_diff_directory_target_does_not_exist() {
        let source = TempDir::new().unwrap();
        write_file(source.path(), "a.txt", "hello\n");
        let fake_target = source.path().join("nonexistent");
        let results = diff_directory(source.path(), &fake_target, &[]);
        assert_eq!(results.len(), 1);
        assert!(results[0].source_only);
    }

    /// Legacy stow leftover: target file is a symlink whose resolved content
    /// matches the source. Even though contents agree, the deployed file's
    /// type is wrong and a redeploy is required.
    #[cfg(unix)]
    #[test]
    fn test_diff_directory_target_symlink_matching_content() {
        let source = TempDir::new().unwrap();
        let target = TempDir::new().unwrap();
        let backing = TempDir::new().unwrap();
        write_file(source.path(), "config", "key=value\n");
        write_file(backing.path(), "config", "key=value\n");
        std::os::unix::fs::symlink(backing.path().join("config"), target.path().join("config"))
            .unwrap();

        let results = diff_directory(source.path(), target.path(), &[]);
        assert_eq!(results.len(), 1);
        assert!(
            results[0].target_is_symlink,
            "target file is a symlink — must be flagged"
        );
        assert!(
            results[0].has_changes,
            "symlink target must be reported as needing redeploy even if resolved content matches"
        );
    }

    #[cfg(unix)]
    #[test]
    fn test_diff_directory_target_symlink_differing_content() {
        let source = TempDir::new().unwrap();
        let target = TempDir::new().unwrap();
        let backing = TempDir::new().unwrap();
        write_file(source.path(), "config", "key=new\n");
        write_file(backing.path(), "config", "key=old\n");
        std::os::unix::fs::symlink(backing.path().join("config"), target.path().join("config"))
            .unwrap();

        let results = diff_directory(source.path(), target.path(), &[]);
        assert_eq!(results.len(), 1);
        assert!(results[0].target_is_symlink);
        assert!(results[0].has_changes);
    }

    #[cfg(unix)]
    #[test]
    fn test_diff_directory_regular_target_not_marked_symlink() {
        let source = TempDir::new().unwrap();
        let target = TempDir::new().unwrap();
        write_file(source.path(), "config", "key=value\n");
        write_file(target.path(), "config", "key=value\n");

        let results = diff_directory(source.path(), target.path(), &[]);
        assert_eq!(results.len(), 1);
        assert!(!results[0].target_is_symlink);
        assert!(!results[0].has_changes);
    }
}
