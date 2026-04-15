//! HWP 레코드 직렬화
//!
//! `Record::read_all()`의 역방향으로, 레코드를 바이너리 스트림으로 인코딩한다.
//!
//! 레코드 헤더 구조 (4바이트):
//! - bits 0~9:   태그 ID (0~1023)
//! - bits 10~19: 레벨 (0~1023)
//! - bits 20~31: 크기 (0~4094, 4095=확장)
//! - 크기 >= 4095이면 헤더에 0xFFF 기록 후 실제 크기 u32 추가

use crate::parser::record::Record;

/// 단일 레코드를 바이너리로 인코딩
///
/// 반환: 레코드 헤더 + 데이터 바이트
pub fn write_record(tag_id: u16, level: u16, data: &[u8]) -> Vec<u8> {
    let size = data.len() as u32;
    let extended = size >= 0xFFF;

    let header_size = if extended { 0xFFF } else { size };
    let header: u32 =
        (tag_id as u32 & 0x3FF) | ((level as u32 & 0x3FF) << 10) | (header_size << 20);

    let mut bytes = Vec::with_capacity(4 + if extended { 4 } else { 0 } + data.len());
    bytes.extend_from_slice(&header.to_le_bytes());

    if extended {
        bytes.extend_from_slice(&size.to_le_bytes());
    }

    bytes.extend_from_slice(data);
    bytes
}

/// Record 구조체를 바이너리로 인코딩
pub fn write_record_from(record: &Record) -> Vec<u8> {
    write_record(record.tag_id, record.level, &record.data)
}

/// 여러 레코드를 연결하여 바이너리 스트림 생성
pub fn write_records(records: &[Record]) -> Vec<u8> {
    let total_size: usize = records
        .iter()
        .map(|r| 4 + if r.data.len() >= 0xFFF { 4 } else { 0 } + r.data.len())
        .sum();

    let mut bytes = Vec::with_capacity(total_size);
    for record in records {
        bytes.extend(write_record_from(record));
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::record::Record;
    use crate::parser::tags;

    #[test]
    fn test_write_record_basic() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let bytes = write_record(tags::HWPTAG_PARA_HEADER, 0, &data);

        let records = Record::read_all(&bytes).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].tag_id, tags::HWPTAG_PARA_HEADER);
        assert_eq!(records[0].level, 0);
        assert_eq!(records[0].size, 4);
        assert_eq!(records[0].data, data);
    }

    #[test]
    fn test_write_record_with_level() {
        let data = [0xAA, 0xBB];
        let bytes = write_record(tags::HWPTAG_PARA_TEXT, 3, &data);

        let records = Record::read_all(&bytes).unwrap();
        assert_eq!(records[0].tag_id, tags::HWPTAG_PARA_TEXT);
        assert_eq!(records[0].level, 3);
        assert_eq!(records[0].data, data);
    }

    #[test]
    fn test_write_record_zero_size() {
        let bytes = write_record(tags::HWPTAG_DOCUMENT_PROPERTIES, 0, &[]);

        let records = Record::read_all(&bytes).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].size, 0);
        assert!(records[0].data.is_empty());
    }

    #[test]
    fn test_write_record_extended_size() {
        // 4095바이트 이상: 확장 크기 사용
        let data = vec![0xCC; 5000];
        let bytes = write_record(tags::HWPTAG_PARA_TEXT, 1, &data);

        let records = Record::read_all(&bytes).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].size, 5000);
        assert_eq!(records[0].data.len(), 5000);
        assert!(records[0].data.iter().all(|&b| b == 0xCC));
    }

    #[test]
    fn test_write_record_boundary_4094() {
        // 4094바이트: 일반 크기 (확장 아님)
        let data = vec![0xDD; 4094];
        let bytes = write_record(tags::HWPTAG_PARA_TEXT, 0, &data);

        // 헤더 4바이트 + 데이터 4094바이트 = 4098바이트 (확장 크기 4바이트 없음)
        assert_eq!(bytes.len(), 4 + 4094);

        let records = Record::read_all(&bytes).unwrap();
        assert_eq!(records[0].size, 4094);
    }

    #[test]
    fn test_write_record_boundary_4095() {
        // 정확히 4095바이트: 확장 크기 사용
        let data = vec![0xEE; 4095];
        let bytes = write_record(tags::HWPTAG_PARA_TEXT, 0, &data);

        // 헤더 4바이트 + 확장크기 4바이트 + 데이터 4095바이트
        assert_eq!(bytes.len(), 4 + 4 + 4095);

        let records = Record::read_all(&bytes).unwrap();
        assert_eq!(records[0].size, 4095);
    }

    #[test]
    fn test_write_multiple_records() {
        let records = vec![
            Record {
                tag_id: tags::HWPTAG_PARA_HEADER,
                level: 0,
                size: 2,
                data: vec![0x01, 0x02],
            },
            Record {
                tag_id: tags::HWPTAG_PARA_TEXT,
                level: 1,
                size: 3,
                data: vec![0x03, 0x04, 0x05],
            },
        ];

        let bytes = write_records(&records);
        let parsed = Record::read_all(&bytes).unwrap();

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].tag_id, tags::HWPTAG_PARA_HEADER);
        assert_eq!(parsed[0].level, 0);
        assert_eq!(parsed[0].data, [0x01, 0x02]);
        assert_eq!(parsed[1].tag_id, tags::HWPTAG_PARA_TEXT);
        assert_eq!(parsed[1].level, 1);
        assert_eq!(parsed[1].data, [0x03, 0x04, 0x05]);
    }

    #[test]
    fn test_write_record_from_struct() {
        let record = Record {
            tag_id: tags::HWPTAG_CHAR_SHAPE,
            level: 0,
            size: 4,
            data: vec![0x10, 0x20, 0x30, 0x40],
        };

        let bytes = write_record_from(&record);
        let parsed = Record::read_all(&bytes).unwrap();

        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].tag_id, record.tag_id);
        assert_eq!(parsed[0].level, record.level);
        assert_eq!(parsed[0].data, record.data);
    }

    #[test]
    fn test_roundtrip_header_encoding() {
        // 헤더 비트 필드가 정확하게 인코딩/디코딩되는지 검증
        // tag_id 최대: 1023, level 최대: 1023
        let bytes = write_record(1023, 1023, &[0xFF; 100]);
        let records = Record::read_all(&bytes).unwrap();
        assert_eq!(records[0].tag_id, 1023);
        assert_eq!(records[0].level, 1023);
        assert_eq!(records[0].size, 100);
    }
}
