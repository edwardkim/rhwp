//! HWPX ZIP 컨테이너 쓰기
//!
//! `parser::hwpx::reader`의 역방향. ZIP 내부 파일을 특정 순서와 압축 옵션으로 조립한다.
//!
//! 규칙:
//! - `mimetype`은 ZIP 최초 엔트리, STORED(무압축), extra field 없음 (OPC 규격)
//! - 그 외 파일은 DEFLATED
//! - mtime은 1980-01-01 00:00로 고정(결정적 출력)
//! - "version made by"의 호스트 OS 바이트도 고정(결정적 출력, #5969)

use std::io::{Cursor, Seek, Write};

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, System, ZipWriter};

use super::SerializeError;

/// HWPX ZIP 쓰기 래퍼
pub struct HwpxZipWriter<W: Write + Seek = Cursor<Vec<u8>>> {
    inner: ZipWriter<W>,
}

impl HwpxZipWriter {
    /// 새 인메모리 ZIP 라이터 생성
    pub fn new() -> Self {
        HwpxZipWriter {
            inner: ZipWriter::new(Cursor::new(Vec::new())),
        }
    }

    /// ZIP을 마감하고 바이트를 반환한다.
    pub fn finish(self) -> Result<Vec<u8>, SerializeError> {
        Ok(self.finish_writer()?.into_inner())
    }
}

impl<W: Write + Seek> HwpxZipWriter<W> {
    pub(crate) fn with_writer(writer: W) -> Self {
        Self {
            inner: ZipWriter::new(writer),
        }
    }
    pub(crate) fn finish_writer(self) -> Result<W, SerializeError> {
        self.inner
            .finish()
            .map_err(|error| SerializeError::ZipError(error.to_string()))
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
    /// 이미 deflate 한 데이터로 DEFLATED 엔트리를 추가한다(다시 압축하지 않는다).
    ///
    /// `raw_deflate` 는 `uncompressed_size` 바이트(CRC-32 `crc32`)를 raw deflate 한 바이트다.
    /// zip 크레이트는 압축된 데이터를 다른 ZIP 엔트리에서만 옮길 수 있으므로, 이 엔트리
    /// 하나만 든 임시 ZIP 을 [`Self::write_deflated`] 와 같은 결정론 메타데이터(고정
    /// mtime·UNIX 호스트·기본 권한)로 만들어 그 엔트리를 그대로 옮긴다. 그래서 같은
    /// 압축 바이트라면 `write_deflated` 와 같은 엔트리가 된다.
    pub fn write_precompressed_deflated(
        &mut self,
        name: &str,
        crc32: u32,
        uncompressed_size: u64,
        raw_deflate: &[u8],
    ) -> Result<(), SerializeError> {
        let staged = single_deflated_entry_zip(crc32, uncompressed_size, raw_deflate)
            .ok_or_else(|| SerializeError::ZipError(format!("{name}: 엔트리가 4GiB 를 넘는다")))?;
        let mut archive = zip::ZipArchive::new(Cursor::new(staged))
            .map_err(|e| SerializeError::ZipError(e.to_string()))?;
        let entry = archive
            .by_index_raw(0)
            .map_err(|e| SerializeError::ZipError(e.to_string()))?;
        self.inner
            .raw_copy_file_rename(entry, name)
            .map_err(|e| SerializeError::ZipError(e.to_string()))
    }
}

/// [`HwpxZipWriter::write_deflated`] 가 남기는 엔트리 메타데이터로 엔트리 하나짜리 ZIP 을 만든다.
///
/// 엔트리 이름은 옮길 때 바뀌므로 임시 이름을 쓴다. 4GiB 이상은 ZIP64 가 필요해 다루지 않는다.
fn single_deflated_entry_zip(
    crc32: u32,
    uncompressed_size: u64,
    raw_deflate: &[u8],
) -> Option<Vec<u8>> {
    const NAME: &[u8] = b"e";
    const VERSION: u16 = 20;
    const DEFLATED: u16 = 8;
    // 1980-01-01 00:00:00 (ZIP epoch) — `HwpxZipWriter::fixed_mtime`.
    const DOS_TIME: u16 = 0;
    const DOS_DATE: u16 = (1 << 5) | 1;
    // 기본 파일 권한(0o100644)을 UNIX 외부 속성 상위 16비트에 둔다.
    const EXTERNAL_ATTRIBUTES: u32 = 0o100644 << 16;
    let host_system = HwpxZipWriter::<Cursor<Vec<u8>>>::FIXED_HOST_SYSTEM as u16;
    let size = u32::try_from(uncompressed_size)
        .ok()
        .filter(|v| *v < u32::MAX)?;
    let compressed = u32::try_from(raw_deflate.len())
        .ok()
        .filter(|v| *v < u32::MAX)?;

    let mut zip = Vec::with_capacity(raw_deflate.len() + 128);
    let local_header_offset = 0u32;
    zip.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
    for value in [VERSION, 0, DEFLATED, DOS_TIME, DOS_DATE] {
        zip.extend_from_slice(&value.to_le_bytes());
    }
    for value in [crc32, compressed, size] {
        zip.extend_from_slice(&value.to_le_bytes());
    }
    zip.extend_from_slice(&(NAME.len() as u16).to_le_bytes());
    zip.extend_from_slice(&0u16.to_le_bytes());
    zip.extend_from_slice(NAME);
    zip.extend_from_slice(raw_deflate);

    let central_offset = u32::try_from(zip.len()).ok()?;
    zip.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
    for value in [
        (host_system << 8) | VERSION,
        VERSION,
        0,
        DEFLATED,
        DOS_TIME,
        DOS_DATE,
    ] {
        zip.extend_from_slice(&value.to_le_bytes());
    }
    for value in [crc32, compressed, size] {
        zip.extend_from_slice(&value.to_le_bytes());
    }
    for value in [NAME.len() as u16, 0, 0, 0, 0] {
        zip.extend_from_slice(&value.to_le_bytes());
    }
    zip.extend_from_slice(&EXTERNAL_ATTRIBUTES.to_le_bytes());
    zip.extend_from_slice(&local_header_offset.to_le_bytes());
    zip.extend_from_slice(NAME);
    let central_size = u32::try_from(zip.len()).ok()? - central_offset;

    zip.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
    for value in [0u16, 0, 1, 1] {
        zip.extend_from_slice(&value.to_le_bytes());
    }
    zip.extend_from_slice(&central_size.to_le_bytes());
    zip.extend_from_slice(&central_offset.to_le_bytes());
    zip.extend_from_slice(&0u16.to_le_bytes());
    Some(zip)
}

impl Default for HwpxZipWriter {
    fn default() -> Self {
        Self::new()
    }
}
