//! Issue #5702: 칸 안 어울림(SQUARE) 중첩표를 줄 흐름에 통째로 쌓아 다음 글줄을 칸 밖으로
//! 밀어낸다.
//!
//! 어울림 개체는 글이 옆으로 흐르므로 줄 흐름에서 차지하는 높이가 개체 높이가 아니라 그
//! 앵커 줄 높이다. 재현 문서(코퍼스 00317 `당직근무 일지`)의 `1.당직함` 칸은 안쪽 40.9px 에
//! 앵커 줄 10.7px + 글줄 13.3px 를 담는데, 표 높이 28.9px 를 흐름으로 더하면 글줄이 19.3px
//! 밀려 칸 아래로 7.3px 나간다.
//!
//! 계약: 저장 앵커 줄이 있는 어울림 중첩표는 그 줄 높이만큼만 흐름을 전진한다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::model::shape::TextWrap;
use rhwp::model::style::ParaShape;
use rhwp::model::table::{Cell, Table};
use rhwp::serializer::hwpx::serialize_hwpx;

/// 어울림 중첩표 크기(HWPUNIT) — 96dpi 에서 91.3 × 28.9px.
const NESTED_W: u32 = 6850;
const NESTED_H: u32 = 2164;
/// 앵커 줄 높이(HWPUNIT) — 10.7px.
const ANCHOR_LH: i32 = 800;
/// 바깥 칸 크기(HWPUNIT) — 214.3 × 48.5px.
const OUTER_CELL_W: u32 = 16072;
const OUTER_CELL_H: u32 = 3640;

fn nested_square_table() -> Table {
    let mut inner = Cell {
        width: NESTED_W,
        height: NESTED_H,
        ..Default::default()
    };
    inner.paragraphs.push(Paragraph {
        text: "O".to_string(),
        ..Default::default()
    });

    let mut table = Table {
        row_count: 1,
        col_count: 1,
        cells: vec![inner],
        ..Default::default()
    };
    table.common.width = NESTED_W;
    table.common.height = NESTED_H;
    table.common.treat_as_char = false;
    table.common.flow_with_text = true;
    table.common.text_wrap = TextWrap::Square;
    table
}

fn document_with_square_nested_table() -> Document {
    let mut doc = Document::default();
    doc.doc_info.para_shapes = vec![ParaShape::default()];

    // 어울림 표를 담은 앵커 문단 — 저장 줄 높이는 표 높이가 아니라 10.7px 다.
    let mut host = Paragraph::default();
    host.controls
        .push(Control::Table(Box::new(nested_square_table())));
    host.line_segs = vec![LineSeg {
        text_start: 0,
        vertical_pos: 0,
        line_height: ANCHOR_LH,
        text_height: ANCHOR_LH,
        baseline_distance: ANCHOR_LH * 85 / 100,
        segment_width: OUTER_CELL_W as i32,
        ..Default::default()
    }];

    let follower = Paragraph {
        text: "1.당직함".to_string(),
        ..Default::default()
    };

    let outer_cell = Cell {
        width: OUTER_CELL_W,
        height: OUTER_CELL_H,
        paragraphs: vec![host, follower],
        ..Default::default()
    };
    let mut outer = Table {
        row_count: 1,
        col_count: 1,
        cells: vec![outer_cell],
        ..Default::default()
    };
    outer.common.width = OUTER_CELL_W;
    outer.common.height = OUTER_CELL_H;

    let mut anchor = Paragraph::default();
    anchor.controls.push(Control::Table(Box::new(outer)));

    let mut section = Section::default();
    section.paragraphs.push(anchor);
    doc.sections.push(section);
    doc
}

/// (`1.당직함` run 의 아래 끝, 그 run 을 담은 칸의 아래 끝).
fn follower_bottom_vs_cell(doc: &Document) -> (f64, f64) {
    let bytes = serialize_hwpx(doc).expect("HWPX 직렬화 실패");
    let core = DocumentCore::from_bytes(&bytes).expect("재로드 실패");
    let tree = core
        .build_page_render_tree(0)
        .expect("render tree 생성 실패");
    let json: serde_json::Value =
        serde_json::from_str(&tree.root.to_json()).expect("render tree JSON 파싱");

    let mut found: Option<(f64, f64)> = None;
    fn walk(node: &serde_json::Value, cell: Option<(f64, f64)>, found: &mut Option<(f64, f64)>) {
        if let Some(obj) = node.as_object() {
            let ty = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let bbox = obj
                .get("bbox")
                .and_then(|b| Some((b.get("y")?.as_f64()?, b.get("h")?.as_f64()?)));
            // 가장 안쪽 칸(=중첩표 칸)은 폭이 좁다. 바깥 칸만 추적하려면 더 큰 상자를 쓴다.
            let cell = if ty == "Cell" {
                match (cell, bbox) {
                    (Some(prev), Some(cur)) if prev.1 >= cur.1 => Some(prev),
                    (_, Some(cur)) => Some(cur),
                    (prev, None) => prev,
                }
            } else {
                cell
            };
            if ty == "TextRun" && found.is_none() {
                let text = obj.get("text").and_then(|v| v.as_str()).unwrap_or("");
                if text.contains("당직함") {
                    if let (Some((cy, ch)), Some((ry, rh))) = (cell, bbox) {
                        *found = Some((ry + rh, cy + ch));
                    }
                }
            }
            for (_, v) in obj {
                walk(v, cell, found);
            }
        } else if let Some(arr) = node.as_array() {
            for v in arr {
                walk(v, cell, found);
            }
        }
    }
    walk(&json, None, &mut found);
    found.expect("`1.당직함` run 을 찾지 못했다")
}

#[test]
fn issue_5702_square_nested_table_does_not_push_text_out_of_cell() {
    let (run_bottom, cell_bottom) = follower_bottom_vs_cell(&document_with_square_nested_table());
    assert!(
        run_bottom <= cell_bottom + 1.0,
        "어울림 중첩표 뒤 글줄이 칸 아래로 {:.1}px 나갔다 (#5702)",
        run_bottom - cell_bottom
    );
}
