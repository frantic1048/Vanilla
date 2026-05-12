use std::path::PathBuf;

use anyhow::{Result, bail};

use crate::cli::{Cli, Commands};
use crate::metadata::Metadata;
use crate::state::StateStore;

/// Runtime context for blend operations
pub struct Context {
    pub home_dir: PathBuf,
    pub blend_dir: PathBuf,
    pub orders_dir: PathBuf,
    pub dry_run: bool,
    pub verbose: bool,
    pub metadata: Metadata,
    pub state: StateStore,
}

impl Context {
    pub fn new(cli: &Cli) -> Result<Self> {
        let home_dir = cli.home.clone().unwrap_or_else(|| {
            PathBuf::from(std::env::var("HOME").expect("Could not determine home directory"))
        });

        let blend_dir = cli.blend_dir.clone().map(Ok).unwrap_or_else(|| {
            if matches!(cli.command, Some(Commands::Init)) {
                find_blend_dir_from_current_dir()
                    .or_else(|_| std::env::current_dir().map_err(Into::into))
            } else {
                find_blend_dir()
            }
        })?;
        let orders_dir = blend_dir.join("orders");
        let metadata = Metadata::detect(&home_dir);
        let state = StateStore::from_env();

        Ok(Self {
            home_dir,
            blend_dir,
            orders_dir,
            dry_run: cli.dry_run,
            verbose: cli.verbose,
            metadata,
            state,
        })
    }

    /// Expand ~ in a string path
    pub fn expand_path_str(&self, path: &str) -> PathBuf {
        if let Some(stripped) = path.strip_prefix("~/") {
            self.home_dir.join(stripped)
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

fn find_blend_dir() -> Result<PathBuf> {
    // Find a blend directory relative to current directory or executable.
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));

    for base in [std::env::current_dir().ok(), exe_dir]
        .into_iter()
        .flatten()
    {
        for candidate in base.ancestors() {
            if candidate.join("orders").is_dir() {
                return Ok(candidate.to_path_buf());
            }
        }
    }

    bail!("Could not find blend directory. Run from a blend checkout or pass --blend-dir <PATH>.")
}

fn find_blend_dir_from_current_dir() -> Result<PathBuf> {
    let current_dir = std::env::current_dir().unwrap_or_default();
    for candidate in current_dir.ancestors() {
        if candidate.join("orders").is_dir() {
            return Ok(candidate.to_path_buf());
        }
    }

    bail!("Could not find blend directory from current directory.")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use clap::Parser;
    use tempfile::TempDir;

    #[test]
    fn context_carries_a_state_store_resolved_from_env() {
        let tmp = TempDir::new().unwrap();
        let xdg = TempDir::new().unwrap();
        // SAFETY: serial test setup; restore via guard after.
        let prev_xdg = std::env::var_os("XDG_STATE_HOME");
        unsafe { std::env::set_var("XDG_STATE_HOME", xdg.path()) };

        let cli = Cli::parse_from([
            "blend",
            "--blend-dir",
            tmp.path().to_str().unwrap(),
            "--home",
            tmp.path().to_str().unwrap(),
        ]);
        let ctx = Context::new(&cli).unwrap();
        assert_eq!(ctx.blend_dir, tmp.path());
        assert_eq!(ctx.orders_dir, tmp.path().join("orders"));

        // Round-trip a snapshot through ctx.state to prove it's wired.
        let target = tmp.path().join("some-target");
        ctx.state.write("order_name", &target, b"hello").unwrap();
        let got = ctx.state.read("order_name", &target).unwrap().unwrap();
        assert_eq!(got, b"hello");

        // Snapshot should land under the XDG override, not the user's real state dir.
        let p = ctx.state.snapshot_path("order_name", &target).unwrap();
        assert!(
            p.starts_with(xdg.path()),
            "expected snapshot under {} but got {}",
            xdg.path().display(),
            p.display()
        );

        unsafe {
            match prev_xdg {
                Some(v) => std::env::set_var("XDG_STATE_HOME", v),
                None => std::env::remove_var("XDG_STATE_HOME"),
            }
        }
    }
}
