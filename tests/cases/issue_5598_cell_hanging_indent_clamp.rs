//! Issue #5598: 좁은 표 칸에서 내어쓰기가 줄 상자를 다 먹으면 이어지는 줄의 글자가 칸 밖으로
//! 밀려 나간다.
//!
//! 재현 문서(코퍼스 00539, `자치수형자 선정 절차`)의 `분류처우위원회 심의ㆍ의결` 칸은 안쪽 폭이
//! 107.7px 인데 문단 내어쓰기가 −104.4px 다. 종전에는 둘째 줄 상자가 `x=193.3 w=3.3` 으로 무너져
//! `의결` 이 칸 오른쪽(206.1) 밖에 그려졌다. 한글은 같은 문단의 두 줄을 모두 칸 안쪽 폭으로
//! 조판한다(저장 LINE_SEG 두 줄 모두 `cs=200 sw=8076`).
//!
//! 첫 줄은 내어쓰기의 기준선이므로 그대로 두고, 이어지는 줄에서 한 글자도 못 담게 되는 경우에만
//! 내어쓰기를 접는다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::Paragraph;
use rhwp::model::style::ParaShape;
use rhwp::model::table::{Cell, Table};
use rhwp::serializer::hwpx::serialize_hwpx;

/// 칸 안쪽 폭(HWPUNIT). 96dpi 에서 약 107.7px.
const CELL_WIDTH: u32 = 9499;
/// 칸 폭에 육박하는 내어쓰기.
const HANGING_INDENT: i32 = -7832;

fn document_with_narrow_cell(indent: i32) -> Document {
    let mut shape = ParaShape {
        margin_left: 200,
        margin_right: 200,
        indent,
        ..Default::default()
    };
    shape.line_spacing = 160;

    let mut doc = Document::default();
    doc.doc_info.para_shapes = vec![shape];

    let mut cell_para = Paragraph::default();
    cell_para.text = "분류처우위원회 심의ㆍ의결".to_string();
    cell_para.para_shape_id = 0;

    let cell = Cell {
        width: CELL_WIDTH,
        height: 6317,
        paragraphs: vec![cell_para],
        ..Default::default()
    };

    let table = Table {
        row_count: 1,
        col_count: 1,
        cells: vec![cell],
        ..Default::default()
    };

    let mut host = Paragraph::default();
    host.controls.push(Control::Table(Box::new(table)));

    let mut section = Section::default();
    section.paragraphs.push(host);
    doc.sections.push(section);
    doc
}

/// 칸 안 TextRun 들이 칸 오른쪽 경계를 넘는 최대 폭(px). 넘지 않으면 0 이하.
fn worst_overflow(doc: &Document) -> f64 {
    // 합성 IR → HWPX 바이트 → DocumentCore 로 실제 렌더 경로를 태운다.
    let bytes = serialize_hwpx(doc).expect("HWPX 직렬화 실패");
    let core = DocumentCore::from_bytes(&bytes).expect("재로드 실패");
    let tree = core
        .build_page_render_tree(0)
        .expect("render tree 생성 실패");
    let json: serde_json::Value =
        serde_json::from_str(&tree.root.to_json()).expect("render tree JSON 파싱");
    let mut worst = f64::MIN;

    fn walk(node: &serde_json::Value, cell: Option<(f64, f64)>, worst: &mut f64) {
        if let Some(obj) = node.as_object() {
            let ty = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let bbox = obj
                .get("bbox")
                .and_then(|b| Some((b.get("x")?.as_f64()?, b.get("w")?.as_f64()?)));
            let cell = if ty == "Cell" { bbox.or(cell) } else { cell };
            if ty == "TextRun" {
                if let (Some((cx, cw)), Some((rx, rw))) = (cell, bbox) {
                    let text = obj.get("text").and_then(|v| v.as_str()).unwrap_or("");
                    if !text.trim().is_empty() {
                        *worst = worst.max((rx + rw) - (cx + cw));
                    }
                }
            }
            for (_, v) in obj {
                walk(v, cell, worst);
            }
        } else if let Some(arr) = node.as_array() {
            for v in arr {
                walk(v, cell, worst);
            }
        }
    }

    walk(&json, None, &mut worst);
    if worst == f64::MIN {
        panic!("칸 안 TextRun 을 찾지 못했다");
    }
    worst
}

#[test]
fn issue_5598_hanging_indent_does_not_push_text_out_of_cell() {
    let over = worst_overflow(&document_with_narrow_cell(HANGING_INDENT));
    assert!(
        over <= 2.0,
        "칸 폭에 육박하는 내어쓰기 때문에 글자가 칸 밖으로 {over:.1}px 나갔다 (#5598)"
    );
}

#[test]
fn issue_5598_no_indent_case_is_unchanged() {
    let over = worst_overflow(&document_with_narrow_cell(0));
    assert!(
        over <= 2.0,
        "내어쓰기가 없는 문단인데 글자가 칸 밖으로 {over:.1}px 나갔다"
    );
}
