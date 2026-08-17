//! [일곱 번째 계약] 표의 캡션 유무가 개체 공통속성 attr **bit 29**(캡션 존재
//! 플래그)에 반영돼야 한다.
//!
//! 어긋나면 한글 2022 가 CTRL_HEADER 직후의 캡션 LIST_HEADER 유무를 오판해 문서
//! 전체 개방을 거부한다(크롤 빈티지 40429 표 COM 이등분 실측, 양방향 반증).
//! 종전 HWP3/HWPX 출처 표는 캡션을 방출하면서 이 비트를 안 켜 개방 거부됐다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::document::Section;
use rhwp::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
use rhwp::model::shape::{Caption, CaptionDirection, CaptionVertAlign};
use rhwp::model::table::{Cell, Table};
use rhwp::serializer::body_text::serialize_section;

const CAPTION_BIT: u32 = 1 << 29;

/// 캡션 1개짜리 픽스처.
fn caption_fixture(text: &str) -> Caption {
    Caption {
        direction: CaptionDirection::Top,
        vert_align: CaptionVertAlign::Center,
        width: 4321,
        spacing: 850,
        max_width: 30000,
        include_margin: true,
        paragraphs: vec![Paragraph {
            char_count: (text.chars().count() + 1) as u32,
            text: text.to_string(),
            char_offsets: (0..text.chars().count() as u32).collect(),
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
            line_segs: vec![LineSeg {
                text_start: 0,
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
}

fn one_cell() -> Cell {
    Cell {
        col: 0,
        row: 0,
        col_span: 1,
        row_span: 1,
        width: 10000,
        height: 5000,
        border_fill_id: 1,
        paragraphs: vec![Paragraph {
            char_count: 1,
            char_offsets: vec![],
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
            line_segs: vec![LineSeg {
                text_start: 0,
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn make_table(caption: Option<Caption>) -> Table {
    Table {
        row_count: 1,
        col_count: 1,
        row_sizes: vec![1],
        border_fill_id: 1,
        cells: vec![one_cell()],
        caption,
        ..Default::default()
    }
}

/// 직렬화된 섹션 레코드에서 `'tbl '` CTRL_HEADER 의 attr(payload 4..8)을 읽는다.
fn tbl_ctrl_attr(bytes: &[u8]) -> u32 {
    let mut off = 0usize;
    while off + 4 <= bytes.len() {
        let h = u32::from_le_bytes(bytes[off..off + 4].try_into().unwrap());
        let tag = h & 0x3ff;
        let mut sz = ((h >> 20) & 0xfff) as usize;
        let mut body = off + 4;
        if sz == 0xfff {
            sz = u32::from_le_bytes(bytes[body..body + 4].try_into().unwrap()) as usize;
            body += 4;
        }
        // CTRL_HEADER(tag 71) 의 ctrl_id 는 payload 0..4 (LE 로 'tbl ' = 0x74626c20)
        if tag == 71 && bytes[body..body + 4] == *b" lbt" {
            return u32::from_le_bytes(bytes[body + 4..body + 8].try_into().unwrap());
        }
        off = body + sz;
    }
    panic!("'tbl ' CTRL_HEADER 를 찾지 못함");
}

fn serialize_table(caption: Option<Caption>) -> Vec<u8> {
    serialize_section(&Section {
        paragraphs: vec![Paragraph {
            char_count: 2,
            controls: vec![Control::Table(Box::new(make_table(caption)))],
            ..Default::default()
        }],
        raw_stream: None,
        ..Default::default()
    })
}

#[test]
fn captioned_table_sets_common_attr_bit29() {
    let with = serialize_table(Some(caption_fixture("표 캡션")));
    assert_ne!(
        tbl_ctrl_attr(&with) & CAPTION_BIT,
        0,
        "캡션 있는 표는 attr bit29 가 켜져야 한다"
    );
}

#[test]
fn uncaptioned_table_leaves_common_attr_bit29_clear() {
    let without = serialize_table(None);
    assert_eq!(
        tbl_ctrl_attr(&without) & CAPTION_BIT,
        0,
        "캡션 없는 표는 attr bit29 가 꺼져야 한다"
    );
}
