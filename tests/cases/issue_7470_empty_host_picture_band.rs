//! [#7470] 문단 기준 자리차지 그림의 빈 host 줄과 쪽 끝 그림 이월.
//!
//! 1. 한/글은 그림에 줄 폭 전체가 막힌 빈 host 줄을 `segment_width = 0` 으로 저장하고
//!    그 줄을 그림 띠 안에 흡수한다 — 다음 문단 저장 vpos − 현 vpos = 그림 높이.
//!    rhwp 는 배치(#683)와 조판(#409) 모두 그림 뒤에 host 한 줄을 더 전진했다.
//!    줄 폭이 남은 빈 host(pr-149, `sw > 0`)는 종전처럼 한 줄을 더 전진한다(Task #683 테스트).
//! 2. 그림 1장의 실제 하단(문단 시작 + 세로 오프셋 + 높이)이 본문을 넘으면 한/글은 host 줄은
//!    그 쪽에 두고 그림만 다음 쪽 맨 위로 넘긴다. 조판은 3장 이상 스택(#2814)만 넘겨서,
//!    host 줄이 들어가는 쪽에 그림까지 남겨 본문을 넘겼다.
//!
//! 기대값은 구현과 독립이다 — 원본 저장 줄 사다리와 한/글 2024 PDF(memo_field 40쪽, 27쪽 맨 위 그림).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const HWPUNIT_PX: f64 = 96.0 / 7200.0;

fn core(rel: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

/// 쪽 본문 영역의 (그림 노드 문단, y) 와 (글줄 문단, y) 를 모은다(머리말·꼬리말 제외).
fn page_nodes(core: &DocumentCore, page: u32) -> (Vec<(usize, f64)>, Vec<(usize, f64)>) {
    fn walk(
        n: &RenderNode,
        in_body: bool,
        images: &mut Vec<(usize, f64)>,
        lines: &mut Vec<(usize, f64)>,
    ) {
        let in_body = in_body || matches!(n.node_type, RenderNodeType::Body { .. });
        if in_body {
            match &n.node_type {
                RenderNodeType::Image(img) => {
                    if let Some(pi) = img.para_index {
                        images.push((pi, n.bbox.y));
                    }
                }
                RenderNodeType::TextLine(line) => {
                    if let Some(pi) = line.para_index {
                        lines.push((pi, n.bbox.y));
                    }
                }
                _ => {}
            }
        }
        for c in &n.children {
            walk(c, in_body, images, lines);
        }
    }
    let tree = core
        .build_page_render_tree(page)
        .unwrap_or_else(|e| panic!("{}쪽 render tree: {e:?}", page + 1));
    let (mut images, mut lines) = (Vec::new(), Vec::new());
    walk(&tree.root, false, &mut images, &mut lines);
    (images, lines)
}

fn stored_vpos(core: &DocumentCore, para: usize) -> i32 {
    core.document().sections[0].paragraphs[para].line_segs[0].vertical_pos
}

/// 156636617 4쪽 문단 74: 빈 host(`sw=0`) 자리차지 그림(높이 20409HU). 다음 문단 75 의 첫 줄은
/// 그림 위에서 저장 사다리 간격(= 그림 높이)만큼 아래에 온다(종전 +host 줄 24.96px).
#[test]
fn absorbed_empty_host_line_follows_stored_ladder() {
    let core = core("samples/156636617_240617 2024년 5월 월간 수출입 현황(확정치).hwp");
    let host = &core.document().sections[0].paragraphs[74];
    assert_eq!(host.line_segs.len(), 1);
    assert_eq!(
        host.line_segs[0].segment_width, 0,
        "전제: 그림에 막힌 host 줄"
    );
    let ladder_px = (stored_vpos(&core, 75) - stored_vpos(&core, 74)) as f64 * HWPUNIT_PX;

    let (images, lines) = page_nodes(&core, 3);
    let picture_top = images
        .iter()
        .find(|(pi, _)| *pi == 74)
        .map(|(_, y)| *y)
        .expect("4쪽에 문단 74 그림");
    let next_top = lines
        .iter()
        .filter(|(pi, _)| *pi == 75)
        .map(|(_, y)| *y)
        .fold(f64::INFINITY, f64::min);
    let step = next_top - picture_top;
    assert!(
        (step - ladder_px).abs() < 1.0,
        "그림 위 → 다음 문단 첫 줄 {step:.2}px 는 저장 사다리 {ladder_px:.2}px 여야 한다"
    );
}

/// memo_field 문단 331 도 같은 형상이다. 이 흡수로 26쪽 흐름이 22.4px 올라가 문단 335 의
/// host 줄이 26쪽에 들어가는데, 그 그림(775px)은 26쪽 본문을 넘으므로 27쪽 맨 위로 넘어가야 한다.
/// 독립 근거: 저장 사다리(문단 335 vpos 66523 은 26쪽 사다리를 잇고, 문단 336 은 58125 로
/// 되감긴다 = 한/글은 335 의 줄을 26쪽, 그림 뒤 흐름을 27쪽에 둠)와 한/글 2024 PDF(40쪽,
/// 27쪽은 그림 한 장).
#[test]
fn overflowing_single_picture_moves_to_next_page_top() {
    let core = core("samples/issue5866/memo_field_hwp5.hwp");
    assert!(
        stored_vpos(&core, 335) > stored_vpos(&core, 334)
            && stored_vpos(&core, 336) < stored_vpos(&core, 335),
        "전제: 저장 사다리가 335 뒤에서 되감긴다"
    );
    assert_eq!(core.page_count(), 40, "한/글 2024 PDF 40쪽");

    let (images26, lines26) = page_nodes(&core, 25);
    assert!(
        lines26.iter().any(|(pi, _)| *pi == 335),
        "문단 335 의 host 줄은 26쪽에 남는다: {lines26:?}"
    );
    assert!(
        !images26.iter().any(|(pi, _)| *pi == 335),
        "문단 335 그림은 26쪽에 그리지 않는다: {images26:?}"
    );

    let (images27, lines27) = page_nodes(&core, 26);
    let (_, pic_y) = *images27
        .iter()
        .find(|(pi, _)| *pi == 335)
        .expect("27쪽에 문단 335 그림");
    let body_top = core
        .build_page_render_tree(26)
        .expect("27쪽 render tree")
        .root
        .children
        .iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Body { .. }))
        .map(|n| n.bbox.y)
        .expect("27쪽 본문 영역");
    assert!(
        (pic_y - body_top).abs() < 0.5,
        "문단 335 그림은 27쪽 본문 맨 위({body_top:.1})에서 시작한다: {pic_y:.1}"
    );
    assert!(
        lines27
            .iter()
            .filter(|(pi, _)| *pi > 335)
            .all(|(_, y)| *y > pic_y),
        "27쪽 뒤 문단은 그림 아래에서 이어진다: 그림 {pic_y:.1}, {lines27:?}"
    );
}

/// 반례 — 그림 하단이 본문 안이면 host 줄과 함께 그대로 둔다(가로 쪽, 문단 시작 0 + 506px < 548px).
#[test]
fn fitting_single_picture_stays_on_its_page() {
    let core = core("samples/issue5595_rotated_picture_topbottom.hwpx");
    assert_eq!(core.page_count(), 1);
}
