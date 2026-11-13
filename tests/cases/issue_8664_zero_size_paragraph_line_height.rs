//! 검사 유형: 관계형 — 명시적 크기 0 문단의 흐름과 잉크 없는 replay를 구분한다.
//! 주석 갱신: 2026-11-12T20:12:56+09:00; 직전 파일 커밋: 5e2f05b9ac, 2026-11-12T12:23:33+09:00.
//! 시각 근거: source 8fe7d1fb8b, 96 DPI·2px raw, Native 99.75% / fresh WASM 99.72%.
//! 현재 기준은 사용자 확인 한컴 2024 13.0.0.3901 Print
//! `pdf/issue8664/zero_size_para_before_table-hwp-2024.pdf`다. 아래 웹 출력 설명은 원 결함의 이력이다.
//!
//! 한컴 웹한글기안기 인쇄 PDF(`pdf/issue8664/zero_size_para_before_table-webhwp-print.pdf`)에서
//! 저장 줄이 없는 문단 `TOP` 과 글자처럼 취급한 표 사이에 글자 크기 0(장평 0) 글자 137 개만 든
//! 문단(줄간격 비율 160%)이 있어도 표는 그 문단이 없을 때와 같은 자리에 놓인다
//! (`TOP` 글자 윗변에서 표 안 `TABLE` 글자 윗변까지 84.00pt, 0 크기 문단을 뺀 문서도 84.00pt).
//!
//! 수정 전 rhwp 는 글자 크기 0 을 "크기 없음"으로 보고 대체값 12px 로 줄을 세워
//! 표를 한 줄(19.2px)만큼 아래로 밀었다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::{
    renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType, TextRunNode},
    DocumentCore,
};

const SAMPLE: &str = "samples/issue8664/zero_size_para_before_table.hwp";

fn text_line_of<'a>(node: &'a RenderNode, para: usize) -> Option<&'a RenderNode> {
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if line.para_index == Some(para) {
            return Some(node);
        }
    }
    node.children
        .iter()
        .find_map(|child| text_line_of(child, para))
}

fn zero_run_mut(node: &mut rhwp::paint::LayerNode) -> Option<&mut TextRunNode> {
    use rhwp::paint::{LayerNodeKind, PaintOp};
    match &mut node.kind {
        LayerNodeKind::Group { children, .. } => children.iter_mut().find_map(zero_run_mut),
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            zero_run_mut(child)
        }
        LayerNodeKind::Leaf { ops } => ops.iter_mut().find_map(|op| match op {
            PaintOp::TextRun { run, .. } if run.text.starts_with('Z') => Some(run.as_mut()),
            _ => None,
        }),
    }
}

#[test]
fn explicit_zero_size_text_is_a_direct_non_painting_canvaskit_operation() {
    use rhwp::renderer::canvaskit_policy::{analyze_canvaskit_replay_plan, CanvasKitReplayMode};
    let bytes = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).unwrap();
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    let mut tree = core.build_page_layer_tree(0).unwrap();
    let zero = zero_run_mut(&mut tree.root).expect("크기 0 원문 run");
    assert_eq!(zero.style.font_size, 0.0);
    assert_eq!(zero.style.ratio, 0.0);
    assert_eq!(zero.text.chars().count(), 137, "숨은 원문을 지우지 않는다");
    for mode in [CanvasKitReplayMode::Compat, CanvasKitReplayMode::Default] {
        let plan = analyze_canvaskit_replay_plan(&tree, mode);
        assert!(
            plan.items
                .iter()
                .all(|item| item.detail.as_deref() != Some("invalidGeometry")),
            "잉크 없는 크기 0 원문을 잘못된 기하로 분류했다: {mode:?}"
        );
    }
}

#[test]
fn zero_size_replay_does_not_accept_negative_or_non_finite_geometry() {
    use rhwp::renderer::canvaskit_policy::{analyze_canvaskit_replay_plan, CanvasKitReplayMode};
    let bytes = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).unwrap();
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    for (font_size, ratio) in [
        (12.0, 0.0),
        (-1.0, 0.0),
        (0.0, -1.0),
        (f64::NAN, 0.0),
        (0.0, f64::INFINITY),
    ] {
        let mut tree = core.build_page_layer_tree(0).unwrap();
        let zero = zero_run_mut(&mut tree.root).unwrap();
        zero.style.font_size = font_size;
        zero.style.ratio = ratio;
        let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);
        assert!(
            plan.items
                .iter()
                .any(|item| item.detail.as_deref() == Some("invalidGeometry")),
            "음수·비유한 값 또는 크기 있는 장평 0 run을 허용했다: {font_size:?}, {ratio:?}"
        );
    }
}

fn table_bbox(node: &RenderNode) -> Option<BoundingBox> {
    if let RenderNodeType::Table(_) = &node.node_type {
        return Some(node.bbox.clone());
    }
    node.children.iter().find_map(table_bbox)
}

#[test]
fn zero_size_paragraph_adds_no_line_before_inline_table() {
    let bytes = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE))
        .expect("글자 크기 0 문단 재현물");
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    assert_eq!(core.page_count(), 1, "웹한글기안기 인쇄와 같은 쪽수");
    let source = &core.document().sections[0].paragraphs[1];
    assert!(
        !source.char_shapes.is_empty()
            && source.char_shapes.iter().all(|shape| {
                core.document().doc_info.char_shapes[shape.char_shape_id as usize].base_size == 0
            }),
        "재현물 문단 1 은 글자 크기 0 글자만 가진다"
    );

    let tree = core.build_page_render_tree(0).unwrap();
    let top = text_line_of(&tree.root, 0).expect("문단 0(TOP) 줄");
    let table = table_bbox(&tree.root).expect("글자처럼 취급한 표");

    // TOP 줄(10pt, 160%)의 줄 피치는 16pt = 21.33px 이다. 0 크기 문단이 줄을 만들지 않으면
    // 표 윗변은 TOP 줄 윗변에서 한 줄 피치와 표 바깥 여백만큼 떨어진다. 대체 크기 12px 로
    // 줄을 세우면 19.2px(12px × 160%)가 더해지므로 그 절반을 경계로 둔다.
    let one_line_pitch = 16.0 * 96.0 / 72.0;
    let gap = table.y - top.bbox.y;
    assert!(
        gap < one_line_pitch + 9.6,
        "0 크기 문단이 줄 높이를 만들어 표가 밀렸다: TOP→표 {gap:.2}px, 한 줄 피치 {one_line_pitch:.2}px"
    );

    // 0 크기 문단 뒤 문단(AFTER)도 표 아래에서 한 줄 피치 안에 이어진다.
    let after = text_line_of(&tree.root, 3).expect("문단 3(AFTER) 줄");
    let table_bottom = table.y + table.height;
    assert!(
        after.bbox.y >= table_bottom - 0.5 && after.bbox.y - table_bottom < one_line_pitch,
        "AFTER 줄은 표 바로 아래에 있다: 표 아래 {table_bottom:.2}px, AFTER {:.2}px",
        after.bbox.y
    );
    if let Some(zero) = text_line_of(&tree.root, 1) {
        assert!(
            zero.bbox.y <= table.y + 0.5,
            "0 크기 문단 줄은 표 위에서 시작한다"
        );
    }
}
