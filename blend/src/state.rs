//! Per-machine state store for sync snapshots.
//!
//! A snapshot is the bytes blend last confirmed at a deployed target. It
//! serves as the 3-way merge ancestor when sync needs to tell which side
//! (source or deployed) moved since the last sync touchpoint.
//!
//! Layout:
//!   $XDG_STATE_HOME/blend/snapshots/<order_name>/<mirrored-absolute-target>
//!   (falls back to $HOME/.local/state/blend/snapshots/...)

use std::path::{Path, PathBuf};

use anyhow::{Context as AnyhowContext, Result};
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize)]
struct StateFile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    blend_dir: Option<PathBuf>,
}

/// Per-machine snapshot store.
pub struct StateStore {
    snapshots_root: PathBuf,
}

impl StateStore {
    /// Resolve the snapshots root from the environment.
    /// Honors `XDG_STATE_HOME`; falls back to `$HOME/.local/state` on every
    /// platform (including macOS, where `dirs::state_dir()` returns `None`).
    #[cfg(test)]
    pub fn from_env() -> Self {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .expect("HOME must be set");
        Self::from_env_for_home(&home)
    }

    /// Resolve the state store from the environment, falling back under the
    /// supplied home directory when `XDG_STATE_HOME` is unset.
    pub fn from_env_for_home(home: &Path) -> Self {
        let base = std::env::var_os("XDG_STATE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".local/state"));
        Self {
            snapshots_root: base.join("blend").join("snapshots"),
        }
    }

    /// Construct with an explicit snapshots root. Test-only constructor —
    /// production code goes through `from_env`.
    #[cfg(test)]
    pub fn with_root(snapshots_root: PathBuf) -> Self {
        Self { snapshots_root }
    }

    /// Compute the snapshot file path for a given (order_name, deployed target).
    /// The result is `<snapshots_root>/<order_name>/<absolute-target-without-leading-slash>`.
    pub fn snapshot_path(&self, order_name: &str, target: &Path) -> Result<PathBuf> {
        if !target.is_absolute() {
            anyhow::bail!("snapshot target must be absolute, got {}", target.display());
        }
        let stripped = target
            .strip_prefix("/")
            .with_context(|| format!("failed to strip leading / from {}", target.display()))?;
        if stripped.components().any(|c| {
            matches!(
                c,
                std::path::Component::ParentDir | std::path::Component::CurDir
            )
        }) {
            anyhow::bail!(
                "snapshot target must not contain . or .. components: {}",
                target.display()
            );
        }
        Ok(self.snapshots_root.join(order_name).join(stripped))
    }

    /// Read the snapshot bytes for (order_name, target). Returns `Ok(None)` if
    /// no snapshot exists; `Err` only on real IO failure (permission, etc.).
    pub fn read(&self, order_name: &str, target: &Path) -> Result<Option<Vec<u8>>> {
        let path = self.snapshot_path(order_name, target)?;
        match std::fs::read(&path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(anyhow::Error::new(e)
                .context(format!("failed to read snapshot at {}", path.display()))),
        }
    }

    /// Write the snapshot for (order_name, target) atomically: write to
    /// `<path>.tmp` then rename. Creates intermediate directories. If the
    /// snapshot already exists, it is overwritten.
    pub fn write(&self, order_name: &str, target: &Path, bytes: &[u8]) -> Result<()> {
        let path = self.snapshot_path(order_name, target)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create snapshot dir {}", parent.display()))?;
        }
        let tmp = {
            let mut s = path.clone().into_os_string();
            s.push(format!(".tmp.{}", std::process::id()));
            PathBuf::from(s)
        };
        std::fs::write(&tmp, bytes)
            .with_context(|| format!("failed to write snapshot temp file {}", tmp.display()))?;
        let rename_result = std::fs::rename(&tmp, &path).with_context(|| {
            format!(
                "failed to rename snapshot {} -> {}",
                tmp.display(),
                path.display()
            )
        });
        if rename_result.is_err() {
            // Best-effort cleanup; ignore secondary errors so the original
            // failure context is preserved.
            let _ = std::fs::remove_file(&tmp);
        }
        rename_result?;
        Ok(())
    }

    /// Read the last remembered blend checkout directory.
    pub fn read_blend_dir(&self) -> Result<Option<PathBuf>> {
        Ok(self.read_state_file()?.and_then(|state| state.blend_dir))
    }

    /// Remember the blend checkout directory for future invocations outside a checkout.
    pub fn write_blend_dir(&self, blend_dir: &Path) -> Result<()> {
        let path = self.state_file_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create state dir {}", parent.display()))?;
        }
        let state = StateFile {
            blend_dir: Some(blend_dir.to_path_buf()),
        };
        let raw = serde_json::to_vec_pretty(&state)?;
        let tmp = {
            let mut s = path.clone().into_os_string();
            s.push(format!(".tmp.{}", std::process::id()));
            PathBuf::from(s)
        };
        std::fs::write(&tmp, raw)
            .with_context(|| format!("failed to write state temp file {}", tmp.display()))?;
        let rename_result = std::fs::rename(&tmp, &path).with_context(|| {
            format!(
                "failed to rename state file {} -> {}",
                tmp.display(),
                path.display()
            )
        });
        if rename_result.is_err() {
            let _ = std::fs::remove_file(&tmp);
        }
        rename_result?;
        Ok(())
    }

    fn read_state_file(&self) -> Result<Option<StateFile>> {
        let path = self.state_file_path();
        match std::fs::read_to_string(&path) {
            Ok(raw) => {
                let state = serde_json::from_str(&raw)
                    .with_context(|| format!("failed to parse state file {}", path.display()))?;
                Ok(Some(state))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(anyhow::Error::new(e)
                .context(format!("failed to read state file {}", path.display()))),
        }
    }

    pub fn state_file_path(&self) -> PathBuf {
        self.snapshots_root
            .parent()
            .expect("snapshots root should have a parent")
            .join("state.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn store_in_tempdir() -> (StateStore, TempDir) {
        let tmp = TempDir::new().unwrap();
        let store = StateStore::with_root(tmp.path().to_path_buf());
        (store, tmp)
    }

    #[test]
    fn snapshot_path_mirrors_absolute_target_under_order() {
        let store = StateStore::with_root(PathBuf::from("/tmp/state"));
        let p = store
            .snapshot_path(
                "alacritty",
                Path::new("/Users/kc/.config/alacritty/alacritty.toml"),
            )
            .unwrap();
        assert_eq!(
            p,
            PathBuf::from("/tmp/state/alacritty/Users/kc/.config/alacritty/alacritty.toml")
        );
    }

    #[test]
    fn snapshot_path_rejects_relative_target() {
        let store = StateStore::with_root(PathBuf::from("/tmp/state"));
        let err = store
            .snapshot_path("order_name", Path::new("relative/path"))
            .unwrap_err();
        assert!(format!("{err}").contains("must be absolute"));
    }

    #[test]
    fn snapshot_path_rejects_parent_dir_components() {
        let store = StateStore::with_root(PathBuf::from("/tmp/state"));
        let err = store
            .snapshot_path("order_name", Path::new("/foo/../bar"))
            .unwrap_err();
        assert!(format!("{err}").contains("must not contain"));
    }

    #[test]
    fn from_env_uses_xdg_state_home_when_set() {
        // SAFETY: tests in this module are not parallelized over this var.
        // Use a fresh, unique value to avoid clobbering.
        let prev = std::env::var_os("XDG_STATE_HOME");
        // SAFETY: single-threaded test scope.
        unsafe { std::env::set_var("XDG_STATE_HOME", "/tmp/xdg-state-test") };
        let store = StateStore::from_env();
        assert_eq!(
            store.snapshots_root,
            PathBuf::from("/tmp/xdg-state-test/blend/snapshots")
        );
        // SAFETY: restore prior env.
        unsafe {
            match prev {
                Some(v) => std::env::set_var("XDG_STATE_HOME", v),
                None => std::env::remove_var("XDG_STATE_HOME"),
            }
        }
    }

    #[test]
    fn from_env_falls_back_to_home_local_state() {
        let prev_xdg = std::env::var_os("XDG_STATE_HOME");
        let prev_home = std::env::var_os("HOME");
        // SAFETY: single-threaded test scope.
        unsafe {
            std::env::remove_var("XDG_STATE_HOME");
            std::env::set_var("HOME", "/home/test-user");
        }
        let store = StateStore::from_env();
        assert_eq!(
            store.snapshots_root,
            PathBuf::from("/home/test-user/.local/state/blend/snapshots")
        );
        // SAFETY: restore prior env.
        unsafe {
            match prev_xdg {
                Some(v) => std::env::set_var("XDG_STATE_HOME", v),
                None => std::env::remove_var("XDG_STATE_HOME"),
            }
            match prev_home {
                Some(v) => std::env::set_var("HOME", v),
                None => std::env::remove_var("HOME"),
            }
        }
    }

    #[test]
    fn from_env_for_home_falls_back_to_supplied_home() {
        let prev_xdg = std::env::var_os("XDG_STATE_HOME");
        // SAFETY: single-threaded test scope.
        unsafe {
            std::env::remove_var("XDG_STATE_HOME");
        }
        let store = StateStore::from_env_for_home(Path::new("/tmp/blend-home"));
        assert_eq!(
            store.snapshots_root,
            PathBuf::from("/tmp/blend-home/.local/state/blend/snapshots")
        );
        // SAFETY: restore prior env.
        unsafe {
            match prev_xdg {
                Some(v) => std::env::set_var("XDG_STATE_HOME", v),
                None => std::env::remove_var("XDG_STATE_HOME"),
            }
        }
    }

    #[test]
    fn blend_dir_state_roundtrips_path() {
        let tmp = TempDir::new().unwrap();
        let store = StateStore {
            snapshots_root: tmp.path().join("blend/snapshots"),
        };
        let path = Path::new("/tmp/my-dotfiles");

        assert!(store.read_blend_dir().unwrap().is_none());
        store.write_blend_dir(path).unwrap();

        assert_eq!(store.read_blend_dir().unwrap(), Some(path.to_path_buf()));
    }

    #[test]
    fn read_returns_none_when_snapshot_missing() {
        let (store, _tmp) = store_in_tempdir();
        let v = store
            .read("order_name", Path::new("/foo/bar.toml"))
            .unwrap();
        assert!(v.is_none());
    }

    #[test]
    fn write_then_read_roundtrips_bytes() {
        let (store, _tmp) = store_in_tempdir();
        let target = Path::new("/foo/bar.toml");
        let bytes = b"hello world\n";
        store.write("order_name", target, bytes).unwrap();
        let read = store.read("order_name", target).unwrap();
        assert_eq!(read.as_deref(), Some(&bytes[..]));
    }

    #[test]
    fn write_creates_intermediate_directories() {
        let (store, _tmp) = store_in_tempdir();
        let target = Path::new("/a/b/c/d/e.txt");
        store.write("order_name", target, b"x").unwrap();
        let p = store.snapshot_path("order_name", target).unwrap();
        assert!(p.exists());
    }

    #[test]
    fn write_is_idempotent_overwrites() {
        let (store, _tmp) = store_in_tempdir();
        let target = Path::new("/foo/bar.toml");
        store.write("order_name", target, b"first").unwrap();
        store.write("order_name", target, b"second").unwrap();
        let read = store.read("order_name", target).unwrap().unwrap();
        assert_eq!(read, b"second");
    }

    #[test]
    fn write_uses_atomic_temp_then_rename() {
        // The snapshot file should never coexist with a stale temp file
        // after a successful write. We assert by listing the parent dir.
        let (store, _tmp) = store_in_tempdir();
        let target = Path::new("/foo/bar.toml");
        store.write("order_name", target, b"hello").unwrap();
        let p = store.snapshot_path("order_name", target).unwrap();
        let parent = p.parent().unwrap();
        let entries: Vec<_> = std::fs::read_dir(parent)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(entries, vec!["bar.toml"]);
    }
}
