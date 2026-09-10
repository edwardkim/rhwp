//! #6956: 텍스트와 표를 감싼 형광펜 3쌍을 HWPX/HWP5 왕복에서 보존한다.
//! 한컴 2022 변환 대조: PARA_RANGE_TAG 종류 2, COLORREF BGR, 끝 위치 exclusive.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::document::Document;
use std::io::{Cursor, Read};

fn sample() -> Document {
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/samples/issue6956/3024739-exposure-algorithm-markpen.hwpx"
    ))
    .expect("공개 재현물");
    rhwp::parser::parse_document(&bytes).expect("HWPX 원본")
}

fn section_xml(bytes: &[u8]) -> String {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).expect("HWPX ZIP");
    let mut xml = String::new();
    archive
        .by_name("Contents/section0.xml")
        .expect("section0")
        .read_to_string(&mut xml)
        .unwrap();
    xml
}

fn assert_marks(doc: &Document, color: &str) {
    let bytes = rhwp::serializer::hwpx::serialize_hwpx(doc).expect("HWPX 저장");
    let xml = section_xml(&bytes);
    assert_eq!(
        xml.matches("<hp:markpenBegin").count(),
        3,
        "표를 감싼 1쌍도 보존"
    );
    assert_eq!(xml.matches("<hp:markpenEnd").count(), 3);
    assert_eq!(
        xml.matches(&format!("<hp:markpenBegin color=\"{color}\"/>"))
            .count(),
        3
    );
    let reloaded = rhwp::parser::parse_document(&bytes).expect("HWPX 재적재");
    let table_para = reloaded
        .sections
        .iter()
        .flat_map(|s| &s.paragraphs)
        .find(|p| p.text.is_empty() && p.markpen_marks.len() == 2)
        .expect("표를 감싼 문단");
    assert_eq!(table_para.markpen_marks[0].utf16_pos, Some(0));
    assert_eq!(table_para.markpen_marks[1].utf16_pos, Some(8));
    assert!(table_para.markpen_marks[0].color.is_some());
    assert!(table_para.markpen_marks[1].color.is_none());
}

#[test]
fn issue_6956_text_and_table_markpen_survive_two_hwpx_roundtrips() {
    let doc = sample();
    assert_marks(&doc, "#FFFFFF");
    let bytes = rhwp::serializer::hwpx::serialize_hwpx(&doc).unwrap();
    let again = rhwp::parser::parse_document(&bytes).unwrap();
    assert_marks(&again, "#FFFFFF");
}

#[test]
fn issue_6956_markpen_does_not_consume_the_text_axis() {
    for para in sample().sections.iter().flat_map(|s| &s.paragraphs) {
        assert!(
            !para.text.contains('\u{0007}'),
            "sentinel이 텍스트에 새면 안 된다"
        );
        for mark in &para.markpen_marks {
            assert!(mark.char_idx <= para.text.chars().count());
            assert!(mark.utf16_pos.unwrap() < para.char_count);
        }
    }
}

#[test]
fn issue_6956_hwp5_colorref_and_table_span_survive_roundtrip() {
    let mut doc = sample();
    for para in doc.sections.iter_mut().flat_map(|s| &mut s.paragraphs) {
        for mark in &mut para.markpen_marks {
            if mark.color.is_some() {
                mark.color = Some("#123456".into());
            }
        }
    }
    let hwp = rhwp::serializer::serialize_document(&doc).expect("HWP5 저장");
    let reloaded = rhwp::parser::parse_document(&hwp).expect("HWP5 재적재");
    let ranges: Vec<_> = reloaded
        .sections
        .iter()
        .flat_map(|s| &s.paragraphs)
        .flat_map(|p| &p.range_tags)
        .filter(|r| r.tag >> 24 == 2)
        .collect();
    assert_eq!(ranges.len(), 3);
    assert!(
        ranges.iter().all(|r| r.tag == 0x0256_3412),
        "한컴 실측 COLORREF"
    );
    assert!(
        ranges.iter().any(|r| r.start == 0 && r.end == 8),
        "표 슬롯 전체"
    );
    assert_marks(&reloaded, "#123456");
}
