//! HWPX ZIP 컨테이너 쓰기
//!
//! `parser::hwpx::reader`의 역방향. ZIP 내부 파일을 특정 순서와 압축 옵션으로 조립한다.
//!
//! 규칙:
//! - `mimetype`은 ZIP 최초 엔트리, STORED(무압축), extra field 없음 (OPC 규격)
//! - 그 외 파일은 DEFLATED
//! - mtime은 1980-01-01 00:00로 고정(결정적 출력)
//! - "version made by"의 호스트 OS 바이트도 고정(결정적 출력, #5969)

use std::io::{Cursor, Write};

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, System, ZipWriter};

use super::SerializeError;

/// HWPX ZIP 쓰기 래퍼
pub struct HwpxZipWriter {
    inner: ZipWriter<Cursor<Vec<u8>>>,
}

impl HwpxZipWriter {
    /// 새 인메모리 ZIP 라이터 생성
    pub fn new() -> Self {
        HwpxZipWriter {
            inner: ZipWriter::new(Cursor::new(Vec::new())),
        }
    }

    /// ZIP 중앙 디렉터리 레코드의 "version made by" 상위 바이트(호스트 OS 표기).
    ///
    /// [#5969] `SimpleFileOptions::default()` 는 이 값을 **빌드한 호스트**에서 가져온다
    /// (리눅스·macOS `0x03` UNIX, Windows `0x00` MS-DOS). 그래서 같은 문서를 저장해도
    /// 산출 플랫폼에 따라 중앙 디렉터리 레코드 수만큼 바이트가 갈라졌다. 엔트리 내용은
    /// 동일한데 컨테이너 해시만 달라져, 크로스 플랫폼 재생성 대조에서 회귀로 오판됐다.
    ///
    /// `UNIX` 로 고정한다 — 커밋된 판정 자산이 리눅스에서 생성돼 이 값이면 기존 해시가
    /// 그대로 유지된다. 외부 속성의 유닉스 권한 필드는 쓰지 않으므로 판독기 영향은 없다.
    const FIXED_HOST_SYSTEM: System = System::Unix;

    fn fixed_mtime() -> DateTime {
        // 1980-01-01 00:00:00 (ZIP epoch)
        DateTime::default()
    }

    /// 두 쓰기 경로가 공유하는 결정론 옵션. 압축 방식만 갈린다.
    ///
    /// 한 자리에 모아 둔다 — 새 결정론 항목이 생겼을 때 한쪽 경로만 받는 일을 막는다.
    fn deterministic_opts(method: CompressionMethod) -> SimpleFileOptions {
        SimpleFileOptions::default()
            .compression_method(method)
            .last_modified_time(Self::fixed_mtime())
            .system(Self::FIXED_HOST_SYSTEM)
    }

    /// STORED(무압축)로 엔트리를 추가한다. `mimetype`에 사용.
    pub fn write_stored(&mut self, name: &str, data: &[u8]) -> Result<(), SerializeError> {
        let opts = Self::deterministic_opts(CompressionMethod::Stored);
        self.inner
            .start_file(name, opts)
            .map_err(|e| SerializeError::ZipError(e.to_string()))?;
        self.inner
            .write_all(data)
            .map_err(|e| SerializeError::ZipError(e.to_string()))?;
        Ok(())
    }

    /// DEFLATED(압축)로 엔트리를 추가한다.
    pub fn write_deflated(&mut self, name: &str, data: &[u8]) -> Result<(), SerializeError> {
        let opts = Self::deterministic_opts(CompressionMethod::Deflated);
        self.inner
            .start_file(name, opts)
            .map_err(|e| SerializeError::ZipError(e.to_string()))?;
        self.inner
            .write_all(data)
            .map_err(|e| SerializeError::ZipError(e.to_string()))?;
        Ok(())
    }

    /// ZIP을 마감하고 바이트를 반환한다.
    pub fn finish(self) -> Result<Vec<u8>, SerializeError> {
        let cursor = self
            .inner
            .finish()
            .map_err(|e| SerializeError::ZipError(e.to_string()))?;
        Ok(cursor.into_inner())
    }
}

impl Default for HwpxZipWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ZIP 중앙 디렉터리 레코드 서명. 뒤이어 "version made by"(2바이트)가 온다.
    const CENTRAL_DIRECTORY_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x01, 0x02];
    /// 서명 시작점에서 호스트 OS 바이트까지의 거리 — spec 버전 다음 바이트다.
    const HOST_SYSTEM_OFFSET: usize = 5;
    /// UNIX. `System::Unix as u8` 과 같아야 한다.
    const EXPECTED_HOST_BYTE: u8 = 3;

    fn sample_archive() -> Vec<u8> {
        let mut w = HwpxZipWriter::new();
        w.write_stored("mimetype", b"application/hwp+zip").unwrap();
        w.write_deflated("version.xml", b"<version/>").unwrap();
        w.write_deflated("Contents/content.hpf", b"<package/>")
            .unwrap();
        w.finish().unwrap()
    }

    /// 중앙 디렉터리 레코드마다 호스트 OS 바이트의 위치를 돌려준다.
    fn host_system_byte_offsets(bytes: &[u8]) -> Vec<usize> {
        bytes
            .windows(CENTRAL_DIRECTORY_SIGNATURE.len())
            .enumerate()
            .filter(|(_, w)| *w == CENTRAL_DIRECTORY_SIGNATURE)
            .map(|(i, _)| i + HOST_SYSTEM_OFFSET)
            .collect()
    }

    #[test]
    fn central_directory_pins_host_system_byte() {
        // [#5969] 종전에는 이 바이트를 빌드 호스트에서 가져와 리눅스 0x03 / Windows 0x00 로
        // 갈렸다. 엔트리 내용이 같은데 컨테이너 해시만 달라져 회귀로 오판됐다.
        let bytes = sample_archive();
        let offsets = host_system_byte_offsets(&bytes);
        assert_eq!(
            offsets.len(),
            3,
            "엔트리 3개면 중앙 디렉터리 레코드도 3개다"
        );
        for off in offsets {
            assert_eq!(
                bytes[off], EXPECTED_HOST_BYTE,
                "중앙 디렉터리 +{HOST_SYSTEM_OFFSET} 의 호스트 OS 바이트가 UNIX(0x03) 여야 한다"
            );
        }
    }

    #[test]
    fn fixed_host_system_matches_expected_byte() {
        assert_eq!(HwpxZipWriter::FIXED_HOST_SYSTEM as u8, EXPECTED_HOST_BYTE);
    }

    #[test]
    fn same_input_produces_identical_bytes() {
        // mtime 고정과 호스트 바이트 고정이 함께 있어야 성립한다.
        assert_eq!(sample_archive(), sample_archive());
    }

    #[test]
    fn both_write_paths_share_the_deterministic_options() {
        // 한쪽 경로만 결정론 설정을 받는 일을 막는다 — 두 옵션은 압축 방식만 달라야 한다.
        let stored = HwpxZipWriter::deterministic_opts(CompressionMethod::Stored);
        let deflated = HwpxZipWriter::deterministic_opts(CompressionMethod::Deflated);
        assert_eq!(
            format!("{stored:?}").replace("Stored", "<method>"),
            format!("{deflated:?}").replace("Deflated", "<method>"),
            "압축 방식 외의 옵션이 두 경로에서 갈렸다"
        );
    }
}
