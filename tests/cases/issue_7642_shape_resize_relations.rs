#![cfg(not(target_arch = "wasm32"))]

//! #7642: 독립 Print를 통과한 도형 리사이즈의 개체 소유·길이 비율·Undo 계약.
//! 기존 시험지 검사 삭제 PR과 구분하기 위해 새 파일·새 검사 이름으로 등록한다.
//!
//! 표본 `21_언어_기출_편집가능본.hwp` s0p4c0 — 한컴 저장 가로선 `common 31804×4`,
//! `original 100×100`. 생성 시 크기를 덮으면 폭 리사이즈의 표시 비율이 무너진다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;

const LINE: &str = "samples/21_언어_기출_편집가능본.hwp";
const LINE_AT: (usize, usize, usize) = (0, 4, 0);

fn load() -> DocumentCore {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(LINE);
    DocumentCore::from_bytes(&std::fs::read(p).expect("표본 로드")).expect("파싱")
}

/// `(common.w, common.h, original_w, original_h, current_w, current_h)`.
fn geometry(core: &DocumentCore) -> (u32, u32, u32, u32, u32, u32) {
    let (si, pi, ci) = LINE_AT;
    let s = match &core.document().sections[si].paragraphs[pi].controls[ci] {
        Control::Shape(s) => s.as_ref(),
        _ => panic!("표본 전제 위반: s{si}p{pi}c{ci} 가 도형이 아니다"),
    };
    (
        s.common().width,
        s.common().height,
        s.shape_attr().original_width,
        s.shape_attr().original_height,
        s.shape_attr().current_width,
        s.shape_attr().current_height,
    )
}

fn resize_to(core: &mut DocumentCore, w: u32, h: u32) {
    let (si, pi, ci) = LINE_AT;
    core.set_shape_properties_native(si, pi, ci, &format!("{{\"width\":{w},\"height\":{h}}}"))
        .expect("set_shape_properties_native");
}

/// 화면 좌표로 검색하지 않고 원본 문단·컨트롤이 소유한 실제 선을 고른다.
fn rendered_line(core: &DocumentCore) -> rhwp::renderer::render_tree::LineNode {
    use rhwp::renderer::render_tree::{LineNode, RenderNode, RenderNodeType};
    fn collect<'a>(node: &'a RenderNode, lines: &mut Vec<&'a LineNode>) {
        if let RenderNodeType::Line(line) = &node.node_type {
            let (section, para, control) = LINE_AT;
            if line.section_index == Some(section)
                && line.para_index == Some(para)
                && line.control_index == Some(control)
            {
                lines.push(line);
            }
        }
        for child in &node.children {
            collect(child, lines);
        }
    }
    let tree = core.build_page_render_tree(0).expect("첫 쪽의 실제 배치");
    let mut lines = Vec::new();
    collect(&tree.root, &mut lines);
    assert_eq!(lines.len(), 1, "같은 원본 도형이 한 번 표시되어야 함");
    (*lines[0]).clone()
}

fn assert_premise(core: &DocumentCore) -> (u32, u32) {
    let (w, h, ow, oh, ..) = geometry(core);
    assert!(h < w, "원본 가로선의 폭이 높이보다 커야 함");
    assert_eq!(
        (ow, oh),
        (100, 100),
        "표본 전제: 한컴이 저장한 생성 시 크기가 100×100 이어야 스케일 판정이 선다"
    );
    assert_ne!(ow, w, "표본 전제: original({ow}) ≠ common({w})");
    (w, h)
}

#[test]
fn issue7642_resize_keeps_owned_line_length_ratio() {
    let mut core = load();
    let (w0, h0) = assert_premise(&core);
    let before = rendered_line(&core);
    let before_width = (before.x2 - before.x1).abs();
    assert!(before_width > 0.0, "원본 선이 실제로 그려져야 함");

    resize_to(&mut core, w0 / 2, h0);

    let after = rendered_line(&core);
    let expected_ratio = f64::from(w0 / 2) / f64::from(w0);
    let observed_ratio = (after.x2 - after.x1).abs() / before_width;
    assert!(
        (observed_ratio - expected_ratio).abs() < 1e-9,
        "실제 선의 가로 길이는 요청한 폭 비율을 따라야 함: {observed_ratio} / {expected_ratio}"
    );
    assert_eq!(
        after.y2 - after.y1,
        before.y2 - before.y1,
        "폭만 바꾸면 선의 세로 성분은 보존되어야 함"
    );
    assert_eq!(
        geometry(&core),
        (w0 / 2, h0, 100, 100, w0 / 2, h0),
        "`current_*` 만 새 크기를 따라가고 `original_*` 는 생성 시 값으로 남아야 한다"
    );
}

#[test]
fn issue7642_resize_undo_restores_owned_line_endpoints() {
    // studio 의 `ResizeObjectCommand.undo` 는 게터가 냈던 원래 크기를 다시 넣는다
    // (`command.ts` `setProps` → `setShapeProperties`).
    let mut core = load();
    let (w0, h0) = assert_premise(&core);
    let before_geometry = geometry(&core);
    let before_line = rendered_line(&core);

    resize_to(&mut core, w0 / 2, h0);
    resize_to(&mut core, w0, h0);

    assert_eq!(
        geometry(&core),
        before_geometry,
        "리사이즈 되돌리기가 개체 기하를 원래대로 돌리지 못했다"
    );
    assert_eq!(
        {
            let line = rendered_line(&core);
            (line.x1, line.y1, line.x2, line.y2)
        },
        (
            before_line.x1,
            before_line.y1,
            before_line.x2,
            before_line.y2
        ),
        "되돌린 원본 도형의 실제 끝점과 길이가 복원되어야 함"
    );
}

#[test]
fn issue7642_resize_updates_size_without_changing_creation_frame() {
    // 대조 — 보존을 넣었다고 실제 리사이즈가 막히면 안 된다.
    let mut core = load();
    let (w0, h0) = assert_premise(&core);

    resize_to(&mut core, w0 + 4000, h0 + 40);

    let (w, h, ..) = geometry(&core);
    assert_eq!((w, h), (w0 + 4000, h0 + 40), "실제 변경은 그대로 적용된다");
}
