use std::path::PathBuf;

use crate::cli::Cli;
use crate::metadata::Metadata;

/// Runtime context for blend operations
pub struct Context {
    pub home_dir: PathBuf,
    pub orders_dir: PathBuf,
    pub dry_run: bool,
    pub verbose: bool,
    pub metadata: Metadata,
}

impl Context {
    pub fn new(cli: &Cli) -> Self {
        let home_dir = cli
            .home
            .clone()
            .unwrap_or_else(|| dirs::home_dir().expect("Could not determine home directory"));

        let orders_dir = cli.orders.clone().unwrap_or_else(find_orders_dir);
        let metadata = Metadata::detect(&home_dir);

        Self {
            home_dir,
            orders_dir,
            dry_run: cli.dry_run,
            verbose: cli.verbose,
            metadata,
        }
    }

    /// Get the Vanilla repo root (parent of orders_dir)
    pub fn repo_root(&self) -> PathBuf {
        self.orders_dir
            .parent()
            .expect("orders_dir should have a parent")
            .to_path_buf()
    }

    /// Expand ~ in a string path
    pub fn expand_path_str(&self, path: &str) -> PathBuf {
        if path.starts_with("~/") {
            self.home_dir.join(&path[2..])
        } else if path == "~" {
            self.home_dir.clone()
        } else {
            PathBuf::from(path)
        }
    }

    /// Expand ~ in a PathBuf
    pub fn expand_path(&self, path: &std::path::Path) -> PathBuf {
        self.expand_path_str(&path.to_string_lossy())
    }
}

fn find_orders_dir() -> PathBuf {
    // Find orders directory relative to executable or current directory
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));

    // Try relative to exe first, then current dir
    for base in [exe_dir, Some(std::env::current_dir().unwrap_or_default())]
        .into_iter()
        .flatten()
    {
        let orders_dir = base.join("orders");
        if orders_dir.is_dir() {
            return orders_dir;
        }
        // Also check parent (in case we're in blend-rs/)
        if let Some(parent) = base.parent() {
            let orders_dir = parent.join("orders");
            if orders_dir.is_dir() {
                return orders_dir;
            }
        }
    }

    // Default to sibling orders dir
    PathBuf::from("../orders")
}
