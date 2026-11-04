//! A native Rust host with one disposable, confined child per PDF request.
//! input.pdf output.hwpx [--verify]; verification uses synthetic private data.
#[cfg(target_arch = "wasm32")]
fn main() {
    eprintln!("pdf_isolation_host requires a native host");
}
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    native::main();
}
#[cfg(not(target_arch = "wasm32"))]
mod native {
    use rhwp::resource_supervisor::{self, pdf_confinement::confine_pdf_worker, ExecutionLimits};
    use std::io::{self, Read, Write};
    use std::path::Path;
    use std::process::Command;

    pub fn main() {
        if let Err(error) = run() {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }

    fn run() -> Result<(), Box<dyn std::error::Error>> {
        let args: Vec<_> = std::env::args_os().collect();
        if args.get(1).is_some_and(|arg| arg == "--worker") {
            let input = Path::new(args.get(2).ok_or("missing input")?);
            #[cfg(target_os = "linux")]
            let inherited = args
                .get(4)
                .map(|arg| {
                    arg.to_str()
                        .ok_or("invalid descriptor")?
                        .parse::<i32>()
                        .map_err(|_| "invalid descriptor")
                })
                .transpose()?;
            #[cfg(target_os = "linux")]
            if let Some(fd) = inherited {
                // SAFETY: inspect only the synthetic descriptor prepared by the parent.
                assert!(unsafe { libc::fcntl(fd, libc::F_GETFD) } >= 0);
            }
            let (confinement, mut input_file) = confine_pdf_worker(input)?;
            #[cfg(target_os = "linux")]
            if let Some(fd) = inherited {
                assert_eq!(unsafe { libc::fcntl(fd, libc::F_GETFD) }, -1);
                assert_eq!(io::Error::last_os_error().raw_os_error(), Some(libc::EBADF));
            }
            if !confinement.filesystem || !confinement.network {
                return Err("this example requires native Linux confinement".into());
            }
            if let Some(forbidden) = args.get(3) {
                verify_restrictions(&mut input_file, input, Path::new(forbidden))?;
            }
            let policy = rhwp::parser::pdf::PdfImportPolicy::default();
            let mut bytes = Vec::new();
            input_file
                .take(policy.max_input_bytes() as u64 + 1)
                .read_to_end(&mut bytes)?;
            let mut request = rhwp::PdfImportSession::new(policy)?;
            let mut core = request.open(&bytes, &Default::default())?;
            let output = request.export_hwpx(&mut core, 128 * 1024 * 1024)?;
            io::stdout().lock().write_all(&output)?;
            eprintln!(
                "PDF_ISOLATION_REPORT {}",
                serde_json::to_string(&confinement)?
            );
            return Ok(());
        }
        let input = Path::new(
            args.get(1)
                .ok_or("usage: pdf_isolation_host input.pdf output.hwpx [--verify]")?,
        );
        let output = Path::new(args.get(2).ok_or("missing output")?);
        let mut command = Command::new(std::env::current_exe()?);
        resource_supervisor::configure_pdf_worker_environment(&mut command);
        command.arg("--worker").arg(input);
        let fixture = output.with_extension(format!("isolation-check-{}", std::process::id()));
        let verify = args.get(3).is_some_and(|arg| arg == "--verify");
        let mut inherited_fixture = None;
        let fixture_guard = if verify {
            let mut file = private_file(&fixture)?;
            let guard = RemoveFile(fixture.clone());
            file.write_all(b"synthetic private fixture")?;
            assert_eq!(std::fs::read(&fixture)?, b"synthetic private fixture");
            let _ = std::net::TcpListener::bind("127.0.0.1:0")?;
            let _ = std::net::UdpSocket::bind("127.0.0.1:0")?;
            command.arg(&fixture);
            #[cfg(target_os = "linux")]
            {
                use std::os::fd::{AsRawFd, FromRawFd};
                let fd = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_DUPFD, 128) };
                if fd < 0 {
                    return Err(io::Error::last_os_error().into());
                }
                file = unsafe { std::fs::File::from_raw_fd(fd) };
                // SAFETY: verification deliberately inherits only this mock
                // fixture to confirm the worker closes ambient descriptors.
                assert_eq!(unsafe { libc::fcntl(fd, libc::F_SETFD, 0) }, 0);
                command.arg(fd.to_string());
            }
            inherited_fixture = Some(file);
            Some(guard)
        } else {
            None
        };
        let result = resource_supervisor::run(&mut command, &[], ExecutionLimits::default())?;
        if !result.status.success() {
            return Err(
                format!("worker failed: {}", String::from_utf8_lossy(&result.stderr)).into(),
            );
        }
        assert!(
            result.stdout.starts_with(b"PK\x03\x04"),
            "worker did not produce HWPX"
        );
        let stderr = String::from_utf8_lossy(&result.stderr);
        let report = stderr
            .lines()
            .find_map(|line| line.strip_prefix("PDF_ISOLATION_REPORT "))
            .ok_or("missing confinement report")?;
        let report: resource_supervisor::pdf_confinement::Confinement =
            serde_json::from_str(report)?;
        assert!(report.filesystem && report.network);
        if verify {
            assert_eq!(std::fs::read(&fixture)?, b"synthetic private fixture");
            assert!(!fixture.with_extension("forbidden-write").exists());
        }
        let temporary = output.with_extension(format!("part-{}", std::process::id()));
        let mut file = private_file(&temporary)?;
        let guard = RemoveFile(temporary.clone());
        file.write_all(&result.stdout)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&temporary, output)?;
        drop(guard);
        drop(inherited_fixture);
        drop(fixture_guard);
        eprintln!("{}", stderr.trim());
        Ok(())
    }

    fn private_file(path: &Path) -> io::Result<std::fs::File> {
        let mut options = std::fs::OpenOptions::new();
        options.create_new(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        options.open(path)
    }

    struct RemoveFile(std::path::PathBuf);
    impl Drop for RemoveFile {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    #[cfg(target_os = "linux")]
    fn verify_restrictions(
        input_file: &mut std::fs::File,
        input: &Path,
        forbidden: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut signature = [0u8; 5];
        input_file.read_exact(&mut signature)?;
        assert_eq!(&signature, b"%PDF-");
        use std::io::{Seek, SeekFrom};
        input_file.seek(SeekFrom::Start(0))?;
        assert_eq!(
            std::fs::read(forbidden).unwrap_err().kind(),
            io::ErrorKind::PermissionDenied
        );
        assert_eq!(
            std::fs::OpenOptions::new()
                .write(true)
                .open(input)
                .unwrap_err()
                .kind(),
            io::ErrorKind::PermissionDenied
        );
        assert_eq!(
            std::fs::write(forbidden.with_extension("forbidden-write"), b"blocked")
                .unwrap_err()
                .kind(),
            io::ErrorKind::PermissionDenied
        );
        assert_eq!(
            std::net::TcpStream::connect("127.0.0.1:9")
                .unwrap_err()
                .raw_os_error(),
            Some(libc::EPERM)
        );
        assert_eq!(
            std::net::UdpSocket::bind("127.0.0.1:0")
                .unwrap_err()
                .raw_os_error(),
            Some(libc::EPERM)
        );
        assert!(Command::new("/bin/true").status().is_err());
        #[cfg(target_os = "linux")]
        unsafe {
            // SAFETY: no foreign pointers are dereferenced. Exit an unexpectedly
            // permitted child immediately; the parent fails the verification.
            let mut death_signal = 0i32;
            assert_eq!(libc::prctl(libc::PR_GET_PDEATHSIG, &mut death_signal), 0);
            assert_eq!(death_signal, libc::SIGKILL);
            assert_eq!(libc::prctl(libc::PR_SET_PDEATHSIG, 0, 0, 0, 0), -1);
            assert_eq!(io::Error::last_os_error().raw_os_error(), Some(libc::EPERM));
            let header = [0x2008_0522u32, 0];
            let mut caps = [u32::MAX; 6];
            assert_eq!(
                libc::syscall(libc::SYS_capget, header.as_ptr(), caps.as_mut_ptr()),
                0
            );
            assert_eq!(caps, [0; 6]);
            let mut limit = libc::rlimit {
                rlim_cur: 0,
                rlim_max: 0,
            };
            assert_eq!(
                libc::prlimit(0, libc::RLIMIT_AS, std::ptr::null(), &mut limit),
                0
            );
            assert_eq!(
                libc::prlimit(
                    libc::getppid(),
                    libc::RLIMIT_AS,
                    std::ptr::null(),
                    &mut limit
                ),
                -1
            );
            assert_eq!(io::Error::last_os_error().raw_os_error(), Some(libc::EPERM));
            let child = libc::fork();
            if child == 0 {
                libc::_exit(120);
            }
            if child > 0 {
                libc::waitpid(child, std::ptr::null_mut(), 0);
            }
            assert_eq!(child, -1, "process creation was allowed");
            assert_eq!(io::Error::last_os_error().raw_os_error(), Some(libc::EPERM));
            let path = std::ffi::CString::new("/bin/false")?;
            let argv = [path.as_ptr(), std::ptr::null()];
            assert_eq!(
                libc::execve(path.as_ptr(), argv.as_ptr(), std::ptr::null()),
                -1
            );
            assert_eq!(io::Error::last_os_error().raw_os_error(), Some(libc::EPERM));
            let path = std::ffi::CString::new(forbidden.as_os_str().as_encoded_bytes())?;
            assert_eq!(libc::chmod(path.as_ptr(), 0o777), -1);
            assert_eq!(io::Error::last_os_error().raw_os_error(), Some(libc::EPERM));
        }
        let mut input_copy = input_file.try_clone()?;
        let bytes = std::thread::spawn(move || {
            let mut bytes = Vec::new();
            input_copy.read_to_end(&mut bytes)?;
            input_copy.seek(SeekFrom::Start(0))?;
            Ok::<_, io::Error>(bytes)
        })
        .join()
        .map_err(|_| "thread failed")??;
        assert!(!bytes.is_empty());
        eprintln!("PASS isolation: allowed input/thread, denied inherited descriptor/unrelated read/write/network/exec/process/metadata/supervisor changes");
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    fn verify_restrictions(
        _: &mut std::fs::File,
        _: &Path,
        _: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Err("verification requires Linux confinement".into())
    }
}
