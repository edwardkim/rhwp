//! [Issue #6134] 자리차지 표와 같은 문단에 앵커된 "문단 기준" 글앞으로 로고 글상자를
//! 표 **위**에 얹어 담당부서 표의 이름 칸을 가린다 (156731730 8쪽).
//!
//! 근인: 한글은 자리차지 개체를 문단 글줄 **위**에 놓는다 — 그래서 그 문단의 글줄은
//! 밴드 아래로 내려가고(저장 lineseg `vpos` 가 그 자리), 같은 문단에 매달린 다른
//! 개체의 "문단 기준" 세로 오프셋도 그 글줄을 기준으로 잰다. rhwp 는 기준점을 문단
//! 상단(= 밴드 상단)으로 잡아 글상자를 표 위에 얹었다.
//!
//! 한글 2024 PDF 실측(원본 8쪽): 표 756.6~995.6, 로고 이미지 1010.7~1052.9 —
//! 로고는 표 **아래**다. rhwp 는 765.1 로 표 위였다.
//!
//! 재현물은 원본(17MB)의 문단 87..88 창을 잘라낸 IR 슬라이스이고, 이미지는 1×1 PNG
//! 로 갈아 끼웠다(위치·크기는 컨트롤 속성이 결정하므로 화소는 무관).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6134/156731730_contact_table_logo_overlay.hwpx";
/// 담당부서 표(자리차지)와 로고 글상자(글앞으로)를 함께 매단 문단.
const HOST_PARA: usize = 1;

#[test]
fn issue_6134_overlay_anchor_sits_below_the_float_band() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    let page = core.build_page_render_tree(0).expect("page 1 render tree");
    let table_bottom = host_table_bottom(&page.root).expect("담당부서 표");
    let logo_top = images(&page.root)
        .into_iter()
        .map(|(top, _)| top)
        .fold(f64::INFINITY, f64::min);
    assert!(logo_top.is_finite(), "로고 글상자의 이미지를 찾지 못했다");

    assert!(
        logo_top >= table_bottom,
        "글앞으로 로고 글상자의 기준점은 자리차지 밴드 아래 글줄이어야 한다 — \
         표 아래끝={table_bottom:.1}, 로고 위끝={logo_top:.1}"
    );
}

/// 호스트 문단이 매단 자리차지 표의 아래끝.
fn host_table_bottom(node: &RenderNode) -> Option<f64> {
    let own = match &node.node_type {
        RenderNodeType::Table(table) if table.para_index == Some(HOST_PARA) => {
            Some(node.bbox.y + node.bbox.height)
        }
        _ => None,
    };
    node.children
        .iter()
        .filter_map(host_table_bottom)
        .chain(own)
        .fold(None, |acc: Option<f64>, bottom| {
            Some(acc.map_or(bottom, |best: f64| best.max(bottom)))
        })
}

/// 페이지의 모든 이미지 노드 (위끝, 아래끝).
fn images(node: &RenderNode) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    if matches!(node.node_type, RenderNodeType::Image(_)) {
        out.push((node.bbox.y, node.bbox.y + node.bbox.height));
    }
    for child in &node.children {
        out.extend(images(child));
    }
    out
}
