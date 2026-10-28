//! Read-only filesystem and syscall restrictions for a fresh PDF child.
//!
//! Call before parsing or starting threads, only in a disposable worker. On
//! Linux this closes inherited descriptors above stderr and requires Landlock
//! ABI 3 plus seccomp. Other hosts report unsupported; host isolation is separate.
use std::io;
use std::path::Path;
#[cfg(all(
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Confinement {
    pub filesystem: bool,
    pub network: bool,
    pub landlock_abi: Option<u32>,
}

/// Restrict the current, fresh PDF worker. This is irreversible and must never
/// be called in the application's main process or after creating threads.
/// Returns the authorized input's preopened read-only handle. Use that handle
/// rather than reopening the path; some mounted filesystems cannot be reopened
/// through Landlock. Other inherited descriptors above stderr are closed.
pub fn confine_pdf_worker(input: &Path) -> io::Result<(Confinement, std::fs::File)> {
    #[cfg(all(
        target_os = "linux",
        any(target_arch = "x86_64", target_arch = "aarch64")
    ))]
    {
        let mut paths = crate::renderer::font_paths::search_dirs(&[]);
        paths.extend([
            PathBuf::from("/etc/fonts"),
            PathBuf::from("/usr/share/fontconfig"),
        ]);
        if let Some(home) = std::env::var_os("HOME") {
            let home = PathBuf::from(home);
            paths.extend([home.join(".fonts"), home.join(".local/share/fonts")]);
        }
        if let Some(data) = std::env::var_os("XDG_DATA_HOME") {
            paths.push(PathBuf::from(data).join("fonts"));
        }
        for name in ["RHWP_FONT_CATALOG_CACHE", "RHWP_FONT_RASTER_CACHE"] {
            if let Some(path) = std::env::var_os(name).filter(|path| !path.is_empty()) {
                paths.push(PathBuf::from(path));
            }
        }
        linux::confine(input, &paths)
    }
    #[cfg(not(all(
        target_os = "linux",
        any(target_arch = "x86_64", target_arch = "aarch64")
    )))]
    {
        let file = std::fs::File::open(input)?;
        Ok((
            Confinement {
                filesystem: false,
                network: false,
                landlock_abi: None,
            },
            file,
        ))
    }
}

#[cfg(all(
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
mod linux {
    use super::*;
    use std::ffi::CString;
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
    use std::os::unix::ffi::OsStrExt;

    const READ_FILE: u64 = 1 << 2;
    const READ_DIR: u64 = 1 << 3;
    // ABI 3 includes REFER and TRUNCATE; no write or execute rule is granted.
    const HANDLED_FS: u64 = (1 << 15) - 1;
    #[repr(C)]
    struct Ruleset {
        handled_access_fs: u64,
    }
    #[repr(C, packed)]
    struct PathRule {
        allowed_access: u64,
        parent_fd: i32,
    }

    fn unsupported(message: &'static str) -> io::Error {
        io::Error::new(io::ErrorKind::Unsupported, message)
    }

    fn allow_read(ruleset: &OwnedFd, path: &Path) -> io::Result<()> {
        let canonical = match path.canonicalize() {
            Ok(path) => path,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error),
        };
        if canonical.parent().is_none() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "filesystem root is not a font resource",
            ));
        }
        let metadata = canonical.metadata()?;
        let path = CString::new(canonical.as_os_str().as_bytes())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid read path"))?;
        // SAFETY: the NUL-terminated path and descriptor remain owned through
        // add_rule. The kernel copies the packed, fixed-width UAPI structure.
        unsafe {
            let fd = libc::open(path.as_ptr(), libc::O_PATH | libc::O_CLOEXEC);
            if fd < 0 {
                return Err(io::Error::last_os_error());
            }
            let fd = OwnedFd::from_raw_fd(fd);
            let rule = PathRule {
                allowed_access: READ_FILE | if metadata.is_dir() { READ_DIR } else { 0 },
                parent_fd: fd.as_raw_fd(),
            };
            if libc::syscall(
                libc::SYS_landlock_add_rule,
                ruleset.as_raw_fd(),
                1u32,
                &rule as *const PathRule,
                0u32,
            ) != 0
            {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(())
    }

    fn stmt(code: u16, k: u32) -> libc::sock_filter {
        libc::sock_filter {
            code,
            jt: 0,
            jf: 0,
            k,
        }
    }
    fn jump(k: u32, yes: u8, no: u8) -> libc::sock_filter {
        libc::sock_filter {
            code: 0x15,
            jt: yes,
            jf: no,
            k,
        }
    }
    fn deny(program: &mut Vec<libc::sock_filter>, number: libc::c_long, errno: u32) {
        program.push(jump(number as u32, 0, 1));
        program.push(stmt(0x06, 0x0005_0000 | errno));
    }

    fn allow_commands(
        program: &mut Vec<libc::sock_filter>,
        number: libc::c_long,
        argument: u32,
        commands: &[u32],
    ) {
        program.push(jump(number as u32, 0, (commands.len() * 2 + 2) as u8));
        program.push(stmt(0x20, 16 + argument * 8));
        for command in commands {
            program.push(jump(*command, 0, 1));
            program.push(stmt(0x06, 0x7fff_0000));
        }
        program.push(stmt(0x06, 0x0005_0000 | libc::EPERM as u32));
    }

    fn syscall_filter() -> io::Result<()> {
        #[cfg(target_arch = "x86_64")]
        const ARCH: u32 = 0xc000_003e;
        #[cfg(target_arch = "aarch64")]
        const ARCH: u32 = 0xc000_00b7;
        const ALLOW: u32 = 0x7fff_0000;
        const EPERM: u32 = 0x0005_0000 | libc::EPERM as u32;
        let mut program = vec![
            stmt(0x20, 4),
            jump(ARCH, 1, 0),
            stmt(0x06, 0x8000_0000),
            stmt(0x20, 0),
        ];
        #[cfg(target_arch = "x86_64")]
        program.extend([
            libc::sock_filter {
                code: 0x35,
                jt: 0,
                jf: 1,
                k: 0x4000_0000,
            },
            stmt(0x06, EPERM),
        ]);
        // clone3 flags are behind a pointer. ENOSYS makes libc use the older
        // clone call, whose flags can be checked without dereferencing memory.
        deny(&mut program, libc::SYS_clone3, libc::ENOSYS as u32);
        program.extend([
            jump(libc::SYS_clone as u32, 0, 4),
            stmt(0x20, 16),
            libc::sock_filter {
                code: 0x45,
                jt: 1,
                jf: 0,
                k: libc::CLONE_THREAD as u32,
            },
            stmt(0x06, EPERM),
            stmt(0x06, ALLOW),
            jump(libc::SYS_tgkill as u32, 0, 4),
            stmt(0x20, 16),
            jump(std::process::id(), 1, 0),
            stmt(0x06, EPERM),
            stmt(0x06, ALLOW),
            jump(libc::SYS_prlimit64 as u32, 0, 5),
            stmt(0x20, 16),
            jump(0, 2, 0),
            jump(std::process::id(), 1, 0),
            stmt(0x06, EPERM),
            stmt(0x06, ALLOW),
        ]);
        // Each argument-inspecting branch returns on a match and leaves the
        // syscall number untouched on a nonmatch. Unknown calls fail closed.
        allow_commands(
            &mut program,
            libc::SYS_prctl,
            0,
            &[
                libc::PR_SET_NAME as u32,
                libc::PR_GET_NAME as u32,
                libc::PR_GET_PDEATHSIG as u32,
                libc::PR_GET_DUMPABLE as u32,
                libc::PR_GET_SECCOMP as u32,
                libc::PR_GET_NO_NEW_PRIVS as u32,
            ],
        );
        allow_commands(
            &mut program,
            libc::SYS_fcntl,
            1,
            &[
                libc::F_DUPFD as u32,
                libc::F_DUPFD_CLOEXEC as u32,
                libc::F_GETFD as u32,
                libc::F_SETFD as u32,
                libc::F_GETFL as u32,
                libc::F_SETFL as u32,
            ],
        );
        let allowed = [
            libc::SYS_read,
            libc::SYS_write,
            libc::SYS_readv,
            libc::SYS_writev,
            libc::SYS_pread64,
            libc::SYS_openat,
            libc::SYS_openat2,
            libc::SYS_close,
            libc::SYS_fstat,
            libc::SYS_newfstatat,
            libc::SYS_statx,
            libc::SYS_lseek,
            libc::SYS_getdents64,
            libc::SYS_readlinkat,
            libc::SYS_faccessat,
            libc::SYS_faccessat2,
            libc::SYS_brk,
            libc::SYS_mmap,
            libc::SYS_mprotect,
            libc::SYS_munmap,
            libc::SYS_mremap,
            libc::SYS_madvise,
            libc::SYS_futex,
            libc::SYS_futex_waitv,
            libc::SYS_membarrier,
            libc::SYS_set_robust_list,
            libc::SYS_set_tid_address,
            libc::SYS_rseq,
            libc::SYS_rt_sigaction,
            libc::SYS_rt_sigprocmask,
            libc::SYS_rt_sigreturn,
            libc::SYS_sigaltstack,
            libc::SYS_clock_gettime,
            libc::SYS_clock_nanosleep,
            libc::SYS_nanosleep,
            libc::SYS_gettimeofday,
            libc::SYS_getrusage,
            libc::SYS_times,
            libc::SYS_getpid,
            libc::SYS_getppid,
            libc::SYS_gettid,
            libc::SYS_getuid,
            libc::SYS_geteuid,
            libc::SYS_getgid,
            libc::SYS_getegid,
            libc::SYS_sched_getaffinity,
            libc::SYS_sched_yield,
            libc::SYS_uname,
            libc::SYS_sysinfo,
            libc::SYS_capget,
            libc::SYS_pipe2,
            libc::SYS_eventfd2,
            libc::SYS_ppoll,
            libc::SYS_epoll_create1,
            libc::SYS_epoll_ctl,
            libc::SYS_epoll_pwait,
            libc::SYS_epoll_pwait2,
            libc::SYS_dup,
            libc::SYS_dup3,
            libc::SYS_getrandom,
            libc::SYS_getcwd,
            libc::SYS_fstatfs,
            libc::SYS_statfs,
            libc::SYS_exit,
            libc::SYS_exit_group,
            libc::SYS_restart_syscall,
        ];
        for number in allowed {
            program.push(jump(number as u32, 0, 1));
            program.push(stmt(0x06, ALLOW));
        }
        #[cfg(target_arch = "x86_64")]
        for number in [
            libc::SYS_open,
            libc::SYS_stat,
            libc::SYS_lstat,
            libc::SYS_access,
            libc::SYS_readlink,
            libc::SYS_getdents,
            libc::SYS_getrlimit,
            libc::SYS_poll,
            libc::SYS_select,
            libc::SYS_time,
            libc::SYS_arch_prctl,
            libc::SYS_dup2,
            libc::SYS_pipe,
        ] {
            program.push(jump(number as u32, 0, 1));
            program.push(stmt(0x06, ALLOW));
        }
        program.push(stmt(0x06, EPERM));
        let filter = libc::sock_fprog {
            len: program.len() as u16,
            filter: program.as_mut_ptr(),
        };
        // SAFETY: the kernel copies the live BPF array; no pointers from a
        // document are inspected. Called before any worker thread is started.
        if unsafe { libc::prctl(libc::PR_SET_SECCOMP, 2, &filter as *const libc::sock_fprog) } != 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    pub(super) fn confine(
        input: &Path,
        paths: &[PathBuf],
    ) -> io::Result<(Confinement, std::fs::File)> {
        // All threads must inherit both restrictions. Refuse a late call.
        if std::fs::read_dir("/proc/self/task")?.count() != 1 {
            return Err(unsupported(
                "PDF confinement requires a fresh single-threaded child",
            ));
        }
        // Protocol descriptors must be pipes, not ambient files or sockets.
        // Closing descriptors above stderr alone would not constrain 0..2.
        for fd in 0..=2 {
            let mut metadata = std::mem::MaybeUninit::<libc::stat>::uninit();
            // SAFETY: fstat initializes the output on success.
            if unsafe { libc::fstat(fd, metadata.as_mut_ptr()) } != 0 {
                return Err(io::Error::last_os_error());
            }
            if unsafe { metadata.assume_init() }.st_mode & libc::S_IFMT != libc::S_IFIFO {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "PDF worker stdin/stdout/stderr must be protocol pipes",
                ));
            }
        }
        // SAFETY: version query has no pointer arguments. close_range removes
        // inherited handles; only this disposable child's protocol uses 0..2.
        unsafe {
            let abi = libc::syscall(
                libc::SYS_landlock_create_ruleset,
                std::ptr::null::<u8>(),
                0usize,
                1u32,
            );
            if abi < 3 {
                return Err(unsupported("PDF confinement requires Landlock ABI 3"));
            }
            if libc::syscall(libc::SYS_close_range, 3u32, u32::MAX, 0u32) != 0 {
                return Err(io::Error::last_os_error());
            }
            // Preopen only the authorized PDF after removing ambient handles.
            // Landlock cannot reopen every mounted filesystem (e.g. WSL DrvFs).
            // No document parsing occurs before restrictions are installed.
            let input_file = std::fs::File::open(input)?;
            if !input_file.metadata()?.is_file() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "PDF input must be a regular file",
                ));
            }
            let attr = Ruleset {
                handled_access_fs: HANDLED_FS,
            };
            let fd = libc::syscall(
                libc::SYS_landlock_create_ruleset,
                &attr as *const Ruleset,
                std::mem::size_of::<Ruleset>(),
                0u32,
            );
            if fd < 0 {
                return Err(io::Error::last_os_error());
            }
            let ruleset = OwnedFd::from_raw_fd(fd as i32);
            for path in paths {
                allow_read(&ruleset, path)?;
            }
            // Empty effective/permitted/inheritable capability sets before
            // processing documents, including when the host started as root.
            let parent = libc::getppid();
            let capability_header = [0x2008_0522u32, 0];
            let capability_data = [0u32; 6];
            if libc::syscall(
                libc::SYS_capset,
                capability_header.as_ptr(),
                capability_data.as_ptr(),
            ) != 0
            {
                return Err(io::Error::last_os_error());
            }
            // Credential changes can clear PDEATHSIG. Restore it after the
            // capability drop and close the intervening parent-death race.
            if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL, 0, 0, 0) != 0 {
                return Err(io::Error::last_os_error());
            }
            if libc::getppid() != parent {
                libc::_exit(1);
            }
            if libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) != 0 {
                return Err(io::Error::last_os_error());
            }
            if libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0) != 0 {
                return Err(io::Error::last_os_error());
            }
            if libc::syscall(libc::SYS_landlock_restrict_self, ruleset.as_raw_fd(), 0u32) != 0 {
                return Err(io::Error::last_os_error());
            }
            syscall_filter()?;
            Ok((
                Confinement {
                    filesystem: true,
                    network: true,
                    landlock_abi: Some(abi as u32),
                },
                input_file,
            ))
        }
    }
}
