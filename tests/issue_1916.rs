//! Issue #1916 — HWP5 저장 시 표(tbl) CommonObjAttr 소실 (flowWithText 축).
//!
//! `serialize_table` 은 `raw_ctrl_data`(HWP5 파스 원본 바이트)가 없으면 CTRL_HEADER
//! 데이터를 **빈 값**으로 방출했고, 재파스 시 표의 CommonObjAttr 전체
//! (treat_as_char/wrap/flowWithText 등)가 기본값으로 붕괴했다. 10k 서베이의
//! 12건은 .hwp 명명 HWPX(역방향 확장자 위장)가 어댑터 미경유 plain serialize 로
//! 들어간 케이스(게이트 분류는 #1914 의 FORMAT_SKIP 으로 별도 해소)지만,
//! **raw_ctrl_data 없는 IR**(편집기 신설 표 등)의 plain 저장이 같은 붕괴를 겪는
//! 실경로다. 수정: 부재 시 IR `common` 으로 합성 (다른 GSO 컨트롤과 동일).

use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::model::shape::TextWrap;
use rhwp::model::style::CharShape;
use rhwp::model::table::{Cell, Table};
use rhwp::parser::parse_document;
use rhwp::serializer::serialize_document;

fn make_doc_with_flow_table() -> Document {
    let mut table = Table {
        row_count: 1,
        col_count: 1,
        cells: vec![Cell {
            col: 0,
            row: 0,
            col_span: 1,
            row_span: 1,
            width: 8000,
            height: 2000,
            paragraphs: vec![Paragraph::default()],
            ..Default::default()
        }],
        ..Default::default()
    };
    // raw_ctrl_data 는 비워 둔다 — plain serialize 의 합성 경로를 검증.
    assert!(table.raw_ctrl_data.is_empty());
    table.common.treat_as_char = true;
    table.common.flow_with_text = true;
    table.common.text_wrap = TextWrap::TopAndBottom;
    table.common.width = 8000;
    table.common.height = 2000;

    let host = Paragraph {
        text: String::new(),
        char_count: 1,
        line_segs: vec![LineSeg {
            line_height: 1000,
            line_spacing: 600,
            ..Default::default()
        }],
        controls: vec![Control::Table(Box::new(table))],
        ..Default::default()
    };

    let mut doc = Document::default();
    doc.doc_info.char_shapes.push(CharShape::default());
    let mut section = Section::default();
    section.paragraphs.push(host);
    doc.sections.push(section);
    doc
}

fn find_table(doc: &Document) -> &Table {
    for para in &doc.sections[0].paragraphs {
        for ctrl in &para.controls {
            if let Control::Table(t) = ctrl {
                return t;
            }
        }
    }
    panic!("표 컨트롤 미발견");
}

/// raw_ctrl_data 없는 표의 plain HWP5 저장 왕복에서 CommonObjAttr 이 보존된다.
#[test]
fn issue_1916_table_common_attr_survives_plain_hwp5_roundtrip() {
    let doc = make_doc_with_flow_table();
    let bytes = serialize_document(&doc).expect("serialize");
    let doc2 = parse_document(&bytes).expect("reparse");
    let t = find_table(&doc2);

    assert!(
        t.common.flow_with_text,
        "flowWithText true → false 소실 (#1916 — CTRL_HEADER 빈 데이터 방출 회귀)"
    );
    assert!(
        t.common.treat_as_char,
        "treat_as_char 소실 (CommonObjAttr 전체 붕괴 회귀)"
    );
    assert_eq!(
        t.common.text_wrap,
        TextWrap::TopAndBottom,
        "text_wrap 소실 (CommonObjAttr 전체 붕괴 회귀)"
    );
    assert_eq!(t.common.width, 8000, "표 폭 소실");
    assert_eq!(t.common.height, 2000, "표 높이 소실");
}
