use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub(crate) fn write_atomically(output: &Path, bytes: &[u8]) -> io::Result<()> {
    write_atomically_with(output, bytes, |from, to| fs::rename(from, to))
}

fn write_atomically_with<F>(output: &Path, bytes: &[u8], replace: F) -> io::Result<()>
where
    F: FnOnce(&Path, &Path) -> io::Result<()>,
{
    let (temp_path, mut temp) = create_sibling_temp(output)?;
    if let Err(error) = write_and_sync(&mut temp, bytes) {
        drop(temp);
        cleanup_temp(&temp_path);
        return Err(error);
    }
    drop(temp);
    if let Err(error) = replace(&temp_path, output) {
        cleanup_temp(&temp_path);
        return Err(error);
    }
    Ok(())
}

fn create_sibling_temp(output: &Path) -> io::Result<(PathBuf, File)> {
    let parent = output.parent().filter(|path| !path.as_os_str().is_empty());
    let parent = parent.unwrap_or_else(|| Path::new("."));
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    for attempt in 0..128 {
        let candidate = parent.join(format!(
            ".rhwp-hml-{}.{}.{}.tmp",
            std::process::id(),
            nonce,
            attempt
        ));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(file) => return Ok((candidate, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "임시 출력 파일 이름을 확보할 수 없습니다",
    ))
}

fn write_and_sync(file: &mut File, bytes: &[u8]) -> io::Result<()> {
    file.write_all(bytes)?;
    file.sync_all()
}

fn cleanup_temp(path: &Path) {
    let _ = fs::remove_file(path);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_temp_dir() -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir().join(format!("rhwp_atomic_file_{}_{nonce}", std::process::id()))
    }

    #[test]
    fn pre_replace_failure_preserves_destination_and_removes_temp_file() {
        let directory = unique_temp_dir();
        std::fs::create_dir_all(&directory).expect("create temp directory");
        let destination = directory.join("output.hml");
        std::fs::write(&destination, b"old bytes").expect("seed destination");

        let error = write_atomically_with(&destination, b"new bytes", |_temp, _destination| {
            Err(std::io::Error::other("injected replacement failure"))
        })
        .expect_err("replacement should fail");

        assert_eq!(error.kind(), std::io::ErrorKind::Other);
        assert_eq!(
            std::fs::read(&destination).expect("read preserved destination"),
            b"old bytes"
        );
        assert_eq!(
            std::fs::read_dir(&directory)
                .expect("read temp directory")
                .filter_map(Result::ok)
                .count(),
            1,
            "only the original destination should remain"
        );
    }
}
