use std::fmt;
use std::io;
use std::path::Path;

/// Filesystem node type observed without following the final path component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    File,
    Directory,
    Symlink,
    Other,
}

impl fmt::Display for NodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::File => "file",
            Self::Directory => "directory",
            Self::Symlink => "symlink",
            Self::Other => "other",
        })
    }
}

/// Return the kind of the exact path node, without following a final symlink.
/// A dangling symlink is therefore present and reported as `Symlink`.
pub fn node_kind(path: &Path) -> io::Result<Option<NodeKind>> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => {
            let file_type = metadata.file_type();
            let kind = if file_type.is_symlink() {
                NodeKind::Symlink
            } else if file_type.is_file() {
                NodeKind::File
            } else if file_type.is_dir() {
                NodeKind::Directory
            } else {
                NodeKind::Other
            };
            Ok(Some(kind))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn classifies_missing_file_and_directory_nodes() {
        let temp = TempDir::new().unwrap();
        assert_eq!(node_kind(&temp.path().join("missing")).unwrap(), None);

        let file = temp.path().join("file");
        std::fs::write(&file, "content").unwrap();
        assert_eq!(node_kind(&file).unwrap(), Some(NodeKind::File));
        assert_eq!(node_kind(temp.path()).unwrap(), Some(NodeKind::Directory));
    }

    #[cfg(unix)]
    #[test]
    fn classifies_broken_symlink_without_following_it() {
        let temp = TempDir::new().unwrap();
        let link = temp.path().join("link");
        std::os::unix::fs::symlink("missing", &link).unwrap();
        assert_eq!(node_kind(&link).unwrap(), Some(NodeKind::Symlink));
    }

    #[test]
    fn does_not_treat_an_uninspectable_child_as_missing() {
        let temp = TempDir::new().unwrap();
        let parent = temp.path().join("parent");
        std::fs::write(&parent, "not a directory").unwrap();

        assert!(node_kind(&parent.join("child")).is_err());
    }
}
