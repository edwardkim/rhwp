//! [Issue #6196] 표 마지막 열 `우수 내용` 칸에서 문장 꼬리가 칸 밖으로 나가 잘린다.
//!
//! 근인은 셀 오버플로우 자간 압축의 **억제 임계**다:
//!
//! ```text
//! suppress_cell_overflow_spacing = cell_ctx.is_some()
//!     && total_text_width > available_width * 1.15
//! ```
//!
//! 이 칸의 자연 폭은 290~331px, 셀 안쪽 폭은 229.2px 이라 ratio 가 1.27~1.44 —
//! 임계를 넘어 압축이 꺼지고 문장이 칸 밖으로 나간다. 1.15 는 `task 1443` 에서
//! exam_kor·복학원서 골든을 지키려고 고른 **경험적 수치**이지 계약이 아니다.
//!
//! 정답지는 저장 사다리에 있다 — 이 셀 문단은 `line_segs.len()==1` 이고 그
//! `segment_width = 17188HU = 229.2px` 로 **셀 안쪽 폭과 정확히 같다**. 즉 한글이
//! 이 문장을 이 폭 한 줄에 담았다는 증언이다(문서가 선언한 자간은 0 이므로 한글이
//! 렌더 시점에 줄인 것이다). 그 증언이 있으면 억제하지 않는다.
//!
//! | | rhwp(수정 전) | rhwp(수정 후) | 한글 2020 |
//! |---|---|---|---|
//! | 한 글자 전진 | 13.333px (1.000 em) | ~10.4px (~0.78 em) | 10.42px |
//! | 행 우변 | 716.9~726.6 (칸 714.5 밖) | ~708~711 | 707~713 |
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6196/cell_char_spacing_fit.hwp";
/// 마지막 열 칸의 좌변 — 이 x 이상에서 시작하는 칸만 본다.
const LAST_COL_MIN_X: f64 = 470.0;

#[test]
fn issue_6196_stored_single_line_cell_compresses_to_fit() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let page = core.build_page_render_tree(0).expect("page 1 render tree");

    let mut overflow = Vec::new();
    let mut checked = 0usize;
    walk(&page.root, None, &mut checked, &mut overflow);

    assert!(
        checked > 0,
        "마지막 열 칸의 텍스트를 찾지 못했다 — 표본이 바뀌었는지 확인하라"
    );
    assert!(
        overflow.is_empty(),
        "칸 안 문장이 칸 밖으로 나가 잘린다 — {}건 (칸 우변 초과 px, 텍스트): {:?}",
        overflow.len(),
        overflow
    );
}

/// 마지막 열 칸 안 TextRun 중 칸 우변을 넘는 것을 모은다.
fn walk(
    node: &RenderNode,
    cell: Option<f64>,
    checked: &mut usize,
    out: &mut Vec<(String, String)>,
) {
    let cell = match &node.node_type {
        RenderNodeType::TableCell(_) if node.bbox.x >= LAST_COL_MIN_X => {
            Some(node.bbox.x + node.bbox.width)
        }
        RenderNodeType::TableCell(_) => None,
        _ => cell,
    };
    if let (Some(right), RenderNodeType::TextRun(run)) = (cell, &node.node_type) {
        if !run.text.trim().is_empty() {
            *checked += 1;
            let run_right = node.bbox.x + node.bbox.width;
            if run_right > right + 0.5 {
                out.push((format!("+{:.1}", run_right - right), run.text.clone()));
            }
        }
    }
    for child in &node.children {
        walk(child, cell, checked, out);
    }
}
