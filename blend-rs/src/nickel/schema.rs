use std::path::PathBuf;

use serde::Deserialize;

/// Output format for rendered configs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Format {
    #[default]
    Toml,
    Json,
    Yaml,
    SpaceDelimitedPairs,
    SpaceDelimitedRecord,
    EqualDelimitedRecord,
    Plaintext,
}

impl Format {
    /// Infer format from file extension
    pub fn from_path(path: &str) -> Self {
        if path.ends_with(".toml") {
            Format::Toml
        } else if path.ends_with(".json") {
            Format::Json
        } else if path.ends_with(".yaml") || path.ends_with(".yml") {
            Format::Yaml
        } else {
            Format::Plaintext
        }
    }
}

/// Condition for when to apply this package
#[derive(Debug, Clone, Deserialize, Default)]
pub struct WhenCondition {
    /// List of allowed operating systems
    #[serde(default)]
    pub os: Vec<String>,
    /// List of allowed architectures
    #[serde(default)]
    pub arch: Vec<String>,
    /// List of allowed hostnames
    #[serde(default)]
    pub hostname: Vec<String>,
}

impl WhenCondition {
    /// Check if condition matches the given metadata
    pub fn matches(&self, os: &str, arch: &str, hostname: &str) -> bool {
        let os_ok = self.os.is_empty() || self.os.iter().any(|o| o == os);
        let arch_ok = self.arch.is_empty() || self.arch.iter().any(|a| a == arch);
        let hostname_ok = self.hostname.is_empty() || self.hostname.iter().any(|h| h == hostname);
        os_ok && arch_ok && hostname_ok
    }

    /// Check if condition matches a platform (os + arch), ignoring hostname
    pub fn matches_platform(&self, os: &str, arch: &str) -> bool {
        let os_ok = self.os.is_empty() || self.os.iter().any(|o| o == os);
        let arch_ok = self.arch.is_empty() || self.arch.iter().any(|a| a == arch);
        os_ok && arch_ok
    }
}

/// Individual file/folder entry in an order
#[derive(Debug, Clone, Deserialize)]
pub struct FileEntry {
    /// Output filename OR path to external file/folder
    pub source: String,
    /// Destination prefixes (overrides global if set)
    #[serde(default)]
    pub prefix: Vec<String>,
    /// Override format (inferred from source if omitted)
    pub format: Option<Format>,
    /// Keys to exclude from diff
    #[serde(default)]
    pub ignore: Vec<String>,
    /// Per-file condition
    pub when: Option<WhenCondition>,
    /// Per-file config data (for structured output)
    pub config: Option<serde_json::Value>,
}

impl FileEntry {
    /// Get the effective format (explicit or inferred from source)
    pub fn effective_format(&self) -> Format {
        self.format.unwrap_or_else(|| Format::from_path(&self.source))
    }

    /// Generate all target paths by combining prefix + source
    /// Uses file-level prefix if set, otherwise falls back to global prefix
    pub fn target_paths(&self, global_prefix: &[String]) -> Vec<PathBuf> {
        let prefixes = if self.prefix.is_empty() {
            global_prefix
        } else {
            &self.prefix
        };

        prefixes
            .iter()
            .map(|p| {
                let mut path = PathBuf::from(p);
                path.push(&self.source);
                path
            })
            .collect()
    }

    /// Check if this entry should be applied for the given system
    pub fn should_apply(&self, os: &str, arch: &str, hostname: &str) -> bool {
        match &self.when {
            Some(when) => when.matches(os, arch, hostname),
            None => true,
        }
    }
}

/// Metadata section of order.ncl (new multi-file schema)
#[derive(Debug, Clone, Deserialize)]
pub struct OrderMeta {
    /// Array of file entries
    pub files: Vec<FileEntry>,
    /// Default prefix for all files (can be overridden per-file)
    #[serde(default)]
    pub prefix: Vec<String>,
    /// Global condition for when to apply this package
    pub when: Option<WhenCondition>,
    /// Global keys to ignore when diffing
    #[serde(default)]
    pub ignore: Vec<String>,
}

/// Full order.ncl package structure (new schema)
#[derive(Debug, Clone, Deserialize)]
pub struct OrderPackage {
    /// Package metadata
    pub blend: OrderMeta,
}

impl OrderPackage {
    /// Check if this package should be applied for the given system
    pub fn should_apply(&self, os: &str, arch: &str, hostname: &str) -> bool {
        match &self.blend.when {
            Some(when) => when.matches(os, arch, hostname),
            None => true,
        }
    }

    /// Get the global prefix
    pub fn global_prefix(&self) -> &[String] {
        &self.blend.prefix
    }

    /// Get the global ignore keys
    pub fn global_ignore(&self) -> &[String] {
        &self.blend.ignore
    }

    /// Check if this package applies on a given platform (os + arch), ignoring hostname
    pub fn applies_on_platform(&self, os: &str, arch: &str) -> bool {
        match &self.blend.when {
            Some(when) => when.matches_platform(os, arch),
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_from_path() {
        assert_eq!(Format::from_path("~/.config/starship.toml"), Format::Toml);
        assert_eq!(Format::from_path("settings.json"), Format::Json);
        assert_eq!(Format::from_path("config.yaml"), Format::Yaml);
        assert_eq!(Format::from_path("config.yml"), Format::Yaml);
        assert_eq!(Format::from_path("kitty.conf"), Format::Plaintext);
        assert_eq!(Format::from_path("init.lua"), Format::Plaintext);
    }

    #[test]
    fn test_when_condition_matches() {
        let when = WhenCondition {
            os: vec!["darwin".to_string(), "linux".to_string()],
            arch: vec![],
            hostname: vec![],
        };

        assert!(when.matches("darwin", "aarch64", "myhost"));
        assert!(when.matches("linux", "x86_64", "myhost"));
        assert!(!when.matches("windows", "x86_64", "myhost"));
    }

    #[test]
    fn test_when_condition_empty() {
        let when = WhenCondition::default();
        assert!(when.matches("darwin", "aarch64", "myhost"));
    }

    #[test]
    fn test_file_entry_target_paths_with_local_prefix() {
        let entry = FileEntry {
            source: "settings.json".to_string(),
            prefix: vec!["~/A/".to_string(), "~/B/".to_string()],
            format: None,
            ignore: vec![],
            when: None,
            config: None,
        };

        // Local prefix takes precedence over global
        let global_prefix = vec!["~/global/".to_string()];
        let paths = entry.target_paths(&global_prefix);
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0], PathBuf::from("~/A/settings.json"));
        assert_eq!(paths[1], PathBuf::from("~/B/settings.json"));
    }

    #[test]
    fn test_file_entry_target_paths_with_global_prefix() {
        let entry = FileEntry {
            source: "settings.json".to_string(),
            prefix: vec![], // Empty - should use global
            format: None,
            ignore: vec![],
            when: None,
            config: None,
        };

        let global_prefix = vec!["~/global/".to_string()];
        let paths = entry.target_paths(&global_prefix);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], PathBuf::from("~/global/settings.json"));
    }
}
