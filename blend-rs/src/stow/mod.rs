pub mod ignore;
mod linker;
mod tree;

use std::fmt;
use std::path::PathBuf;

pub use linker::execute_actions;
pub use tree::{stow_package, unstow_package};

#[derive(Debug, Clone)]
pub enum StowAction {
    CreateSymlink { source: PathBuf, target: PathBuf },
    RemoveSymlink(PathBuf),
    CreateDirectory(PathBuf),
    Conflict { target: PathBuf, reason: String },
}

impl fmt::Display for StowAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StowAction::CreateSymlink { source, target } => {
                write!(f, "LINK {} -> {}", target.display(), source.display())
            }
            StowAction::RemoveSymlink(path) => {
                write!(f, "UNLINK {}", path.display())
            }
            StowAction::CreateDirectory(path) => {
                write!(f, "MKDIR {}", path.display())
            }
            StowAction::Conflict { target, reason } => {
                write!(f, "CONFLICT {}: {}", target.display(), reason)
            }
        }
    }
}
