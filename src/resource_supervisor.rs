//! Native request supervision. Counters in the PDF parser are independent of
//! these process limits. Linux enforces AS/CPU; all hosts enforce wall/output
//! caps. This is not a filesystem/network sandbox or a process-tree RSS limit.
use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

pub mod pdf_confinement;

/// Forward only font/platform settings to a PDF child, excluding credentials
/// and loader hooks. Add the host's protocol handshake after this call.
pub fn configure_pdf_worker_environment(command: &mut Command) {
    command.env_clear();
    for name in [
        "PATH",
        "HOME",
        "USERPROFILE",
        "LOCALAPPDATA",
        "APPDATA",
        "SystemRoot",
        "WINDIR",
        "TEMP",
        "TMP",
        "XDG_CACHE_HOME",
        "XDG_DATA_HOME",
        "LANG",
        "LC_ALL",
        "RHWP_FONT_PATH",
        "RHWP_FONT_CATALOG_CACHE",
        "RHWP_FONT_RASTER_CACHE",
    ] {
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExecutionLimits {
    pub wall_timeout_ms: u64,
    pub cpu_seconds: u64,
    pub address_space_bytes: u64,
    pub output_bytes: u64,
    pub diagnostics_bytes: u64,
}

impl Default for ExecutionLimits {
    fn default() -> Self {
        Self {
            wall_timeout_ms: 120_000,
            cpu_seconds: 90,
            address_space_bytes: 4 * 1024 * 1024 * 1024,
            output_bytes: crate::pdf_runtime::PdfTransferLimits::standard().output_bytes as u64,
            diagnostics_bytes: crate::pdf_runtime::PdfTransferLimits::standard().diagnostics_bytes
                as u64,
        }
    }
}

impl ExecutionLimits {
    pub fn constrained() -> Self {
        Self {
            wall_timeout_ms: 30_000,
            cpu_seconds: 20,
            address_space_bytes: 1024 * 1024 * 1024,
            output_bytes: crate::pdf_runtime::PdfTransferLimits::constrained().output_bytes as u64,
            diagnostics_bytes: crate::pdf_runtime::PdfTransferLimits::constrained()
                .diagnostics_bytes as u64,
        }
    }

    pub fn validate(&self) -> io::Result<()> {
        let cap = Self::default();
        for (value, maximum) in [
            (self.wall_timeout_ms, cap.wall_timeout_ms),
            (self.cpu_seconds, cap.cpu_seconds),
            (self.address_space_bytes, cap.address_space_bytes),
            (self.output_bytes, cap.output_bytes),
            (self.diagnostics_bytes, cap.diagnostics_bytes),
        ] {
            if value == 0 || value > maximum {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "invalid execution limit",
                ));
            }
        }
        Ok(())
    }

    fn wire_bytes(&self) -> u64 {
        self.output_bytes + self.diagnostics_bytes + 8
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Metrics {
    pub wall_millis: u128,
    pub observed_peak_rss_bytes: Option<u64>,
    pub address_space_limit_enforced: bool,
    pub cpu_limit_enforced: bool,
    pub descendant_group_cleanup: bool,
    pub filesystem_network_sandbox: bool,
    pub limits: ExecutionLimits,
}

pub struct SupervisedOutput {
    pub status: ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub metrics: Metrics,
}

struct ChildGuard {
    child: Child,
    terminated: bool,
}

impl ChildGuard {
    fn terminate(&mut self) {
        if self.terminated {
            return;
        }
        self.terminated = true;
        #[cfg(unix)]
        // SAFETY: the child is the leader of the private group established at
        // spawn. A negative pid targets that group, including inherited pipes.
        unsafe {
            libc::kill(-(self.child.id() as libc::pid_t), libc::SIGKILL);
        }
        let _ = self.child.kill();
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        self.terminate();
        let _ = self.child.wait();
    }
}

fn configure(command: &mut Command, limits: ExecutionLimits) {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::process::CommandExt;
        let parent = std::process::id() as libc::pid_t;
        // SAFETY: only async-signal-safe syscalls and stack values are used in
        // this post-fork closure. No formatting, allocation or environment read.
        unsafe {
            command.pre_exec(move || {
                for (resource, requested) in [
                    (libc::RLIMIT_AS, limits.address_space_bytes),
                    (libc::RLIMIT_CPU, limits.cpu_seconds),
                    (libc::RLIMIT_CORE, 0),
                ] {
                    let mut inherited = libc::rlimit {
                        rlim_cur: 0,
                        rlim_max: 0,
                    };
                    if libc::getrlimit(resource, &mut inherited) != 0 {
                        return Err(io::Error::last_os_error());
                    }
                    let requested = libc::rlim_t::try_from(requested)
                        .map_err(|_| io::Error::from_raw_os_error(libc::EINVAL))?;
                    let maximum = requested.min(inherited.rlim_max);
                    let limit = libc::rlimit {
                        rlim_cur: maximum,
                        rlim_max: maximum,
                    };
                    if libc::setrlimit(resource, &limit) != 0 {
                        return Err(io::Error::last_os_error());
                    }
                }
                if libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) != 0
                    || libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL, 0, 0, 0) != 0
                {
                    return Err(io::Error::last_os_error());
                }
                if libc::getppid() != parent {
                    libc::_exit(1);
                }
                Ok(())
            });
        }
    }
    #[cfg(not(target_os = "linux"))]
    let _ = (command, limits);
}

#[cfg(unix)]
fn nonblocking(pipe: &impl std::os::fd::AsRawFd) -> io::Result<()> {
    let fd = pipe.as_raw_fd();
    // SAFETY: fcntl operates on an owned live child pipe; no borrowed buffer is
    // passed to C and the flags are read before adding O_NONBLOCK.
    unsafe {
        let flags = libc::fcntl(fd, libc::F_GETFL);
        if flags < 0 || libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) < 0 {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}

fn capture(
    mut reader: impl Read,
    cap: u64,
    exceeded: &AtomicBool,
    finished: &AtomicBool,
) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut buffer = [0; 8192];
    loop {
        let remaining = (cap + 1 - bytes.len() as u64).min(buffer.len() as u64) as usize;
        match reader.read(&mut buffer[..remaining]) {
            Ok(0) => break,
            Ok(count) => {
                bytes.extend_from_slice(&buffer[..count]);
                if bytes.len() as u64 > cap {
                    exceeded.store(true, Ordering::Release);
                    break;
                }
            }
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                if finished.load(Ordering::Acquire) {
                    break;
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            Err(e) => return Err(e),
        }
    }
    Ok(bytes)
}

fn send_request(mut writer: impl Write, mut bytes: &[u8], finished: &AtomicBool) -> io::Result<()> {
    while !bytes.is_empty() {
        if finished.load(Ordering::Acquire) {
            return Err(io::Error::from(io::ErrorKind::BrokenPipe));
        }
        match writer.write(bytes) {
            Ok(0) => return Err(io::Error::from(io::ErrorKind::WriteZero)),
            Ok(count) => bytes = &bytes[count..],
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(5))
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

fn observed_rss(pid: u32) -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        let status = std::fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
        let line = status
            .lines()
            .find_map(|line| line.strip_prefix("VmRSS:"))?;
        line.split_whitespace()
            .next()?
            .parse::<u64>()
            .ok()?
            .checked_mul(1024)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = pid;
        None
    }
}

/// Keep all child pipes and descendants within the request lifetime. Return no
/// partial result after a timeout, oversized pipe or I/O failure.
pub fn run(
    command: &mut Command,
    input: &[u8],
    limits: ExecutionLimits,
) -> Result<SupervisedOutput, &'static str> {
    limits.validate().map_err(|_| "PDF_RESOURCE_POLICY")?;
    configure(command, limits);
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let started = Instant::now();
    let mut child = ChildGuard {
        child: command.spawn().map_err(|_| "PDF_PROCESS_START_FAILED")?,
        terminated: false,
    };
    let stdin = child.child.stdin.take().ok_or("PDF_PROCESS_IO_FAILED")?;
    let stdout = child.child.stdout.take().ok_or("PDF_PROCESS_IO_FAILED")?;
    let stderr = child.child.stderr.take().ok_or("PDF_PROCESS_IO_FAILED")?;
    #[cfg(unix)]
    {
        nonblocking(&stdin)
            .and_then(|_| nonblocking(&stdout))
            .and_then(|_| nonblocking(&stderr))
            .map_err(|_| "PDF_PROCESS_IO_FAILED")?;
    }
    let exceeded = AtomicBool::new(false);
    let finished = AtomicBool::new(false);
    let mut peak_rss: Option<u64> = None;
    std::thread::scope(|scope| {
        let completed = &finished;
        let writer = scope.spawn(move || send_request(stdin, input, completed));
        let out = scope.spawn(|| capture(stdout, limits.wire_bytes(), &exceeded, &finished));
        let err = scope.spawn(|| capture(stderr, 64 * 1024, &exceeded, &finished));
        let status = loop {
            if let Some(rss) = observed_rss(child.child.id()) {
                peak_rss = Some(peak_rss.unwrap_or(0).max(rss));
            }
            if exceeded.load(Ordering::Acquire) {
                break Err("PDF_PROCESS_OUTPUT_LIMIT");
            }
            if started.elapsed() >= Duration::from_millis(limits.wall_timeout_ms) {
                break Err("PDF_PROCESS_TIMEOUT");
            }
            match child.child.try_wait() {
                Ok(Some(status)) => break Ok(status),
                Ok(None) => std::thread::sleep(Duration::from_millis(5)),
                Err(_) => break Err("PDF_PROCESS_IO_FAILED"),
            }
        };
        // Even a completed leader can leave a descendant holding stdout open.
        // Close the entire group before waiting for any pipe reader thread.
        child.terminate();
        let _ = child.child.wait();
        finished.store(true, Ordering::Release);
        let wrote = writer.join().map_err(|_| "PDF_PROCESS_IO_FAILED")?;
        let stdout = out
            .join()
            .map_err(|_| "PDF_PROCESS_IO_FAILED")?
            .map_err(|_| "PDF_PROCESS_IO_FAILED")?;
        let stderr = err
            .join()
            .map_err(|_| "PDF_PROCESS_IO_FAILED")?
            .map_err(|_| "PDF_PROCESS_IO_FAILED")?;
        let status = status?;
        if exceeded.load(Ordering::Acquire) {
            return Err("PDF_PROCESS_OUTPUT_LIMIT");
        }
        if status.success() && wrote.is_err() {
            return Err("PDF_PROCESS_IO_FAILED");
        }
        Ok(SupervisedOutput {
            status,
            stdout,
            stderr,
            metrics: Metrics {
                wall_millis: started.elapsed().as_millis(),
                observed_peak_rss_bytes: peak_rss,
                address_space_limit_enforced: cfg!(target_os = "linux"),
                cpu_limit_enforced: cfg!(target_os = "linux"),
                descendant_group_cleanup: cfg!(unix),
                filesystem_network_sandbox: false,
                limits,
            },
        })
    })
}
