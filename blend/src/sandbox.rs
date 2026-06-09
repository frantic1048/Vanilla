use anyhow::Result;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum SandboxMode {
    Force,
    #[default]
    Prefer,
    Never,
}

/// Install blend's process sandbox.
///
/// The first hardening layer deliberately stays narrow: deny later process
/// execution and socket-based networking, while leaving filesystem access to
/// blend's existing ownership and path checks.
pub fn install() -> Result<()> {
    platform::install()
}

#[cfg(debug_assertions)]
pub fn run_probe_from_env() -> Result<()> {
    let Ok(probe) = std::env::var("BLEND_SANDBOX_PROBE") else {
        return Ok(());
    };

    match probe.as_str() {
        "exec" => probe::exec(),
        other => anyhow::bail!("unknown BLEND_SANDBOX_PROBE={other:?}; expected \"exec\""),
    }
}

#[cfg(debug_assertions)]
mod probe {
    use std::process::Command;

    use anyhow::{Context, Result, bail};

    pub fn exec() -> Result<()> {
        match Command::new("/bin/sh").arg("-c").arg("true").status() {
            Ok(status) => bail!("exec probe unexpectedly succeeded with status {status}"),
            Err(e) if is_permission_denied(&e) => Ok(()),
            Err(e) => Err(e).context("exec probe failed for an unexpected reason"),
        }
    }

    fn is_permission_denied(error: &std::io::Error) -> bool {
        error.kind() == std::io::ErrorKind::PermissionDenied
            || error.raw_os_error() == Some(libc::EPERM)
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use anyhow::{Context, Result};

    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    pub fn install() -> Result<()> {
        install_seccomp()
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    pub fn install() -> Result<()> {
        anyhow::bail!(
            "seccomp sandbox is not wired for Linux architecture {}",
            std::env::consts::ARCH
        );
    }

    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    fn install_seccomp() -> Result<()> {
        let mut filter = vec![
            stmt(
                libc::BPF_LD | libc::BPF_W | libc::BPF_ABS,
                SECCOMP_DATA_ARCH_OFFSET,
            ),
            jump(
                libc::BPF_JMP | libc::BPF_JEQ | libc::BPF_K,
                AUDIT_ARCH,
                1,
                0,
            ),
            stmt(libc::BPF_RET | libc::BPF_K, libc::SECCOMP_RET_KILL_PROCESS),
            stmt(
                libc::BPF_LD | libc::BPF_W | libc::BPF_ABS,
                SECCOMP_DATA_NR_OFFSET,
            ),
        ];

        for &syscall in denied_syscalls() {
            filter.push(jump(
                libc::BPF_JMP | libc::BPF_JEQ | libc::BPF_K,
                syscall as u32,
                0,
                1,
            ));
            filter.push(stmt(
                libc::BPF_RET | libc::BPF_K,
                libc::SECCOMP_RET_ERRNO | libc::EPERM as u32,
            ));
        }

        filter.push(stmt(libc::BPF_RET | libc::BPF_K, libc::SECCOMP_RET_ALLOW));

        let mut prog = libc::sock_fprog {
            len: filter
                .len()
                .try_into()
                .context("seccomp filter is too large")?,
            filter: filter.as_mut_ptr(),
        };

        let no_new_privs = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
        if no_new_privs == -1 {
            return Err(std::io::Error::last_os_error())
                .context("prctl(PR_SET_NO_NEW_PRIVS) failed");
        }

        let seccomp = unsafe {
            libc::prctl(
                libc::PR_SET_SECCOMP,
                libc::SECCOMP_MODE_FILTER,
                &mut prog as *mut libc::sock_fprog,
            )
        };
        if seccomp == -1 {
            return Err(std::io::Error::last_os_error()).context("prctl(PR_SET_SECCOMP) failed");
        }

        Ok(())
    }

    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    fn denied_syscalls() -> &'static [libc::c_long] {
        &[
            libc::SYS_execve,
            libc::SYS_execveat,
            libc::SYS_socket,
            libc::SYS_socketpair,
            libc::SYS_connect,
            libc::SYS_bind,
            libc::SYS_listen,
            libc::SYS_accept,
            libc::SYS_accept4,
            libc::SYS_sendto,
            libc::SYS_recvfrom,
            libc::SYS_sendmsg,
            libc::SYS_recvmsg,
            libc::SYS_sendmmsg,
            libc::SYS_recvmmsg,
            libc::SYS_setsockopt,
            libc::SYS_getsockopt,
            libc::SYS_getsockname,
            libc::SYS_getpeername,
            libc::SYS_shutdown,
        ]
    }

    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    fn stmt(code: u32, k: u32) -> libc::sock_filter {
        libc::sock_filter {
            code: code as u16,
            jt: 0,
            jf: 0,
            k,
        }
    }

    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    fn jump(code: u32, k: u32, jt: u8, jf: u8) -> libc::sock_filter {
        libc::sock_filter {
            code: code as u16,
            jt,
            jf,
            k,
        }
    }

    // Linux UAPI: `struct seccomp_data` in include/uapi/linux/seccomp.h.
    // Classic BPF loads fields by byte offset from this struct.
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    const SECCOMP_DATA_NR_OFFSET: u32 = 0;
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    const SECCOMP_DATA_ARCH_OFFSET: u32 = 4;

    // Linux UAPI: `AUDIT_ARCH_*` in include/uapi/linux/audit.h.
    // Values are ELF EM_* machine ids OR'd with audit ABI flags.
    // libc does not expose these audit constants consistently, so keep the
    // small set blend supports here with their source formula visible.
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    const AUDIT_ARCH_64BIT: u32 = 0x8000_0000;
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    const AUDIT_ARCH_LE: u32 = 0x4000_0000;

    // Linux UAPI: `EM_*` values from include/uapi/linux/elf-em.h.
    #[cfg(target_arch = "x86_64")]
    const EM_X86_64: u32 = 62;
    #[cfg(target_arch = "aarch64")]
    const EM_AARCH64: u32 = 183;

    #[cfg(target_arch = "x86_64")]
    const AUDIT_ARCH: u32 = AUDIT_ARCH_64BIT | AUDIT_ARCH_LE | EM_X86_64;
    #[cfg(target_arch = "aarch64")]
    const AUDIT_ARCH: u32 = AUDIT_ARCH_64BIT | AUDIT_ARCH_LE | EM_AARCH64;
}

#[cfg(target_os = "macos")]
mod platform {
    use std::ffi::{CStr, CString};
    use std::os::raw::{c_char, c_int};

    use anyhow::{Context, Result};

    const PROFILE: &str = r#"
(version 1)
(allow default)
(deny process-exec)
(deny network*)
"#;

    pub fn install() -> Result<()> {
        let profile = CString::new(PROFILE).context("sandbox profile contains NUL byte")?;
        let mut error: *mut c_char = std::ptr::null_mut();
        let result = unsafe { sandbox_init(profile.as_ptr(), 0, &mut error) };

        if result == -1 {
            let message = if error.is_null() {
                "unknown sandbox_init error".to_string()
            } else {
                let message = unsafe { CStr::from_ptr(error) }
                    .to_string_lossy()
                    .into_owned();
                unsafe { sandbox_free_error(error) };
                message
            };
            anyhow::bail!("sandbox_init failed: {message}");
        }

        Ok(())
    }

    #[link(name = "sandbox")]
    unsafe extern "C" {
        fn sandbox_init(profile: *const c_char, flags: u64, errorbuf: *mut *mut c_char) -> c_int;
        fn sandbox_free_error(errorbuf: *mut c_char);
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
mod platform {
    use anyhow::{Result, bail};

    pub fn install() -> Result<()> {
        bail!(
            "process sandbox is not supported on {}",
            std::env::consts::OS
        );
    }
}
