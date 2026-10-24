//! [#8167] 편집으로 넣은 글자처럼 취급 도형은 자기 줄을 도형 높이만큼 키우고, 줄의 글자는
//! 그 줄의 기준선(도형 아래 변 쪽)에 놓인다.
//!
//! 결함 두 겹:
//! 1. `create_shape_control_native` 가 글자처럼 취급 도형을 넣고 host 문단의 줄을 다시 나누지
//!    않았다. 저장 줄이 글자 높이(1000HU)로 남아 도형이 줄 위(본문 위 여백)로 넘쳐 그려졌다.
//! 2. 본문의 «도형 + 글자» 줄은 줄 높이·기준선을 글자 크기로 접어, 줄을 다시 나눠도 글자가
//!    줄 맨 위 글자 높이 자리에 남았다.
//!
//! 독립 기준: 같은 편집을 저장한 HWPX 를 한/글 2024(13.0.0.564) Print 로 출력하면, 글상자·
//! 사각형·글자처럼 취급 그림 세 경우 모두 줄이 도형 높이만큼 내려가고 앞뒤 글자 「앞」·「글 뒤 글」
//! 이 도형 아래 변 높이에 놓이며 「다음 문단」은 도형 아래에 온다. 입력은 이 테스트가 편집 API
//! 로 만드는 합성 문서다(저장 LineSeg 는 rhwp 재조판 결과).

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

/// 도형 크기 17008 × 5669 HU (60mm × 20mm).
const SHAPE_W: u32 = 17008;
const SHAPE_H: u32 = 5669;

fn edited(shape_type: Option<&str>) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().expect("blank");
    core.insert_text_native(0, 0, 0, "앞 글 뒤 글")
        .expect("text");
    core.split_paragraph_native(0, 0, 7, None).expect("split");
    core.insert_text_native(0, 1, 0, "다음 문단")
        .expect("text 2");
    if let Some(shape_type) = shape_type {
        core.create_shape_control_native(
            0,
            0,
            2,
            SHAPE_W,
            SHAPE_H,
            0,
            0,
            true,
            "Square",
            shape_type,
            false,
            false,
            &[],
        )
        .expect("shape");
    }
    core
}

fn text(node: &RenderNode) -> String {
    let mut out = match &node.node_type {
        RenderNodeType::TextRun(run) => run.text.clone(),
        _ => String::new(),
    };
    for child in &node.children {
        out.push_str(&text(child));
    }
    out
}

/// 도형 밖 본문 글줄 `(줄 상자, 기준선 y)` — `needle` 을 담은 첫 줄.
fn body_line(node: &RenderNode, needle: &str) -> Option<(BoundingBox, f64)> {
    if matches!(
        node.node_type,
        RenderNodeType::Rectangle(_) | RenderNodeType::TextBox | RenderNodeType::Group(_)
    ) {
        return None;
    }
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if text(node).contains(needle) {
            return Some((node.bbox, node.bbox.y + line.baseline));
        }
    }
    node.children
        .iter()
        .find_map(|child| body_line(child, needle))
}

/// 넣은 도형의 사각형 상자(높이가 도형 높이와 같은 사각형).
fn shape_box(node: &RenderNode, shape_h_px: f64) -> Option<BoundingBox> {
    if matches!(node.node_type, RenderNodeType::Rectangle(_))
        && (node.bbox.height - shape_h_px).abs() < 1.0
    {
        return Some(node.bbox);
    }
    node.children
        .iter()
        .find_map(|child| shape_box(child, shape_h_px))
}

fn body_top(core: &DocumentCore) -> f64 {
    core.dump_page_items_json(None).as_array().unwrap()[0]["bodyArea"]["y"]
        .as_f64()
        .unwrap()
}

fn check_tac_shape_line(shape_type: &str) {
    let core = edited(Some(shape_type));
    let host = &core.document().sections[0].paragraphs[0];
    assert!(
        host.line_segs[0].line_height >= SHAPE_H as i32,
        "{shape_type}: host 첫 줄 높이 {}HU 가 도형 높이 {SHAPE_H}HU 를 점유해야 한다",
        host.line_segs[0].line_height
    );

    let tree = core.build_page_render_tree(0).expect("page 1");
    let shape_h_px = SHAPE_H as f64 * 96.0 / 7200.0;
    let shape = shape_box(&tree.root, shape_h_px).expect("도형 사각형");
    let (line, baseline) = body_line(&tree.root, "앞").expect("host 줄 「앞」");
    let (next, _) = body_line(&tree.root, "다음").expect("다음 문단 줄");
    let top = body_top(&core);

    assert!(
        shape.y >= top - 0.5,
        "{shape_type}: 도형 위({:.1})가 본문 위({top:.1}) 밖으로 넘치지 않는다",
        shape.y
    );
    assert!(
        shape.y >= line.y - 0.5 && shape.y + shape.height <= line.y + line.height + 0.5,
        "{shape_type}: 도형({:.1}–{:.1})이 host 줄 상자({:.1}–{:.1}) 안에 든다",
        shape.y,
        shape.y + shape.height,
        line.y,
        line.y + line.height
    );
    assert!(
        baseline >= shape.y + shape.height * 0.5 && baseline <= shape.y + shape.height + 0.5,
        "{shape_type}: 줄 글자 기준선({baseline:.1})이 도형({:.1}–{:.1}) 아래쪽 절반에 놓인다",
        shape.y,
        shape.y + shape.height
    );
    assert!(
        next.y >= shape.y + shape.height - 0.5,
        "{shape_type}: 다음 문단({:.1})이 도형 아래({:.1})에 온다",
        next.y,
        shape.y + shape.height
    );
}

#[test]
fn inserted_tac_textbox_grows_its_line_and_sits_on_the_baseline() {
    check_tac_shape_line("textbox");
}

#[test]
fn inserted_tac_rectangle_grows_its_line_and_sits_on_the_baseline() {
    check_tac_shape_line("rectangle");
}

/// 대조군: 도형이 없으면 host 줄은 글자 높이이고 다음 문단이 바로 이어진다.
#[test]
fn text_only_line_keeps_text_height() {
    let core = edited(None);
    let host = &core.document().sections[0].paragraphs[0];
    assert!(host.line_segs[0].line_height < SHAPE_H as i32 / 2);
    let tree = core.build_page_render_tree(0).expect("page 1");
    let (line, _) = body_line(&tree.root, "앞").expect("host 줄");
    let (next, _) = body_line(&tree.root, "다음").expect("다음 문단 줄");
    assert!(next.y >= line.y + line.height - 0.5);
    assert!(
        next.y - line.y < 40.0,
        "글자 줄 뒤 다음 문단은 줄 하나 거리"
    );
}
