//! Process-local integrity primitives shared by CLI writers.
//!
//! Edit commands and agent plans use one SHA-256 and path-lock contract so
//! compare-and-swap preconditions cannot drift between command surfaces.

use std::fs;
use std::path::Path;

pub(crate) fn sha256_hex_of(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let out = Sha256::digest(bytes);
    let mut hex = String::with_capacity(out.len() * 2);
    for b in out {
        use std::fmt::Write as _;
        let _ = write!(hex, "{b:02x}");
    }
    hex
}

/// 같은 입력 경로를 다루는 rhwp writer 사이의 read-check-write 경계를 직렬화한다.
/// 잠금 파일은 rename 뒤에도 같은 inode/handle을 유지해야 하므로 원본 파일이 아니라
/// 정규화한 경로의 해시로 만든 안정적인 temp sidecar를 사용한다.
pub(crate) struct CasPathLock {
    _file: fs::File,
}

impl CasPathLock {
    pub(crate) fn acquire(source: &Path) -> std::io::Result<Self> {
        #[cfg(unix)]
        use std::os::unix::fs::OpenOptionsExt;

        let canonical = fs::canonicalize(source)?;
        let key = sha256_hex_of(canonical.to_string_lossy().as_bytes());
        let lock_path = std::env::temp_dir().join(format!("rhwp-cas-v1-{key}.lock"));
        let mut options = fs::OpenOptions::new();
        options.read(true).write(true).create(true);
        #[cfg(unix)]
        options.mode(0o600);
        let file = options.open(lock_path)?;
        file.lock()?;
        Ok(Self { _file: file })
    }
}

/// debug 통합 회귀에서 두 별도 프로세스를 잠금 시도 직전까지 모은다. release
/// binary에는 환경변수 기반 파일 쓰기·대기 경로 자체를 컴파일하지 않는다.
#[cfg(debug_assertions)]
pub(crate) fn cas_test_synchronize_before_lock() -> Result<(), String> {
    let Some(directory) = std::env::var_os("RHWP_INTERNAL_TEST_CAS_BARRIER") else {
        return Ok(());
    };
    let directory = std::path::PathBuf::from(directory);
    fs::write(
        directory.join(format!("arrived-{}", std::process::id())),
        b"",
    )
    .map_err(|e| e.to_string())?;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let arrived = fs::read_dir(&directory)
            .map_err(|e| e.to_string())?
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().starts_with("arrived-"))
            .count();
        if arrived >= 2 {
            return Ok(());
        }
        if std::time::Instant::now() >= deadline {
            return Err("CAS test barrier 에 두 프로세스가 도착하지 않았습니다".to_string());
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

#[cfg(not(debug_assertions))]
pub(crate) fn cas_test_synchronize_before_lock() -> Result<(), String> {
    Ok(())
}

/// 최초 해시 검사를 통과한 프로세스를 표시한다. 잠금이 사라진 mutation에서는 두
/// marker가 생기고, 정상 구현에서는 첫 writer만 이 경계에 도달한다.
#[cfg(debug_assertions)]
pub(crate) fn cas_test_mark_checked_and_wait() {
    let Some(directory) = std::env::var_os("RHWP_INTERNAL_TEST_CAS_BARRIER") else {
        return;
    };
    let directory = std::path::PathBuf::from(directory);
    let _ = fs::write(
        directory.join(format!("checked-{}", std::process::id())),
        b"",
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    while std::time::Instant::now() < deadline {
        let checked = fs::read_dir(&directory)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().starts_with("checked-"))
            .count();
        if checked >= 2 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

#[cfg(not(debug_assertions))]
pub(crate) fn cas_test_mark_checked_and_wait() {}
