use std::path::Path;

use anyhow::Result;

use crate::output::log;

pub fn clear(path: &Path, warn_on_failure: bool) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    record_test_event(TestImmutableEventKind::Clear, path);

    if let Err(e) = platform::clear(path)
        && warn_on_failure
    {
        log::warn(&format!(
            "Failed to clear immutable flag on {}: {} — may require elevated permissions",
            path.display(),
            e
        ));
    }

    Ok(())
}

pub fn set(path: &Path) -> Result<()> {
    record_test_event(TestImmutableEventKind::Set, path);

    if let Err(e) = platform::set(path) {
        log::warn(&format!(
            "Failed to set immutable flag on {}: {} — may require elevated permissions",
            path.display(),
            e
        ));
    }

    Ok(())
}

#[cfg(target_os = "macos")]
mod platform {
    use std::ffi::CString;
    use std::mem::MaybeUninit;
    use std::os::unix::ffi::OsStrExt;
    use std::path::Path;

    use anyhow::{Context, Result};

    pub fn clear(path: &Path) -> Result<()> {
        update(path, |flags| flags & !libc::UF_IMMUTABLE)
    }

    pub fn set(path: &Path) -> Result<()> {
        update(path, |flags| flags | libc::UF_IMMUTABLE)
    }

    fn update(path: &Path, update_flags: impl FnOnce(libc::c_uint) -> libc::c_uint) -> Result<()> {
        let c_path = c_path(path)?;
        let flags = stat_flags(&c_path, path)?;
        let next_flags = update_flags(flags);

        if next_flags == flags {
            return Ok(());
        }

        let result = unsafe { libc::chflags(c_path.as_ptr(), next_flags) };
        if result == -1 {
            return Err(std::io::Error::last_os_error())
                .with_context(|| format!("chflags failed on {}", path.display()));
        }

        Ok(())
    }

    fn c_path(path: &Path) -> Result<CString> {
        CString::new(path.as_os_str().as_bytes())
            .with_context(|| format!("Path contains an interior NUL byte: {}", path.display()))
    }

    fn stat_flags(c_path: &CString, path: &Path) -> Result<libc::c_uint> {
        let mut stat = MaybeUninit::<libc::stat>::uninit();
        let result = unsafe { libc::stat(c_path.as_ptr(), stat.as_mut_ptr()) };
        if result == -1 {
            return Err(std::io::Error::last_os_error())
                .with_context(|| format!("stat failed on {}", path.display()));
        }

        Ok(unsafe { stat.assume_init().st_flags })
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use std::fs::File;
    use std::os::fd::AsRawFd;
    use std::path::Path;

    use anyhow::{Context, Result};

    const FS_IMMUTABLE_FL: libc::c_uint = 0x0000_0010;

    pub fn clear(path: &Path) -> Result<()> {
        update(path, |flags| flags & !FS_IMMUTABLE_FL)
    }

    pub fn set(path: &Path) -> Result<()> {
        update(path, |flags| flags | FS_IMMUTABLE_FL)
    }

    fn update(path: &Path, update_flags: impl FnOnce(libc::c_uint) -> libc::c_uint) -> Result<()> {
        let file = File::open(path).with_context(|| {
            format!(
                "Failed to open {} for immutable flag update",
                path.display()
            )
        })?;
        let mut flags = get_flags(&file, path)?;
        let next_flags = update_flags(flags);

        if next_flags == flags {
            return Ok(());
        }

        flags = next_flags;
        let result = unsafe { libc::ioctl(file.as_raw_fd(), libc::FS_IOC_SETFLAGS, &flags) };
        if result == -1 {
            return Err(std::io::Error::last_os_error())
                .with_context(|| format!("FS_IOC_SETFLAGS failed on {}", path.display()));
        }

        Ok(())
    }

    fn get_flags(file: &File, path: &Path) -> Result<libc::c_uint> {
        let mut flags: libc::c_uint = 0;
        let result = unsafe { libc::ioctl(file.as_raw_fd(), libc::FS_IOC_GETFLAGS, &mut flags) };
        if result == -1 {
            return Err(std::io::Error::last_os_error())
                .with_context(|| format!("FS_IOC_GETFLAGS failed on {}", path.display()));
        }

        Ok(flags)
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
mod platform {
    use std::path::Path;

    use anyhow::Result;

    pub fn clear(_path: &Path) -> Result<()> {
        Ok(())
    }

    pub fn set(_path: &Path) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TestImmutableEventKind {
    Clear,
    Set,
}

#[cfg(test)]
pub(crate) type TestImmutableEvent = (TestImmutableEventKind, std::path::PathBuf);

#[cfg(test)]
static TEST_EVENTS: std::sync::OnceLock<std::sync::Mutex<Vec<TestImmutableEvent>>> =
    std::sync::OnceLock::new();

#[cfg(test)]
fn record_test_event(kind: TestImmutableEventKind, path: &Path) {
    TEST_EVENTS
        .get_or_init(|| std::sync::Mutex::new(Vec::new()))
        .lock()
        .unwrap()
        .push((kind, path.to_path_buf()));
}

#[cfg(not(test))]
fn record_test_event(_kind: TestImmutableEventKind, _path: &Path) {}

#[cfg(not(test))]
enum TestImmutableEventKind {
    Clear,
    Set,
}

#[cfg(test)]
pub(crate) fn take_test_events() -> Vec<TestImmutableEvent> {
    let mut events = TEST_EVENTS
        .get_or_init(|| std::sync::Mutex::new(Vec::new()))
        .lock()
        .unwrap();
    std::mem::take(&mut *events)
}
