//! [#6353] 머리말 셀 안 TAC 사각형은 저장 줄 폭(sw)에 붙는다.
//!
//! `samples/exam_kor.hwp` 6쪽 머리말 "홀수형" 박스(폭 ≈79.3px). 한/글 인쇄
//! 실측 좌단 924.0px. 종전엔 셀 유도 내폭(22206HU)을 써 저장 sw(22054HU)보다
//! 152HU(≈2.03px @96dpi) 오른쪽에 놓였다. 쪽-Right 부동 폴백이 아니라
//! 글자처럼 인라인 도형의 줄 폭 권위 문제다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/exam_kor.hwp";
/// 이슈 본문 6쪽 (0 기준 5).
const PAGE: u32 = 5;
/// 한/글 2022 인쇄 실측 좌단 @96dpi.
const HANGUL_BOX_X: f64 = 924.0;
/// 종전 rhwp 좌단. 이보다 한/글에 가까워야 한다.
const OLD_RHWP_BOX_X: f64 = 926.1;
const BOX_W_MIN: f64 = 78.5;
const BOX_W_MAX: f64 = 81.0;

fn load() -> DocumentCore {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|e| panic!("{} 읽기 실패: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("{SAMPLE} 파싱 실패: {e}"))
}

fn collect_header_boxes(node: &RenderNode, in_header: bool, out: &mut Vec<(f64, f64)>) {
    let in_header = in_header || matches!(node.node_type, RenderNodeType::Header);
    if in_header {
        let b = &node.bbox;
        if b.width >= BOX_W_MIN && b.width <= BOX_W_MAX && b.x > 800.0 {
            out.push((b.x, b.width));
        }
    }
    for child in &node.children {
        collect_header_boxes(child, in_header, out);
    }
}

#[test]
fn exam_kor_odd_header_box_follows_stored_line_width() {
    let core = load();
    assert!(
        PAGE < core.page_count(),
        "{SAMPLE} 쪽수={} < {}",
        core.page_count(),
        PAGE + 1
    );
    let tree = core
        .build_page_render_tree(PAGE)
        .expect("exam_kor 6쪽 렌더 트리");
    let mut boxes = Vec::new();
    collect_header_boxes(&tree.root, false, &mut boxes);
    assert!(
        !boxes.is_empty(),
        "6쪽 머리말에 폭 {BOX_W_MIN}..{BOX_W_MAX}px 박스가 없다"
    );
    let x = boxes.iter().map(|(x, _)| *x).fold(f64::INFINITY, f64::min);
    let hangul_err = (x - HANGUL_BOX_X).abs();
    let old_err = (x - OLD_RHWP_BOX_X).abs();
    assert!(
        hangul_err < old_err,
        "홀수형 박스 x={x:.2} 가 종전 {OLD_RHWP_BOX_X} 쪽에 가깝다 (한/글 {HANGUL_BOX_X})"
    );
    assert!(
        x <= HANGUL_BOX_X + 1.0,
        "홀수형 박스 x={x:.2} 가 한/글 {HANGUL_BOX_X} 보다 1px 넘게 오른쪽"
    );
}
