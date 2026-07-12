//! Issue #1898 기전 1: tac(글자처럼) 인라인 그림 문단의 렌더 줄 전진 과대 회귀 가드.
//!
//! tac 그림의 PageItem::Shape 처리가 vpos base 를 리셋한 뒤, 다음 문단의 lazy_base
//! 재역산에서 spacing_before 가 저장 vpos 에 인코딩된 gap(=sb)을 불연속으로 오판해
//! +trailing_ls bridge 를 이중 가산 — 그림 문단 뒤 줄 전진이 44.8px 로 layout(33.1px)
//! ·한컴 오라클(32.9px) 대비 line gap 1회분(+11.7px) 과대해졌다.
//!
//! fixture: `samples/hwpx/opengov/36388711_...hwpx` p9 — 2.5mm tac 불릿 그림 문단
//! (pi=87/95/96/97, ps line=180%). 오라클: 한글 2022 PDF (mutool stext) 라인 pitch
//! 24.7pt(32.9px) 균일 — 그림 유무와 무관.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

const SAMPLE: &str =
    "samples/hwpx/opengov/36388711_사회보장제도 신설 협의요청서(청년오피스)_260624.hwpx";

fn page_tree(page: u32) -> rhwp::renderer::render_tree::PageRenderTree {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&p).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let doc =
        rhwp::wasm_api::HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse: {e:?}"));
    doc.build_page_render_tree(page)
        .unwrap_or_else(|e| panic!("render p{}: {e}", page + 1))
}

/// pi 의 첫 TextLine y (본문 단 직속 — 셀 내부 pi 충돌 방지를 위해 Table 하위는 제외).
fn first_text_line_y(node: &RenderNode, pi: usize) -> Option<f64> {
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        return None;
    }
    if let RenderNodeType::TextLine(tl) = &node.node_type {
        if tl.para_index == Some(pi) {
            return Some(node.bbox.y);
        }
    }
    node.children
        .iter()
        .filter_map(|c| first_text_line_y(c, pi))
        .fold(None, |acc, y| Some(acc.map_or(y, |a: f64| a.min(y))))
}

/// tac 그림 문단(pi=87/95/96) 다음 줄까지의 렌더 전진이 layout 전진(33.1px)과 일치.
/// 수정 전에는 세 곳 모두 44.8px(+line gap 1회분)로 확실히 구분된다.
#[test]
fn tac_image_paragraph_render_advance_matches_layout() {
    let tree = page_tree(8); // p9 (0-based)

    // (그림 문단 pi, 다음 문단 pi). 기대 전진 = lh(14.67) + ls(11.73) + sb(6.67) = 33.1px.
    for (img_pi, next_pi) in [(87usize, 88usize), (95, 96), (96, 97)] {
        let y_img =
            first_text_line_y(&tree.root, img_pi).unwrap_or_else(|| panic!("pi={img_pi} TextLine"));
        let y_next = first_text_line_y(&tree.root, next_pi)
            .unwrap_or_else(|| panic!("pi={next_pi} TextLine"));
        let advance = y_next - y_img;
        assert!(
            (advance - 33.1).abs() <= 3.0,
            "#1898: tac 그림 문단 pi={img_pi} → pi={next_pi} 렌더 전진 {advance:.1}px \
             (기대 33.1±3) — trailing_ls bridge 이중 가산(+11.7px) 회귀",
        );
    }
    // 대조군: 그림 없는 pi=93(빈) → pi=94 는 수정 전에도 정상(26.4px, sb=0).
    let y93 = first_text_line_y(&tree.root, 93).expect("pi=93");
    let y94 = first_text_line_y(&tree.root, 94).expect("pi=94");
    assert!(
        ((y94 - y93) - 26.4).abs() <= 3.0,
        "#1898 대조군: pi=93→94 전진 {:.1}px (기대 26.4±3)",
        y94 - y93,
    );
}
