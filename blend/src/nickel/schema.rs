use std::path::PathBuf;

use anyhow::{Result, bail};
use serde::Deserialize;

/// Output format for rendered configs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Format {
    #[default]
    Toml,
    Json,
    Jsonc,
    Yaml,
    SpacePairLines,
    SpaceRecordLines,
    EqualsRecordLines,
    Plaintext,
}

impl Format {
    /// Infer format from file extension
    pub fn from_path(path: &str) -> Self {
        if path.ends_with(".toml") {
            Format::Toml
        } else if path.ends_with(".jsonc") {
            Format::Jsonc
        } else if path.ends_with(".json") {
            Format::Json
        } else if path.ends_with(".yaml") || path.ends_with(".yml") {
            Format::Yaml
        } else {
            Format::Plaintext
        }
    }
}

/// Condition for when to apply this order
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
    /// Destination filename, combined with prefix for target path
    #[serde(default)]
    pub name: String,
    /// External file/folder to copy (plaintext mode)
    pub from_file: Option<String>,
    /// Inline structured config data
    pub from_config: Option<serde_json::Value>,
    /// Destination prefixes (overrides global if set)
    #[serde(default)]
    pub prefix: Vec<String>,
    /// Override format (inferred from name if omitted)
    pub format: Option<Format>,
    /// Keys to exclude from diff
    #[serde(default)]
    pub ignore: Vec<String>,
    /// Per-file condition
    pub when: Option<WhenCondition>,
    /// Create symlink instead of copying (from_file only)
    #[serde(default)]
    pub symlink: bool,
    /// Glob patterns to exclude when shipping a from_file directory
    #[serde(default)]
    pub exclude: Vec<String>,
    /// Path to a local overlay directory (gitignored) that overrides/extends
    /// the main from_file directory. Only valid for directory from_file entries.
    pub local: Option<String>,
    /// Set OS immutable flag on the deployed file after writing
    #[serde(default)]
    pub immutable: bool,
}

impl FileEntry {
    /// Resolve default values after deserialization.
    ///
    /// - If `name` is empty and `from_file` is Some, set `name = from_file`
    /// - If `name` is empty and only `from_config` is set, error
    /// - If both `from_file` and `from_config` are set, error
    /// - If neither is set, error
    pub fn resolve_defaults(&mut self) -> Result<()> {
        if self.from_file.is_some() && self.from_config.is_some() {
            bail!("file entry has both 'from_file' and 'from_config' set; exactly one is required");
        }
        if self.from_file.is_none() && self.from_config.is_none() {
            bail!(
                "file entry has neither 'from_file' nor 'from_config' set; exactly one is required"
            );
        }
        if self.symlink && self.from_config.is_some() {
            bail!("'symlink' can only be used with 'from_file', not 'from_config'");
        }
        if self.local.is_some() && self.from_file.is_none() {
            bail!("'local' can only be used with 'from_file', not 'from_config'");
        }
        if self.name.is_empty() {
            if let Some(from_file) = &self.from_file {
                self.name.clone_from(from_file);
            } else {
                bail!("'name' is required when using 'from_config'");
            }
        }
        Ok(())
    }

    /// Get the effective format (explicit or inferred from name)
    pub fn effective_format(&self) -> Format {
        self.format.unwrap_or_else(|| Format::from_path(&self.name))
    }

    /// Generate all target paths by combining prefix + name
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
                path.push(&self.name);
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
    #[serde(default)]
    pub files: Vec<FileEntry>,
    /// Default prefix for all files (can be overridden per-file)
    #[serde(default)]
    pub prefix: Vec<String>,
    /// Global condition for when to apply this order
    pub when: Option<WhenCondition>,
    /// Global keys to ignore when diffing
    #[serde(default)]
    pub ignore: Vec<String>,
}

/// Full order.ncl order structure (new schema)
#[derive(Debug, Clone, Deserialize)]
pub struct Order {
    /// Order metadata
    pub blend: OrderMeta,
}

impl Order {
    /// Check if this order should be applied for the given system
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

    /// Check if this order applies on a given platform (os + arch), ignoring hostname
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
            name: "settings.json".to_string(),
            from_file: None,
            from_config: Some(serde_json::json!({})),
            prefix: vec!["~/A/".to_string(), "~/B/".to_string()],
            format: None,
            ignore: vec![],
            when: None,
            symlink: false,
            exclude: vec![],
            local: None,
            immutable: false,
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
            name: "settings.json".to_string(),
            from_file: None,
            from_config: Some(serde_json::json!({})),
            prefix: vec![], // Empty - should use global
            format: None,
            ignore: vec![],
            when: None,
            symlink: false,
            exclude: vec![],
            local: None,
            immutable: false,
        };

        let global_prefix = vec!["~/global/".to_string()];
        let paths = entry.target_paths(&global_prefix);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], PathBuf::from("~/global/settings.json"));
    }

    #[test]
    fn test_resolve_defaults_from_file() {
        let mut entry = FileEntry {
            name: String::new(),
            from_file: Some("nvim".to_string()),
            from_config: None,
            prefix: vec![],
            format: None,
            ignore: vec![],
            when: None,
            symlink: false,
            exclude: vec![],
            local: None,
            immutable: false,
        };
        entry.resolve_defaults().unwrap();
        assert_eq!(entry.name, "nvim");
    }

    #[test]
    fn test_resolve_defaults_from_config_requires_name() {
        let mut entry = FileEntry {
            name: String::new(),
            from_file: None,
            from_config: Some(serde_json::json!({})),
            prefix: vec![],
            format: None,
            ignore: vec![],
            when: None,
            symlink: false,
            exclude: vec![],
            local: None,
            immutable: false,
        };
        assert!(entry.resolve_defaults().is_err());
    }

    #[test]
    fn test_resolve_defaults_both_set_errors() {
        let mut entry = FileEntry {
            name: "test".to_string(),
            from_file: Some("test".to_string()),
            from_config: Some(serde_json::json!({})),
            prefix: vec![],
            format: None,
            ignore: vec![],
            when: None,
            symlink: false,
            exclude: vec![],
            local: None,
            immutable: false,
        };
        assert!(entry.resolve_defaults().is_err());
    }

    #[test]
    fn test_resolve_defaults_neither_set_errors() {
        let mut entry = FileEntry {
            name: "test".to_string(),
            from_file: None,
            from_config: None,
            prefix: vec![],
            format: None,
            ignore: vec![],
            when: None,
            symlink: false,
            exclude: vec![],
            local: None,
            immutable: false,
        };
        assert!(entry.resolve_defaults().is_err());
    }

    #[test]
    fn test_resolve_defaults_symlink_with_from_config_errors() {
        let mut entry = FileEntry {
            name: "test".to_string(),
            from_file: None,
            from_config: Some(serde_json::json!({})),
            prefix: vec![],
            format: None,
            ignore: vec![],
            when: None,
            symlink: true,
            exclude: vec![],
            local: None,
            immutable: false,
        };
        assert!(entry.resolve_defaults().is_err());
    }

    #[test]
    fn test_resolve_defaults_symlink_with_from_file_ok() {
        let mut entry = FileEntry {
            name: String::new(),
            from_file: Some("bin".to_string()),
            from_config: None,
            prefix: vec![],
            format: None,
            ignore: vec![],
            when: None,
            symlink: true,
            exclude: vec![],
            local: None,
            immutable: false,
        };
        entry.resolve_defaults().unwrap();
        assert_eq!(entry.name, "bin");
    }

    #[test]
    fn test_resolve_defaults_local_with_from_config_errors() {
        let mut entry = FileEntry {
            name: "test".to_string(),
            from_file: None,
            from_config: Some(serde_json::json!({})),
            prefix: vec![],
            format: None,
            ignore: vec![],
            when: None,
            symlink: false,
            exclude: vec![],
            local: Some("test.local".to_string()),
            immutable: false,
        };
        assert!(entry.resolve_defaults().is_err());
    }

    #[test]
    fn test_resolve_defaults_local_with_from_file_ok() {
        let mut entry = FileEntry {
            name: String::new(),
            from_file: Some("elvish".to_string()),
            from_config: None,
            prefix: vec![],
            format: None,
            ignore: vec![],
            when: None,
            symlink: false,
            exclude: vec![],
            local: Some("elvish.local".to_string()),
            immutable: false,
        };
        entry.resolve_defaults().unwrap();
        assert_eq!(entry.name, "elvish");
    }
}
