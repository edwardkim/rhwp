//! 고정형 저장 계약: 무늬 없음 -1과 원본 무늬 값의 바이트 보존 (#8733).
//! 주석 갱신: 2026-11-07 UTC. 조판 좌표 검사가 아닌 HWP/HWPX 저장 속성 검사다.
//! 독립 근거: issuecomment-6702261909의 한컴 2020 출력은 원본 HWP의 pattern=0을
//! 가로 무늬로 인쇄하며, 같은 원본의 무늬 없음은 -1이다. PNG 일치율은 이 검사 대상이 아니다.
//! 양수 무늬 번호의 기존 계약과 바이너리 writer는 변경하지 않는다.

use std::io::{Cursor, Read};

use rhwp::model::control::Control;
use rhwp::model::style::SolidFill;
use rhwp::wasm_api::HwpDocument;

fn table_doc() -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    doc.create_table_native(0, 0, 0, 1, 1).unwrap();
    doc
}

fn table_pos(doc: &HwpDocument) -> (usize, usize) {
    for (p, para) in doc.document().sections[0].paragraphs.iter().enumerate() {
        for (c, control) in para.controls.iter().enumerate() {
            if matches!(control, Control::Table(_)) {
                return (p, c);
            }
        }
    }
    panic!("table missing");
}

fn set_cell(doc: &mut HwpDocument, json: &str) {
    let (p, c) = table_pos(doc);
    doc.set_cell_properties_native(0, p, c, 0, json).unwrap();
}

fn cell_fill(doc: &HwpDocument) -> (u16, SolidFill) {
    let (p, c) = table_pos(doc);
    let Control::Table(table) = &doc.document().sections[0].paragraphs[p].controls[c] else {
        unreachable!();
    };
    let id = table.cells[0].border_fill_id;
    (
        id,
        doc.document().doc_info.border_fills[usize::from(id - 1)]
            .fill
            .solid
            .unwrap(),
    )
}

// HWP record header(tag/level/size)를 직접 읽는다. 재파서와 같은 sentinel 오류를 공유하지 않는다.
fn saved_pattern(bytes: &[u8], id: u16) -> i32 {
    let mut cfb = cfb::CompoundFile::open(Cursor::new(bytes)).unwrap();
    let mut file_header = Vec::new();
    cfb.open_stream("/FileHeader")
        .unwrap()
        .read_to_end(&mut file_header)
        .unwrap();
    let mut stream = Vec::new();
    cfb.open_stream("/DocInfo")
        .unwrap()
        .read_to_end(&mut stream)
        .unwrap();
    if file_header[36] & 1 != 0 {
        let mut inflated = Vec::new();
        flate2::read::DeflateDecoder::new(stream.as_slice())
            .read_to_end(&mut inflated)
            .unwrap();
        stream = inflated;
    }
    let mut offset = 0;
    let mut border_fill_id = 0;
    while offset + 4 <= stream.len() {
        let header = u32::from_le_bytes(stream[offset..offset + 4].try_into().unwrap());
        offset += 4;
        let mut size = (header >> 20) as usize;
        if size == 0xfff {
            size = u32::from_le_bytes(stream[offset..offset + 4].try_into().unwrap()) as usize;
            offset += 4;
        }
        if header & 0x3ff == 20 {
            border_fill_id += 1;
            if border_fill_id == id {
                assert!(size >= 48, "solid BorderFill payload is too short");
                assert_eq!(
                    u32::from_le_bytes(stream[offset + 32..offset + 36].try_into().unwrap()),
                    1,
                    "solid fill flag"
                );
                // attr2 + 5 borders*6 + fill-kind4 + background4 + hatch-color4.
                return i32::from_le_bytes(stream[offset + 44..offset + 48].try_into().unwrap());
            }
        }
        offset += size;
    }
    panic!("BorderFill {id} missing");
}

#[test]
fn newly_authored_solid_fill_defaults_to_no_pattern_in_hwp_and_hwpx() {
    for pattern in [None, Some(-1)] {
        let mut doc = table_doc();
        let mut props = serde_json::json!({"fillType":"solid", "fillColor":"#aabbcc"});
        if let Some(pattern) = pattern {
            props["patternType"] = pattern.into();
        }
        set_cell(&mut doc, &props.to_string());
        let (id, solid) = cell_fill(&doc);
        assert_eq!(solid.pattern_type, -1);
        let hwp = doc.export_hwp_native().unwrap();
        assert_eq!(saved_pattern(&hwp, id), -1, "raw HWP sentinel");
        for bytes in [hwp, doc.export_hwpx_native().unwrap()] {
            let reopened = HwpDocument::from_bytes(&bytes).unwrap();
            let (_, fill) = cell_fill(&reopened);
            assert_eq!(fill.pattern_type, -1);
            assert_eq!(fill.background_color, solid.background_color);
        }
    }
}

#[test]
fn explicit_hwp_patterns_survive_save_and_sparse_color_edit() {
    for pattern in [0, 1, 3, 6] {
        let mut doc = table_doc();
        set_cell(&mut doc, &serde_json::json!({"fillType":"solid", "fillColor":"#aabbcc", "patternColor":"#112233", "patternType":pattern}).to_string());
        let bytes = doc.export_hwp_native().unwrap();
        assert_eq!(saved_pattern(&bytes, cell_fill(&doc).0), pattern);
        let mut reopened = HwpDocument::from_bytes(&bytes).unwrap();
        let old_fill = cell_fill(&reopened).1;
        let hatch_color = old_fill.pattern_color;
        set_cell(
            &mut reopened,
            r##"{"fillType":"solid","fillColor":"#ddeeff"}"##,
        );
        let (id, fill) = cell_fill(&reopened);
        assert_eq!(
            fill.pattern_type, pattern,
            "sparse color edit preserves existing pattern"
        );
        assert_eq!(fill.pattern_color, hatch_color);
        assert_ne!(fill.background_color, old_fill.background_color);
        assert_eq!(fill.background_color, 0x00ff_eedd);
        assert_eq!(
            saved_pattern(&reopened.export_hwp_native().unwrap(), id),
            pattern
        );
        set_cell(&mut reopened, r##"{"fillType":"solid","patternType":-1}"##);
        let (id, cleared) = cell_fill(&reopened);
        assert_eq!(
            cleared.background_color, fill.background_color,
            "pattern-only edit keeps the face color"
        );
        assert_eq!(
            saved_pattern(&reopened.export_hwp_native().unwrap(), id),
            -1
        );
    }
}

#[test]
fn table_background_without_pattern_uses_no_pattern() {
    let mut doc = table_doc();
    let (p, c) = table_pos(&doc);
    doc.set_table_properties_native(0, p, c, r##"{"fillType":"solid","fillColor":"#aabbcc"}"##)
        .unwrap();
    let Control::Table(table) = &doc.document().sections[0].paragraphs[p].controls[c] else {
        unreachable!()
    };
    let id = table.border_fill_id;
    assert_eq!(saved_pattern(&doc.export_hwp_native().unwrap(), id), -1);
}
