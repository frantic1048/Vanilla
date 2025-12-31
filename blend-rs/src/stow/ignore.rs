use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::Path;

pub struct IgnorePatterns {
    globset: GlobSet,
    simple_patterns: Vec<String>,
}

impl IgnorePatterns {
    pub fn new() -> Self {
        Self {
            globset: GlobSet::empty(),
            simple_patterns: Vec::new(),
        }
    }

    pub fn should_ignore(&self, path: &Path) -> bool {
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        // Always ignore these files
        if file_name == ".DS_Store"
            || file_name == ".stow-local-ignore"
            || file_name == ".stow-global-ignore"
            || file_name == "program.home.nix"
        {
            return true;
        }

        // Check globset patterns
        if self.globset.is_match(path) || self.globset.is_match(&file_name) {
            return true;
        }

        // Check simple string patterns (for regex-like patterns we can't convert)
        for pattern in &self.simple_patterns {
            if file_name.contains(pattern) {
                return true;
            }
        }

        false
    }
}

impl Default for IgnorePatterns {
    fn default() -> Self {
        Self::new()
    }
}

pub fn load_ignore_patterns(packages_dir: &Path, package: &str) -> IgnorePatterns {
    let mut builder = GlobSetBuilder::new();
    let mut simple_patterns = Vec::new();

    // Load global ignore from packages/stow/.stow-global-ignore
    let global_ignore = packages_dir.join("stow").join(".stow-global-ignore");
    if global_ignore.exists() {
        load_ignore_file(&mut builder, &mut simple_patterns, &global_ignore);
    }

    // Load local ignore from packages/<package>/.stow-local-ignore
    let local_ignore = packages_dir.join(package).join(".stow-local-ignore");
    if local_ignore.exists() {
        load_ignore_file(&mut builder, &mut simple_patterns, &local_ignore);
    }

    IgnorePatterns {
        globset: builder.build().unwrap_or_else(|_| GlobSet::empty()),
        simple_patterns,
    }
}

fn load_ignore_file(builder: &mut GlobSetBuilder, simple: &mut Vec<String>, path: &Path) {
    let Ok(content) = std::fs::read_to_string(path) else {
        return;
    };

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Convert stow regex patterns to glob patterns where possible
        if let Some(glob) = convert_stow_pattern_to_glob(line) {
            if let Ok(g) = Glob::new(&glob) {
                builder.add(g);
            }
        } else {
            // Fall back to simple substring matching for complex patterns
            let simplified = simplify_pattern(line);
            if !simplified.is_empty() {
                simple.push(simplified);
            }
        }
    }
}

/// Convert stow's Perl-style regex patterns to glob patterns
fn convert_stow_pattern_to_glob(pattern: &str) -> Option<String> {
    let mut p = pattern.to_string();

    // Remove common regex anchors
    p = p.trim_start_matches("^/").to_string();
    p = p.trim_start_matches('^').to_string();
    p = p.trim_end_matches('$').to_string();

    // Skip patterns with complex regex syntax
    if p.contains("(?")
        || p.contains("\\d")
        || p.contains("\\w")
        || p.contains("\\s")
        || p.contains('[')
        || p.contains('(')
        || p.contains('|')
    {
        return None;
    }

    // Convert regex to glob
    p = p.replace("\\.", ".");
    p = p.replace(".+", "*");
    p = p.replace(".*", "*");
    p = p.replace("\\#", "#");
    p = p.replace("\\/", "/");

    // If pattern still has problematic backslashes, skip it
    if p.contains('\\') {
        return None;
    }

    Some(p)
}

/// Simplify a regex pattern to a basic substring for fallback matching
fn simplify_pattern(pattern: &str) -> String {
    let mut p = pattern.to_string();

    p = p.replace("^/", "");
    p = p.replace('^', "");
    p = p.replace('$', "");
    p = p.replace("\\.", ".");
    p = p.replace(".+", "");
    p = p.replace(".*", "");
    p = p.replace("\\#", "#");
    p = p.replace('\\', "");

    p.trim().to_string()
}
