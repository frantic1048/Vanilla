use serde::Serialize;
use std::path::PathBuf;

/// System metadata available to Nickel configs via blend://metadata
#[derive(Debug, Clone, Serialize)]
pub struct Metadata {
    /// Operating system: "darwin", "linux", "windows"
    pub os: String,
    /// CPU architecture: "aarch64", "x86_64"
    pub arch: String,
    /// System hostname
    pub hostname: String,
    /// Desktop environment (Linux only): "gnome", "kde", "sway", etc.
    pub desktop: Option<String>,
    /// User's home directory
    pub home: PathBuf,
    /// Current username
    pub user: String,
}

impl Metadata {
    /// Detect system metadata
    pub fn detect(home_dir: &PathBuf) -> Self {
        Self {
            os: detect_os(),
            arch: detect_arch(),
            hostname: detect_hostname(),
            desktop: detect_desktop(),
            home: home_dir.clone(),
            user: detect_user(),
        }
    }

    /// Convert to JSON value for Nickel injection
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_default()
    }
}

fn detect_os() -> String {
    #[cfg(target_os = "macos")]
    return "darwin".to_string();

    #[cfg(target_os = "linux")]
    return "linux".to_string();

    #[cfg(target_os = "windows")]
    return "windows".to_string();

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    return std::env::consts::OS.to_string();
}

fn detect_arch() -> String {
    std::env::consts::ARCH.to_string()
}

fn detect_hostname() -> String {
    hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "unknown".to_string())
}

fn detect_desktop() -> Option<String> {
    // Check common environment variables for desktop detection
    std::env::var("XDG_CURRENT_DESKTOP")
        .ok()
        .or_else(|| std::env::var("DESKTOP_SESSION").ok())
        .map(|d| d.to_lowercase())
}

fn detect_user() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_metadata() {
        let home = PathBuf::from("/tmp/test");
        let meta = Metadata::detect(&home);

        assert!(!meta.os.is_empty());
        assert!(!meta.arch.is_empty());
        assert!(!meta.hostname.is_empty());
        assert_eq!(meta.home, home);
    }

    #[test]
    fn test_to_json() {
        let home = PathBuf::from("/tmp/test");
        let meta = Metadata::detect(&home);
        let json = meta.to_json();

        assert!(json.get("os").is_some());
        assert!(json.get("arch").is_some());
        assert!(json.get("hostname").is_some());
    }
}
