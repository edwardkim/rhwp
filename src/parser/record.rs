//! HWP 레코드 파싱
//!
//! HWP 레코드 헤더 구조 (4바이트):
//! - bits 0~9:   태그 ID (0~1023)
//! - bits 10~19: 레벨 (0~1023)
//! - bits 20~31: 크기 (0~4095)
//! - 크기 == 4095이면 다음 4바이트가 실제 크기 (확장)

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};

use super::tags;

/// HWP 레코드
#[derive(Debug, Clone)]
pub struct Record {
    /// 태그 ID
    pub tag_id: u16,
    /// 레벨 (트리 깊이)
    pub level: u16,
    /// 데이터 크기
    pub size: u32,
    /// 레코드 데이터
    pub data: Vec<u8>,
}

impl Record {
    /// 태그 이름 반환
    pub fn tag_name(&self) -> &'static str {
        tags::tag_name(self.tag_id)
    }

    /// 바이트 스트림에서 모든 레코드를 파싱
    pub fn read_all(data: &[u8]) -> Result<Vec<Record>, RecordError> {
        let mut cursor = Cursor::new(data);
        let mut records = Vec::new();

        while (cursor.position() as usize) < data.len() {
            let remaining = data.len() - cursor.position() as usize;
            if remaining < 4 {
                break;
            }

            let header = cursor
                .read_u32::<LittleEndian>()
                .map_err(|e| RecordError::IoError(e.to_string()))?;

            let tag_id = (header & 0x3FF) as u16;
            let level = ((header >> 10) & 0x3FF) as u16;
            let mut size = (header >> 20) as u32;

            // 확장 크기: 크기 필드가 4095(0xFFF)이면 다음 4바이트가 실제 크기
            if size == 0xFFF {
                size = cursor
                    .read_u32::<LittleEndian>()
                    .map_err(|e| RecordError::IoError(e.to_string()))?;
            }

            // 데이터 읽기
            let pos = cursor.position() as usize;
            if pos + size as usize > data.len() {
                return Err(RecordError::UnexpectedEof {
                    tag_id,
                    expected: size as usize,
                    available: data.len() - pos,
                });
            }

            let mut record_data = vec![0u8; size as usize];
            cursor
                .read_exact(&mut record_data)
                .map_err(|e| RecordError::IoError(e.to_string()))?;

            records.push(Record {
                tag_id,
                level,
                size,
                data: record_data,
            });
        }

        Ok(records)
    }
}

impl std::fmt::Display for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Record(tag={}/{}, level={}, size={})",
            self.tag_id,
            self.tag_name(),
            self.level,
            self.size
        )
    }
}

/// 레코드 파싱 에러
#[derive(Debug)]
pub enum RecordError {
    IoError(String),
    UnexpectedEof {
        tag_id: u16,
        expected: usize,
        available: usize,
    },
}

impl std::fmt::Display for RecordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecordError::IoError(e) => write!(f, "레코드 IO 오류: {}", e),
            RecordError::UnexpectedEof {
                tag_id,
                expected,
                available,
            } => write!(
                f,
                "레코드 데이터 부족: tag={}, 필요={}, 가용={}",
                tag_id, expected, available
            ),
        }
    }
}

impl std::error::Error for RecordError {}

#[cfg(test)]
mod tests {
    use super::*;

    /// 기본 레코드 생성 헬퍼
    fn make_record_bytes(tag_id: u16, level: u16, data: &[u8]) -> Vec<u8> {
        let size = data.len() as u32;
        let header = (tag_id as u32) | ((level as u32) << 10) | (size << 20);
        let mut bytes = header.to_le_bytes().to_vec();
        bytes.extend_from_slice(data);
        bytes
    }

    /// 확장 크기 레코드 생성 헬퍼
    fn make_extended_record_bytes(tag_id: u16, level: u16, data: &[u8]) -> Vec<u8> {
        let size = data.len() as u32;
        let header = (tag_id as u32) | ((level as u32) << 10) | (0xFFF << 20);
        let mut bytes = header.to_le_bytes().to_vec();
        bytes.extend_from_slice(&size.to_le_bytes());
        bytes.extend_from_slice(data);
        bytes
    }

    #[test]
    fn test_read_single_record() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let bytes = make_record_bytes(tags::HWPTAG_PARA_HEADER, 0, &data);

        let records = Record::read_all(&bytes).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].tag_id, tags::HWPTAG_PARA_HEADER);
        assert_eq!(records[0].level, 0);
        assert_eq!(records[0].size, 4);
        assert_eq!(records[0].data, data);
    }

    #[test]
    fn test_read_multiple_records() {
        let mut bytes = make_record_bytes(tags::HWPTAG_PARA_HEADER, 0, &[0x01, 0x02]);
        bytes.extend(make_record_bytes(
            tags::HWPTAG_PARA_TEXT,
            1,
            &[0x03, 0x04, 0x05],
        ));

        let records = Record::read_all(&bytes).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].tag_id, tags::HWPTAG_PARA_HEADER);
        assert_eq!(records[0].level, 0);
        assert_eq!(records[1].tag_id, tags::HWPTAG_PARA_TEXT);
        assert_eq!(records[1].level, 1);
        assert_eq!(records[1].data, [0x03, 0x04, 0x05]);
    }

    #[test]
    fn test_extended_size_record() {
        let data = vec![0xAA; 5000];
        let bytes = make_extended_record_bytes(tags::HWPTAG_PARA_TEXT, 0, &data);

        let records = Record::read_all(&bytes).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].size, 5000);
        assert_eq!(records[0].data.len(), 5000);
    }

    #[test]
    fn test_record_display() {
        let bytes = make_record_bytes(tags::HWPTAG_PARA_HEADER, 2, &[0x01]);
        let records = Record::read_all(&bytes).unwrap();
        let display = format!("{}", records[0]);
        assert!(display.contains("PARA_HEADER"));
        assert!(display.contains("level=2"));
    }

    #[test]
    fn test_empty_data() {
        let records = Record::read_all(&[]).unwrap();
        assert!(records.is_empty());
    }

    #[test]
    fn test_zero_size_record() {
        let bytes = make_record_bytes(tags::HWPTAG_DOCUMENT_PROPERTIES, 0, &[]);
        let records = Record::read_all(&bytes).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].size, 0);
        assert!(records[0].data.is_empty());
    }

    #[test]
    fn test_truncated_data_error() {
        // 헤더는 크기 100을 가리키지만 데이터는 2바이트만
        let header = (tags::HWPTAG_PARA_TEXT as u32) | (100 << 20);
        let mut bytes = header.to_le_bytes().to_vec();
        bytes.extend_from_slice(&[0x01, 0x02]);

        let result = Record::read_all(&bytes);
        assert!(result.is_err());
    }
}
