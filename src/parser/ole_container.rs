//! OLE 컨테이너 내부 CFB 파싱 (Task #195 단계 7)
//!
//! BinData/BIN000N.OLE 스트림의 압축 해제 후 바이트는 표준 CFB(Compound File Binary) 컨테이너이다.
//! 이 모듈은 그 내부 스트림(`\x02OlePres000`, `OOXMLChartContents`, `Contents`)을 추출한다.

use cfb::CompoundFile;
use std::io::{Cursor, Read};

/// OLE 컨테이너에서 추출한 네이티브 이미지 종류
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeImageKind {
    Bmp,
    Png,
    Jpeg,
    Gif,
}

impl NativeImageKind {
    pub fn mime(&self) -> &'static str {
        match self {
            Self::Bmp => "image/bmp",
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Gif => "image/gif",
        }
    }
}

/// OLE 컨테이너 내용
#[derive(Debug, Clone, Default)]
pub struct OleContainer {
    /// `\x02OlePres000` 스트림에서 추출한 EMF 바이트 (OLE Presentation Stream 헤더 스킵됨)
    pub preview_emf: Option<Vec<u8>>,
    /// `OOXMLChartContents` 원본 바이트 (OOXML 차트 XML)
    pub ooxml_chart: Option<Vec<u8>>,
    /// `Contents` 원본 바이트 (내부 OLE 데이터)
    pub raw_contents: Option<Vec<u8>>,
    /// `\x01Ole10Native` 스트림에서 추출한 네이티브 임베딩 바이트 (BMP/PNG/JPEG 등)
    pub native_image: Option<(NativeImageKind, Vec<u8>)>,
}

impl OleContainer {
    /// OOXML 차트 XML을 포함하는지 여부
    pub fn has_ooxml_chart(&self) -> bool {
        self.ooxml_chart.as_ref().is_some_and(|b| !b.is_empty())
    }

    /// EMF 프리뷰를 포함하는지 여부
    pub fn has_preview(&self) -> bool {
        self.preview_emf.as_ref().is_some_and(|b| !b.is_empty())
    }
}

/// 해제된 BinData 바이트(CFB 컨테이너)에서 주요 스트림 추출
///
/// 입력: CFB 매직(`D0CF11E0...`)로 시작하는 바이트 슬라이스
/// 반환: 내부 스트림이 하나라도 존재하면 `Some(container)`, CFB 파싱 실패 시 `None`
pub fn parse_ole_container(cfb_bytes: &[u8]) -> Option<OleContainer> {
    if cfb_bytes.len() < 8 {
        return None;
    }
    let cursor = Cursor::new(cfb_bytes);
    let mut comp = CompoundFile::open(cursor).ok()?;

    let mut container = OleContainer::default();

    // 최상위 스트림 목록 수집
    let entries: Vec<String> = comp
        .walk()
        .filter(|e| e.is_stream())
        .map(|e| e.path().to_string_lossy().to_string())
        .collect();

    for path in entries {
        let name = path.trim_start_matches('/');
        if name == "\u{0002}OlePres000" || name.ends_with("OlePres000") {
            if let Ok(mut s) = comp.open_stream(&path) {
                let mut buf = Vec::new();
                if s.read_to_end(&mut buf).is_ok() {
                    container.preview_emf = strip_ole_presentation_header(&buf);
                }
            }
        } else if name == "OOXMLChartContents" {
            if let Ok(mut s) = comp.open_stream(&path) {
                let mut buf = Vec::new();
                if s.read_to_end(&mut buf).is_ok() && !buf.is_empty() {
                    container.ooxml_chart = Some(buf);
                }
            }
        } else if name == "Contents" {
            if let Ok(mut s) = comp.open_stream(&path) {
                let mut buf = Vec::new();
                if s.read_to_end(&mut buf).is_ok() && !buf.is_empty() {
                    container.raw_contents = Some(buf);
                }
            }
        } else if name == "\u{0001}Ole10Native" || name.ends_with("Ole10Native") {
            if let Ok(mut s) = comp.open_stream(&path) {
                let mut buf = Vec::new();
                if s.read_to_end(&mut buf).is_ok() && buf.len() > 4 {
                    // Ole10Native: [u32 LE length][payload] — payload가 BMP/PNG/JPEG 등 네이티브 바이트
                    let inner = &buf[4..];
                    if let Some(img) = detect_native_image(inner) {
                        container.native_image = Some(img);
                    }
                }
            }
        }
    }

    // preview_emf가 없으면 OlePres000에서 DIB 추출 시도 → BMP로 포장
    if container.preview_emf.is_none() && container.native_image.is_none() {
        // OlePres000을 다시 읽어 DIB 헤더를 찾아본다
        // (이미 preview_emf가 None인 경우만)
        if let Ok(entries) = std::panic::catch_unwind(|| {
            let cursor = Cursor::new(cfb_bytes);
            CompoundFile::open(cursor).ok().map(|mut comp| {
                comp.walk()
                    .filter(|e| e.is_stream())
                    .map(|e| e.path().to_string_lossy().to_string())
                    .collect::<Vec<_>>()
            })
        }) {
            if let Some(Some(paths)) = Some(entries) {
                let cursor = Cursor::new(cfb_bytes);
                if let Ok(mut comp2) = CompoundFile::open(cursor) {
                    for path in &paths {
                        let name = path.trim_start_matches('/');
                        if name == "\u{0002}OlePres000" || name.ends_with("OlePres000") {
                            if let Ok(mut s) = comp2.open_stream(path) {
                                let mut buf = Vec::new();
                                if s.read_to_end(&mut buf).is_ok() {
                                    if let Some(bmp) = extract_dib_as_bmp(&buf) {
                                        container.native_image = Some((NativeImageKind::Bmp, bmp));
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if container.preview_emf.is_some()
        || container.ooxml_chart.is_some()
        || container.raw_contents.is_some()
        || container.native_image.is_some()
    {
        Some(container)
    } else {
        None
    }
}

/// 바이트 슬라이스의 선두 매직으로 이미지 포맷을 판별
pub fn detect_native_image(data: &[u8]) -> Option<(NativeImageKind, Vec<u8>)> {
    if data.len() < 4 {
        return None;
    }
    if data.starts_with(b"BM") {
        return Some((NativeImageKind::Bmp, data.to_vec()));
    }
    if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        return Some((NativeImageKind::Png, data.to_vec()));
    }
    if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some((NativeImageKind::Jpeg, data.to_vec()));
    }
    if data.starts_with(b"GIF8") {
        return Some((NativeImageKind::Gif, data.to_vec()));
    }
    None
}

/// OlePres000 스트림에서 DIB(Device Independent Bitmap) 데이터를 찾아 BMP 파일 바이트로 재포장한다.
///
/// DIB 시그니처: BITMAPINFOHEADER는 `biSize=40` (0x28 0x00 0x00 0x00)로 시작.
/// 앞에 BMP FILEHEADER(14바이트, "BM"+파일크기+예약+픽셀오프셋)를 합성하여 표준 BMP로 만든다.
fn extract_dib_as_bmp(data: &[u8]) -> Option<Vec<u8>> {
    let scan_limit = data.len().min(4096);
    for i in 0..scan_limit.saturating_sub(40) {
        // BITMAPINFOHEADER.biSize == 40
        let bi_size = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);
        if bi_size != 40 {
            continue;
        }
        // 유효성: width/height가 현실적인 범위
        let w = i32::from_le_bytes([data[i + 4], data[i + 5], data[i + 6], data[i + 7]]);
        let h = i32::from_le_bytes([data[i + 8], data[i + 9], data[i + 10], data[i + 11]]);
        if w <= 0 || w > 100_000 || h.abs() == 0 || h.abs() > 100_000 {
            continue;
        }
        let bit_count = u16::from_le_bytes([data[i + 14], data[i + 15]]);
        if !matches!(bit_count, 1 | 4 | 8 | 16 | 24 | 32) {
            continue;
        }
        let compression =
            u32::from_le_bytes([data[i + 16], data[i + 17], data[i + 18], data[i + 19]]);
        // 색상 테이블 크기 계산
        let clr_used = u32::from_le_bytes([data[i + 32], data[i + 33], data[i + 34], data[i + 35]]);
        let palette_entries = if bit_count <= 8 {
            if clr_used > 0 && clr_used <= 256 {
                clr_used
            } else {
                1u32 << bit_count
            }
        } else {
            0
        };
        let palette_bytes = palette_entries * 4;
        let dib_and_data = &data[i..];
        let offset_to_pixels = 14 + 40 + palette_bytes as usize;
        // 파일 전체 크기 = 14 헤더 + DIB 나머지
        let file_size = 14 + dib_and_data.len() as u32;
        let mut bmp = Vec::with_capacity(file_size as usize);
        bmp.extend_from_slice(b"BM");
        bmp.extend_from_slice(&file_size.to_le_bytes());
        bmp.extend_from_slice(&[0u8; 4]); // reserved
        bmp.extend_from_slice(&(offset_to_pixels as u32).to_le_bytes());
        bmp.extend_from_slice(dib_and_data);
        let _ = compression;
        return Some(bmp);
    }
    None
}

/// OLE Presentation Stream 헤더를 스킵하고 내부 EMF/메타파일 바이트를 반환한다.
///
/// OLE Presentation Stream 대략 구조 (MS-OLEDS):
/// `ULONG ansiClipboardFormat, ULONG tgtDevSize, tgtDev(variable), ULONG aspect,
///  ULONG lindex, ULONG advf, ULONG reserved, DWORD width, DWORD height, DWORD size, bytes[size]`
///
/// 여기서는 EMR_HEADER 매직(record_type=0x00000001 + " EMF" @ offset +40)을
/// 찾아서 그 위치부터 바이트를 반환한다. 매직을 찾지 못하면 `None`.
fn strip_ole_presentation_header(data: &[u8]) -> Option<Vec<u8>> {
    // EMF record header: u32 type=1, u32 size, 16 bytes bounds, 16 bytes frame, u32 signature=" EMF"(0x464D4520)
    // signature(" EMF")는 EMR_HEADER의 offset 40부터
    if data.len() < 64 {
        return None;
    }
    // 스캔 범위 제한 (OLE 헤더가 보통 수십~수백 바이트)
    let scan_limit = data.len().min(4096);
    for i in 0..(scan_limit.saturating_sub(44)) {
        let type_ok = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]) == 1;
        if !type_ok {
            continue;
        }
        // " EMF" = 0x20 0x45 0x4D 0x46
        let sig = &data[i + 40..i + 44];
        if sig == b" EMF" {
            return Some(data[i..].to_vec());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_no_emf_magic() {
        let data = vec![0u8; 100];
        assert!(strip_ole_presentation_header(&data).is_none());
    }

    #[test]
    fn test_strip_emf_at_offset() {
        // 헤더 20바이트 + EMR_HEADER(44바이트: type=1, size, 32바이트 bounds/frame, " EMF")
        let mut data = vec![0u8; 20];
        // type = 1
        data.extend_from_slice(&1u32.to_le_bytes());
        // size = 100
        data.extend_from_slice(&100u32.to_le_bytes());
        // bounds(16) + frame(16) = 32 bytes zero
        data.extend_from_slice(&[0u8; 32]);
        // " EMF"
        data.extend_from_slice(b" EMF");
        // 더미 남은 바이트
        data.extend_from_slice(&[0xAA; 20]);

        let stripped = strip_ole_presentation_header(&data).expect("EMF should be found");
        assert_eq!(&stripped[..4], &1u32.to_le_bytes()); // record type
        assert_eq!(&stripped[40..44], b" EMF");
    }

    #[test]
    fn test_parse_empty_bytes() {
        assert!(parse_ole_container(&[]).is_none());
        assert!(parse_ole_container(&[0u8; 4]).is_none());
    }

    #[test]
    fn test_parse_non_cfb() {
        // CFB 매직이 아닌 임의 바이트
        let bytes: Vec<u8> = (0..128u8).collect();
        assert!(parse_ole_container(&bytes).is_none());
    }
}
