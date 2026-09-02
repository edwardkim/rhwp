//! [#4318] 미주 다단 17쪽 오른쪽 단 마지막 줄이 본문 프레임 안에 남는다.
//!
//! `samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp` 17쪽. 한컴 PDF 는
//! 프레임 밖 본문이 0px 인데, rhwp 는 오른쪽 단 마지막 줄
//! (`의 가지 경우이므로 이 경우 구하는 확률은`) 이 본문 하단 1096.1px 을
//! 약 14px 넘겼다. 24px bleed 로 한 줄(≈12px)을 마지막 단에 남긴 것이 원인.
//!
//! `ENDNOTE_PAGE_OFFCANVAS_GUARD_PX`(56) 는 건드리지 않는다 — 낮추면 한글
//! 23쪽이 22쪽으로 줄어 #5886 이 열린다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp";
const PAGE: u32 = 16;
const HANGUL_PAGE_COUNT: u32 = 23;

fn load() -> DocumentCore {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|e| panic!("{} 읽기 실패: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("{SAMPLE} 파싱 실패: {e}"))
}

fn body_bottom(node: &RenderNode) -> Option<f64> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node.bbox.y + node.bbox.height);
    }
    node.children.iter().find_map(body_bottom)
}

fn collect_right_col_line_bottoms(node: &RenderNode, in_right: bool, out: &mut Vec<f64>) {
    let in_right = in_right || matches!(node.node_type, RenderNodeType::Column(1));
    let skip = matches!(
        node.node_type,
        RenderNodeType::Header | RenderNodeType::Footer
    );
    if skip {
        return;
    }
    if in_right {
        if let RenderNodeType::TextLine(_) = node.node_type {
            out.push(node.bbox.y + node.bbox.height);
        }
    }
    for child in &node.children {
        collect_right_col_line_bottoms(child, in_right, out);
    }
}

#[test]
fn sep2020_page17_right_column_last_line_stays_inside_body() {
    let core = load();
    assert_eq!(
        core.page_count(),
        HANGUL_PAGE_COUNT,
        "한글 23쪽을 유지해야 한다 (56px 용지밖 가드를 낮추면 22쪽으로 줄어 #5886)"
    );
    let tree = core.build_page_render_tree(PAGE).expect("17쪽 렌더 트리");
    let body_bottom = body_bottom(&tree.root).expect("Body");
    let mut bottoms = Vec::new();
    collect_right_col_line_bottoms(&tree.root, false, &mut bottoms);
    assert!(
        !bottoms.is_empty(),
        "17쪽 오른쪽 단에 TextLine 이 없다 body_bottom={body_bottom:.1}"
    );
    let deepest = bottoms.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    assert!(
        deepest <= body_bottom + 1.0,
        "17쪽 오른쪽 단 마지막 줄 bottom={deepest:.1} 이 본문 하단 {body_bottom:.1} 을 넘는다"
    );
}
