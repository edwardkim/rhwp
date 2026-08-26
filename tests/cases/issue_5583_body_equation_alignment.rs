//! Issue #5583: 본문 흐름의 수식-only 줄이 문단 정렬을 따르지 않고 항상 왼쪽에 붙는다.
//!
//! `place_empty_line_inline_equations` 의 정렬 오프셋은 표 셀 안에서만 계산되고 본문에서는
//! `0.0` 으로 굳어 있었다. 그래서 가운데 정렬 문단에 놓인 글자처럼 취급(treatAsChar=1) 수식이
//! 단 왼쪽 끝에 그려졌다(코퍼스 00192 `국가유산수리 감리대가 기준` 2·3쪽: 본문 좌측 75.6px,
//! 가운데였다면 269.6px).
//!
//! 저장 `LINE_SEG.column_start` 가 0 이면 그 줄은 흐름 x 를 담고 있지 않으므로 문단 정렬을
//! 적용한다. `column_start > 0` 인 줄은 한컴이 흐름 x 를 적어 둔 경우라 #1256/#1308 계약대로
//! 저장값을 존중한다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::{Control, Equation};
use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::model::style::{Alignment, ParaShape};
use rhwp::serializer::hwpx::serialize_hwpx;

/// 단 폭(HWPUNIT) — 96dpi 에서 642.5px.
const COLUMN_WIDTH: i32 = 48188;
/// 수식 폭(HWPUNIT) — 96dpi 에서 254.6px.
const EQ_WIDTH: u32 = 19094;

fn document_with_centered_equation(column_start: i32) -> Document {
    let shape = ParaShape {
        alignment: Alignment::Center,
        ..Default::default()
    };

    let mut doc = Document::default();
    doc.doc_info.para_shapes = vec![shape];

    let mut eq = Equation::default();
    eq.common.width = EQ_WIDTH;
    eq.common.height = 4333;
    eq.common.treat_as_char = true;
    eq.script = "1 over 2".to_string();
    eq.font_size = 1000;

    let mut para = Paragraph::default();
    para.para_shape_id = 0;
    para.controls.push(Control::Equation(Box::new(eq)));
    para.line_segs = vec![LineSeg {
        text_start: 0,
        vertical_pos: 0,
        line_height: 4333,
        text_height: 4333,
        baseline_distance: 2773,
        line_spacing: 1820,
        column_start,
        segment_width: COLUMN_WIDTH,
        ..Default::default()
    }];

    let mut section = Section::default();
    section.paragraphs.push(para);
    doc.sections.push(section);
    doc
}

/// 첫 수식 노드의 (x, w) 와 그 본문 영역 (x, w).
fn equation_and_body(doc: &Document) -> ((f64, f64), (f64, f64)) {
    let bytes = serialize_hwpx(doc).expect("HWPX 직렬화 실패");
    let core = DocumentCore::from_bytes(&bytes).expect("재로드 실패");
    let tree = core
        .build_page_render_tree(0)
        .expect("render tree 생성 실패");
    let json: serde_json::Value =
        serde_json::from_str(&tree.root.to_json()).expect("render tree JSON 파싱");

    let mut eq: Option<(f64, f64)> = None;
    let mut body: Option<(f64, f64)> = None;

    fn walk(node: &serde_json::Value, eq: &mut Option<(f64, f64)>, body: &mut Option<(f64, f64)>) {
        if let Some(obj) = node.as_object() {
            let ty = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let bbox = obj
                .get("bbox")
                .and_then(|b| Some((b.get("x")?.as_f64()?, b.get("w")?.as_f64()?)));
            if ty == "Body" && body.is_none() {
                *body = bbox;
            }
            if ty == "Equation" && eq.is_none() {
                *eq = bbox;
            }
            for (_, v) in obj {
                walk(v, eq, body);
            }
        } else if let Some(arr) = node.as_array() {
            for v in arr {
                walk(v, eq, body);
            }
        }
    }

    walk(&json, &mut eq, &mut body);
    (
        eq.expect("수식 노드를 찾지 못했다"),
        body.expect("본문 영역을 찾지 못했다"),
    )
}

#[test]
fn issue_5583_body_equation_follows_paragraph_center() {
    let ((eq_x, eq_w), (body_x, body_w)) = equation_and_body(&document_with_centered_equation(0));
    let want = body_x + (body_w - eq_w) / 2.0;
    assert!(
        (eq_x - want).abs() <= 2.0,
        "가운데 정렬 문단의 본문 수식이 x={eq_x:.1} 에 놓였다 (가운데는 {want:.1}) (#5583)"
    );
}

#[test]
fn issue_5583_stored_line_start_still_wins() {
    // 저장 LINE_SEG 가 흐름 x 를 담고 있으면(#1256/#1308) 그 값을 존중한다 —
    // 정렬 오프셋으로 덮어쓰지 않는다.
    let ((eq_x, eq_w), (body_x, body_w)) =
        equation_and_body(&document_with_centered_equation(6000));
    let centered = body_x + (body_w - eq_w) / 2.0;
    assert!(
        (eq_x - centered).abs() > 2.0,
        "저장 column_start 가 있는 줄까지 가운데로 옮기면 #1256/#1308 계약이 깨진다 (x={eq_x:.1})"
    );
}
