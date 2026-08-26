//! [#4860] 공개 HWP5 BodyText 경계의 UTF-16 벌크 처리 계약.

use rhwp::parser::body_text::parse_body_text_section;
use rhwp::parser::tags::{HWPTAG_PARA_HEADER, HWPTAG_PARA_TEXT};

fn record(tag_id: u16, level: u16, data: &[u8]) -> Vec<u8> {
    let header = (tag_id as u32) | ((level as u32) << 10) | ((data.len() as u32) << 20);
    let mut bytes = header.to_le_bytes().to_vec();
    bytes.extend_from_slice(data);
    bytes
}

fn paragraph_text(units: &[u16]) -> String {
    let mut header = Vec::new();
    header.extend_from_slice(&(units.len() as u32).to_le_bytes());
    header.extend_from_slice(&0u32.to_le_bytes());
    header.extend_from_slice(&0u16.to_le_bytes());
    header.extend_from_slice(&[0, 0]);

    let mut text = Vec::new();
    for unit in units {
        text.extend_from_slice(&unit.to_le_bytes());
    }
    let mut stream = record(HWPTAG_PARA_HEADER, 0, &header);
    stream.extend(record(HWPTAG_PARA_TEXT, 1, &text));
    let section = parse_body_text_section(&stream).expect("BodyText 파싱");
    assert_eq!(section.paragraphs.len(), 1);
    section.paragraphs[0].text.clone()
}

#[test]
fn bulk_text_path_preserves_tab_and_utf16_surrogate_pair() {
    let units = [
        0x0041, 0x0009, 0, 0, 0, 0, 0, 0, 0, 0x0042, 0xD83D, 0xDE00, 0x000D,
    ];
    assert_eq!(paragraph_text(&units), "A\tB😀");
}

#[test]
fn bulk_text_path_skips_unpaired_surrogate_without_losing_neighbors() {
    let units = [0x0041, 0xD83D, 0x0042, 0x000D];
    assert_eq!(paragraph_text(&units), "AB");
}

#[test]
fn bulk_text_path_stops_at_paragraph_end() {
    let units = [0xAC00, 0x000D, 0xB098];
    assert_eq!(paragraph_text(&units), "가");
}
