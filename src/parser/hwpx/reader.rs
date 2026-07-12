//! HWPX ZIP 컨테이너 읽기
//!
//! HWPX 파일은 ZIP 아카이브이다. 내부 파일을 읽는 래퍼를 제공한다.
//!
//! ## 압축 해제 폭탄 방어
//!
//! ZIP은 높은 압축률을 허용하므로, 수 KB짜리 HWPX가 수 GB로 팽창하는
//! "zip bomb"을 만들 수 있다. 단일 `.xml` 엔트리가 무제한으로 `read_to_end`
//! 되면 호스트 프로세스를 OOM으로 몰 수 있다.
//!
//! [`MAX_XML_SIZE`] / [`MAX_BINDATA_SIZE`] 상한을 적용해 이를 차단한다.
//! 실제 한국 법령/보도자료 HWPX는 충분히 이 한도 아래에 있다.

use std::io::{self, Cursor, Read};
use zip::ZipArchive;

use super::HwpxError;

/// XML 엔트리(section, header, content.hpf 등) 엔트리당 압축 해제 상한.
///
/// [#1917 XML 축] 종전 32MB 는 실문서를 거부했다 — 정책연구 최종보고서
/// (KYRBS, 1790387-202300133)의 Contents/section1.xml 이 **75.2MB**
/// (압축 2.2MB, 압축비 35:1)로 실재하며 한글은 정상 열람한다. 정상 XML 도
/// 압축비가 수십 배에 달해 압축비 기반 가드는 오탐 — 절대 상한을 256MB 로
/// 상향한다 (관측 최대 ×3 여유). zip-bomb 방어(무제한 read_to_end 차단)
/// 목적은 유지된다.
pub const MAX_XML_SIZE: usize = 256 * 1024 * 1024; // 256 MB

/// BinData(이미지·폰트 등) 엔트리당 압축 해제 상한.
///
/// [#1917] 종전 64MB 는 실문서를 거부했다 — 정부 보도자료 계열에 비압축
/// BMP/TIF 대형 이미지가 실재한다 (10k 서베이: 최대 103.7MB BMP, 한글은
/// 정상 열람). 로드 거부는 그림 소실 + 재직렬화에서 pic 컨트롤 드롭(왕복
/// 데이터 손실)으로 이어지므로 512MB 로 상향한다. zip-bomb 방어(무제한
/// read_to_end 차단)라는 목적은 유지된다.
pub const MAX_BINDATA_SIZE: usize = 512 * 1024 * 1024; // 512 MB

/// `reader`에서 최대 `max` 바이트까지 읽는다. 초과 시 `InvalidData` 에러.
///
/// `Read::take(max + 1)`을 사용해 오버플로를 감지하되, 버퍼는 실제 읽은
/// 크기 + 1 이상으로 자라지 않는다.
fn read_limited<R: Read>(reader: &mut R, max: usize) -> io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    let cap = (max as u64).saturating_add(1);
    reader.take(cap).read_to_end(&mut buf)?;
    if buf.len() > max {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "HWPX entry exceeds {} byte limit (possible decompression bomb)",
                max
            ),
        ));
    }
    Ok(buf)
}

/// HWPX ZIP 컨테이너 리더
pub struct HwpxReader {
    archive: ZipArchive<Cursor<Vec<u8>>>,
}

impl HwpxReader {
    /// ZIP 아카이브를 연다.
    pub fn open(data: &[u8]) -> Result<Self, HwpxError> {
        let cursor = Cursor::new(data.to_vec());
        let archive = ZipArchive::new(cursor)?;
        Ok(HwpxReader { archive })
    }

    /// 지정한 경로의 파일을 UTF-8 문자열로 읽는다.
    ///
    /// 엔트리 압축 해제 크기는 [`MAX_XML_SIZE`]로 제한된다.
    ///
    /// [Issue #1932] UTF-8 이 부분 손상된 실문서(통계청 보도자료 계열 —
    /// header.xml 선두부 invalid byte)를 한글은 정상 열람하므로, 엄격 변환
    /// 실패 시 관용(lossy) 디코딩으로 폴백한다 (손상 바이트는 U+FFFD 치환,
    /// 경고 로그). 문서 전체를 버리는 종전 동작은 한글 대비 과잉 거부였다.
    pub fn read_file(&mut self, path: &str) -> Result<String, HwpxError> {
        let mut file = self
            .archive
            .by_name(path)
            .map_err(|e| HwpxError::MissingFile(format!("{}: {}", path, e)))?;
        let bytes = read_limited(&mut file, MAX_XML_SIZE)
            .map_err(|e| HwpxError::ZipError(format!("{} 읽기 실패: {}", path, e)))?;
        match String::from_utf8(bytes) {
            Ok(s) => Ok(s),
            Err(e) => {
                eprintln!(
                    "경고: {} UTF-8 손상({}) — 관용(lossy) 디코딩 적용 (U+FFFD 치환)",
                    path,
                    e.utf8_error()
                );
                Ok(String::from_utf8_lossy(e.as_bytes()).into_owned())
            }
        }
    }

    /// 지정한 경로의 파일을 바이트 배열로 읽는다.
    ///
    /// 엔트리 압축 해제 크기는 [`MAX_BINDATA_SIZE`]로 제한된다.
    pub fn read_file_bytes(&mut self, path: &str) -> Result<Vec<u8>, HwpxError> {
        let mut file = self
            .archive
            .by_name(path)
            .map_err(|e| HwpxError::MissingFile(format!("{}: {}", path, e)))?;
        read_limited(&mut file, MAX_BINDATA_SIZE)
            .map_err(|e| HwpxError::ZipError(format!("{} 읽기 실패: {}", path, e)))
    }

    /// 아카이브 내 파일 목록을 반환한다.
    pub fn file_names(&self) -> Vec<String> {
        self.archive.file_names().map(|s| s.to_string()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_invalid_zip() {
        let result = HwpxReader::open(&[0u8; 100]);
        assert!(result.is_err());
    }

    /// [#1946] ODF 암호화 manifest 감지 + 비암호화 manifest 무시.
    #[test]
    fn test_detect_odf_encryption() {
        use crate::parser::hwpx::{detect_odf_encryption, parse_hwpx, HwpxError};
        use std::io::Write;
        use zip::write::SimpleFileOptions;
        use zip::ZipWriter;

        let enc = br#"<odf:manifest><odf:file-entry full-path="Contents/header.xml"><odf:encryption-data><odf:algorithm algorithm-name="http://www.w3.org/2001/04/xmlenc#aes256-cbc"/><odf:key-derivation key-derivation-name="...#pbkdf2"/></odf:encryption-data></odf:file-entry></odf:manifest>"#;
        let detail = detect_odf_encryption(enc).expect("암호화 감지");
        assert!(detail.contains("AES-256-CBC"), "{detail}");
        assert!(detail.contains("PBKDF2"), "{detail}");

        let plain =
            br#"<odf:manifest><odf:file-entry full-path="Contents/header.xml"/></odf:manifest>"#;
        assert!(detect_odf_encryption(plain).is_none());

        // parse_hwpx 진입 감지: 암호화 manifest + 암호문 header.xml → Encrypted 에러.
        let mut out = Cursor::new(Vec::<u8>::new());
        {
            let mut zip = ZipWriter::new(&mut out);
            let opts =
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
            zip.start_file("mimetype", opts).unwrap();
            zip.write_all(b"application/hwp+zip").unwrap();
            zip.start_file("META-INF/manifest.xml", opts).unwrap();
            zip.write_all(enc).unwrap();
            zip.start_file("Contents/header.xml", opts).unwrap();
            zip.write_all(&[0x93u8, 0xFF, 0x00, 0x11]).unwrap(); // 암호문(비 UTF-8) 모사
            zip.finish().unwrap();
        }
        let bytes = out.into_inner();
        match parse_hwpx(&bytes) {
            Err(e @ HwpxError::Encrypted(_)) => {
                assert!(e.is_encrypted());
                assert!(e.to_string().contains("암호화된 문서"), "{e}");
            }
            other => panic!("expected Encrypted, got {other:?}"),
        }
    }

    #[test]
    fn test_read_limited_under_cap() {
        let data = vec![0u8; 1000];
        let mut cursor = Cursor::new(data.clone());
        let result = read_limited(&mut cursor, 2000).unwrap();
        assert_eq!(result.len(), 1000);
    }

    #[test]
    fn test_read_limited_at_cap() {
        let data = vec![0u8; 1000];
        let mut cursor = Cursor::new(data.clone());
        let result = read_limited(&mut cursor, 1000).unwrap();
        assert_eq!(result.len(), 1000);
    }

    #[test]
    fn test_read_limited_over_cap() {
        let data = vec![0u8; 1001];
        let mut cursor = Cursor::new(data);
        let result = read_limited(&mut cursor, 1000);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    /// [#1917 XML 축] 실문서급 대형 XML(40MB — 종전 32MB 한도 초과, 새 256MB
    /// 한도 이내)은 수용되어야 한다 (KYRBS section1.xml 75.2MB 실측 대응).
    #[test]
    fn test_large_legit_xml_entry_accepted() {
        use std::io::Write;
        use zip::write::SimpleFileOptions;
        use zip::ZipWriter;

        let mut out = Cursor::new(Vec::<u8>::new());
        {
            let mut zip = ZipWriter::new(&mut out);
            let opts =
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
            zip.start_file("Contents/section1.xml", opts).unwrap();
            let payload = vec![b'A'; 40 * 1024 * 1024]; // 40MB — 종전 한도(32MB) 초과
            zip.write_all(&payload).unwrap();
            zip.finish().unwrap();
        }
        let bytes = out.into_inner();
        let mut reader = HwpxReader::open(&bytes).unwrap();
        let result = reader.read_file("Contents/section1.xml");
        assert!(
            result.is_ok(),
            "40MB XML entry should be accepted: {:?}",
            result.err()
        );
        assert_eq!(result.unwrap().len(), 40 * 1024 * 1024);
    }

    /// [#1932] UTF-8 부분 손상 엔트리는 lossy 폴백으로 수용되어야 한다
    /// (한글 정합 — 통계청 보도자료 header.xml invalid byte 실측 대응).
    #[test]
    fn test_invalid_utf8_entry_lossy_accepted() {
        use std::io::Write;
        use zip::write::SimpleFileOptions;
        use zip::ZipWriter;

        let mut out = Cursor::new(Vec::<u8>::new());
        {
            let mut zip = ZipWriter::new(&mut out);
            let opts =
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
            zip.start_file("Contents/header.xml", opts).unwrap();
            // 유효 XML 사이에 invalid UTF-8 바이트(0x93) 삽입 — 실측 케이스 모사
            zip.write_all(b"<hwpml>\x93</hwpml>").unwrap();
            zip.finish().unwrap();
        }
        let bytes = out.into_inner();
        let mut reader = HwpxReader::open(&bytes).unwrap();
        let s = reader
            .read_file("Contents/header.xml")
            .expect("#1932: 손상 UTF-8 은 lossy 폴백으로 수용되어야 함");
        assert!(s.starts_with("<hwpml>"));
        assert!(
            s.contains('\u{FFFD}'),
            "손상 바이트는 U+FFFD 로 치환: {s:?}"
        );
        assert!(s.ends_with("</hwpml>"));
    }

    /// 해제 시 상한을 넘는 엔트리가 포함된 ZIP은 `ZipError`로 거부되어야 한다.
    ///
    /// 실제 "zip bomb"을 흉내내기 위해 고압축 가능한(반복 패턴) 데이터
    /// `MAX_XML_SIZE + 1` 바이트를 deflate로 압축한 뒤 `.xml` 엔트리로
    /// 넣는다. 압축 결과물은 수십 KB지만, 압축 해제 시도는 상한에
    /// 걸려 실패해야 한다.
    #[test]
    fn test_zip_bomb_xml_entry_rejected() {
        use std::io::Write;
        use zip::write::SimpleFileOptions;
        use zip::ZipWriter;

        let mut out = Cursor::new(Vec::<u8>::new());
        {
            let mut zip = ZipWriter::new(&mut out);
            let opts =
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
            zip.start_file("Contents/bomb.xml", opts).unwrap();
            // 상한 + 1 바이트짜리 반복 패턴 — 매우 높은 압축률
            let payload = vec![b'A'; MAX_XML_SIZE + 1];
            zip.write_all(&payload).unwrap();
            zip.finish().unwrap();
        }
        let bytes = out.into_inner();
        // 압축본은 실제로 수십 KB에 불과
        assert!(
            bytes.len() < 1024 * 1024,
            "bomb compressed too large: {}",
            bytes.len()
        );

        let mut reader = HwpxReader::open(&bytes).unwrap();
        let result = reader.read_file("Contents/bomb.xml");
        assert!(result.is_err(), "bomb entry should be rejected");
        match result.unwrap_err() {
            HwpxError::ZipError(msg) => {
                assert!(
                    msg.contains("decompression bomb") || msg.contains("limit"),
                    "unexpected error message: {}",
                    msg
                );
            }
            other => panic!("expected ZipError, got {:?}", other),
        }
    }
}
