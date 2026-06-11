use std::path::{Path, PathBuf};

use anyhow::{Context as AnyhowContext, Result, bail};
use serde::{Deserialize, Serialize};

use crate::cli::{Cli, Commands};
use crate::metadata::Metadata;
use crate::output::log;
use crate::sandbox::SandboxMode;
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
    update_config_after_success: bool,
}

impl Context {
    pub fn new(cli: &Cli) -> Result<Self> {
        let home_dir = home_dir_from_cli(cli);
        let state = StateStore::from_env_for_home(&home_dir);

        let blend_dir_choice = resolve_blend_dir(cli, &state)?;
        let blend_dir = blend_dir_choice.path;
        let orders_dir = blend_dir.join("orders");
        let metadata = Metadata::detect(&home_dir);

        Ok(Self {
            home_dir,
            blend_dir,
            orders_dir,
            dry_run: cli.dry_run,
            verbose: cli.verbose,
            metadata,
            state,
            update_config_after_success: blend_dir_choice.update_config_after_success,
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

    pub fn update_config_after_success(&self) -> Result<()> {
        if !self.update_config_after_success || self.dry_run {
            return Ok(());
        }

        self.state.write_blend_dir(&self.blend_dir)?;
        if self.verbose {
            log::info(&format!(
                "Updated blend dir state: {}",
                self.state.state_file_path().display()
            ));
        }

        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
struct BlendConfig {
    #[serde(default)]
    sandbox: SandboxMode,
}

struct BlendDirChoice {
    path: PathBuf,
    update_config_after_success: bool,
}

fn resolve_blend_dir(cli: &Cli, state: &StateStore) -> Result<BlendDirChoice> {
    if let Some(blend_dir) = &cli.blend_dir {
        return Ok(BlendDirChoice {
            path: blend_dir.clone(),
            update_config_after_success: false,
        });
    }

    if matches!(cli.command, Some(Commands::Init)) {
        if let Some(current) = find_blend_dir_from_current_dir() {
            return choice_from_current_dir(state, current);
        }

        if let Some(remembered) = state.read_blend_dir()? {
            return Ok(BlendDirChoice {
                path: remembered,
                update_config_after_success: false,
            });
        }

        return Ok(BlendDirChoice {
            path: std::env::current_dir()?,
            update_config_after_success: true,
        });
    }

    find_blend_dir(state)
}

fn find_blend_dir(state: &StateStore) -> Result<BlendDirChoice> {
    if let Some(current) = find_blend_dir_from_current_dir() {
        return choice_from_current_dir(state, current);
    }

    if let Some(remembered) = state.read_blend_dir()? {
        return Ok(BlendDirChoice {
            path: remembered,
            update_config_after_success: false,
        });
    }

    bail!("Could not find blend directory. Run from a blend checkout or pass --blend-dir <PATH>.")
}

fn choice_from_current_dir(state: &StateStore, current: PathBuf) -> Result<BlendDirChoice> {
    let remembered = state.read_blend_dir()?;
    let update_config_after_success = remembered
        .as_ref()
        .is_none_or(|remembered| !same_path(remembered, &current));

    if let Some(remembered) = remembered
        && remembered.join("orders").is_dir()
        && !same_path(&remembered, &current)
    {
        log::warn(&format!(
            "warning: current blend dir {} differs from remembered blend-dir {}; using current directory",
            current.display(),
            remembered.display()
        ));
    }

    Ok(BlendDirChoice {
        path: current,
        update_config_after_success,
    })
}

fn config_path(home_dir: &Path) -> PathBuf {
    home_dir.join(".config/blend/config.toml")
}

pub fn home_dir_from_cli(cli: &Cli) -> PathBuf {
    cli.home.clone().unwrap_or_else(|| {
        PathBuf::from(std::env::var("HOME").expect("Could not determine home directory"))
    })
}

pub fn sandbox_mode_from_cli_and_config(cli: &Cli) -> Result<SandboxMode> {
    if let Some(sandbox) = cli.sandbox {
        return Ok(sandbox);
    }

    let home_dir = home_dir_from_cli(cli);
    Ok(read_blend_config(&home_dir)?
        .map(|config| config.sandbox)
        .unwrap_or_default())
}

fn read_blend_config(home_dir: &Path) -> Result<Option<BlendConfig>> {
    let path = config_path(home_dir);
    if !path.exists() {
        return Ok(None);
    }

    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let config: BlendConfig =
        toml::from_str(&raw).with_context(|| format!("Failed to parse {}", path.display()))?;

    Ok(Some(config))
}

fn find_blend_dir_from_current_dir() -> Option<PathBuf> {
    let current_dir = std::env::current_dir().ok()?;
    for candidate in current_dir.ancestors() {
        if candidate.join("orders").is_dir() {
            return Some(candidate.to_path_buf());
        }
    }

    None
}

fn same_path(a: &Path, b: &Path) -> bool {
    let normalize = |path: &Path| path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    normalize(a) == normalize(b)
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

    #[test]
    fn sandbox_mode_defaults_to_prefer_when_config_is_absent() {
        let tmp = TempDir::new().unwrap();
        let cli = Cli::parse_from([
            "blend",
            "--home",
            tmp.path().to_str().unwrap(),
            "--blend-dir",
            tmp.path().to_str().unwrap(),
        ]);

        assert_eq!(
            sandbox_mode_from_cli_and_config(&cli).unwrap(),
            SandboxMode::Prefer
        );
    }

    #[test]
    fn sandbox_mode_reads_config() {
        let home = TempDir::new().unwrap();
        let config_dir = home.path().join(".config/blend");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(
            config_dir.join("config.toml"),
            "sandbox = \"never\"\n",
        )
        .unwrap();
        let cli = Cli::parse_from(["blend", "--home", home.path().to_str().unwrap()]);

        assert_eq!(
            sandbox_mode_from_cli_and_config(&cli).unwrap(),
            SandboxMode::Never
        );
    }

    #[test]
    fn sandbox_cli_flags_override_config() {
        let home = TempDir::new().unwrap();
        let config_dir = home.path().join(".config/blend");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(
            config_dir.join("config.toml"),
            "sandbox = \"never\"\n",
        )
        .unwrap();
        let cli = Cli::parse_from([
            "blend",
            "--home",
            home.path().to_str().unwrap(),
            "--sandbox",
            "force",
        ]);

        assert_eq!(
            sandbox_mode_from_cli_and_config(&cli).unwrap(),
            SandboxMode::Force
        );
    }
}
