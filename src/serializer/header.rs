//! FileHeader 직렬화
//!
//! `parse_file_header()`의 역방향으로, FileHeader를 256바이트 바이너리로 변환한다.
//!
//! FileHeader 구조 (256바이트, 비압축):
//! - [0-17]: "HWP Document File\0" 시그니처
//! - [18-31]: 0 패딩
//! - [32-35]: 버전 (revision, build, minor, major) LE
//! - [36-39]: 속성 플래그 u32 LE
//! - [40-43]: 라이선스 속성
//! - [44-47]: EncryptVersion u32 LE
//! - [48-255]: 0 패딩

use crate::model::document::FileHeader;
use crate::parser::header::{FILE_HEADER_SIZE, HWP_SIGNATURE};

/// FileHeader를 256바이트 바이너리로 직렬화
pub fn serialize_file_header(header: &FileHeader) -> Vec<u8> {
    // 원본 데이터가 있으면 그대로 반환 (완벽한 라운드트립)
    if let Some(ref raw) = header.raw_data {
        return raw.clone();
    }

    let mut data = vec![0u8; FILE_HEADER_SIZE];

    // [0-17]: 시그니처 "HWP Document File"
    data[..HWP_SIGNATURE.len()].copy_from_slice(HWP_SIGNATURE);
    // [18] 이후는 이미 0으로 패딩됨

    // [32-35]: 버전 (revision, build, minor, major)
    data[32] = header.version.revision;
    data[33] = header.version.build;
    data[34] = header.version.minor;
    data[35] = header.version.major;

    // [36-39]: 플래그
    data[36..40].copy_from_slice(&header.flags.to_le_bytes());

    data
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::document::{FileHeader, HwpVersion};
    use crate::parser::header::parse_file_header;

    #[test]
    fn test_serialize_file_header_size() {
        let header = FileHeader::default();
        let bytes = serialize_file_header(&header);
        assert_eq!(bytes.len(), 256);
    }

    #[test]
    fn test_serialize_file_header_signature() {
        let header = FileHeader::default();
        let bytes = serialize_file_header(&header);
        assert_eq!(&bytes[..18], b"HWP Document File\0");
    }

    #[test]
    fn test_serialize_file_header_roundtrip() {
        let header = FileHeader {
            version: HwpVersion {
                major: 5,
                minor: 0,
                build: 6,
                revision: 1,
            },
            flags: 0x01, // compressed
            compressed: true,
            encrypted: false,
            distribution: false,
            raw_data: None,
        };

        let bytes = serialize_file_header(&header);
        let parsed = parse_file_header(&bytes).unwrap();

        assert_eq!(parsed.version.major, 5);
        assert_eq!(parsed.version.minor, 0);
        assert_eq!(parsed.version.build, 6);
        assert_eq!(parsed.version.revision, 1);
        assert!(parsed.flags.compressed);
        assert!(!parsed.flags.encrypted);
        assert!(!parsed.flags.distribution);
    }

    #[test]
    fn test_serialize_file_header_all_flags() {
        let header = FileHeader {
            version: HwpVersion {
                major: 5,
                minor: 1,
                build: 0,
                revision: 0,
            },
            flags: 0x07, // compressed + encrypted + distribution
            compressed: true,
            encrypted: true,
            distribution: true,
            raw_data: None,
        };

        let bytes = serialize_file_header(&header);
        let parsed = parse_file_header(&bytes).unwrap();

        assert!(parsed.flags.compressed);
        assert!(parsed.flags.encrypted);
        assert!(parsed.flags.distribution);
    }

    #[test]
    fn test_serialize_file_header_padding() {
        let header = FileHeader::default();
        let bytes = serialize_file_header(&header);

        // [18-31] 패딩 영역은 0
        assert!(bytes[18..32].iter().all(|&b| b == 0));
        // 새 문서는 라이선스/EncryptVersion/예약 영역을 0으로 초기화
        assert!(bytes[40..256].iter().all(|&b| b == 0));
    }
}
