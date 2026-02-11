mod semantic;
mod text;

pub use semantic::semantic_diff;
pub use text::text_diff;

use std::path::Path;

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

/// Check whether a deployed target file is in sync with its source.
///
/// Returns:
/// - `Some(true)` if target matches source (in sync)
/// - `Some(false)` if target differs from source
/// - `None` if comparison is not applicable (directories, missing target, errors)
pub fn check_file_sync(
    pkg_dir: &Path,
    file_entry: &FileEntry,
    target: &Path,
    global_ignore: &[String],
) -> Option<bool> {
    if !target.exists() {
        return None;
    }

    let deployed = std::fs::read_to_string(target).ok()?;
    let mut ignore_keys: Vec<String> = global_ignore.to_vec();
    ignore_keys.extend(file_entry.ignore.iter().cloned());

    // Plaintext source file on disk
    let source_path = pkg_dir.join(&file_entry.source);
    if source_path.exists() {
        if source_path.is_dir() {
            return None;
        }
        let source_content = std::fs::read_to_string(&source_path).ok()?;
        let result = diff_configs(Format::Plaintext, &source_content, &deployed, &ignore_keys);
        return Some(!result.has_changes);
    }

    // Structured config rendered from nickel
    if let Some(ref config) = file_entry.config {
        let format = file_entry
            .format
            .unwrap_or_else(|| Format::from_path(&file_entry.source));
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
        | Format::Yaml
        | Format::SpaceDelimitedRecord
        | Format::EqualDelimitedRecord => semantic_diff(format, generated, deployed, ignore_keys),
        Format::SpaceDelimitedPairs | Format::Plaintext => {
            text_diff(generated, deployed, ignore_keys)
        }
    }
}
