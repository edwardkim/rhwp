//! [Issue #6389] 편람 p68 본문 표 셀의 `○` 문단 줄들이 셀 우측 테두리를 +72~85px 넘는다.
//!
//! 이 셀 문단들의 저장 사다리(2~4줄, 각 `segment_width=37560HU`=셀 안쪽 폭)는
//! kopub/no-ttf 오라클 PDF 와 문자 단위로 일치한다 — 한글이 이 내용을 이 폭에
//! 담았다는 증언이다. 그런데 내장 KoPub돋움체 한글 진행폭(1.0em)이 오라클
//! 실측(≈0.83em)보다 넓어 자연 폭이 줄 상자의 1.12~1.17배가 되고, 억제 임계
//! (1.15)를 넘은 줄은 압축이 꺼져 자연 폭 그대로 칸 밖에 그려졌다.
//!
//! #6196 이 한 줄 사다리에 연 증언 예외를 다줄 사다리로 일반화한다: 조합이
//! 저장 줄 수를 그대로 따랐고 모든 저장 줄폭이 셀 열폭 이내면 억제하지 않고
//! 압축해 칸 안에 맞춘다. 열폭보다 넓게 기록된 사다리([sw>w] 낡은 캐시)는
//! 증언이 성립하지 않아 종전대로 클리핑된다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/2025 행정업무운영 편람(최종).hwp";
/// 대상 셀의 식별 텍스트 — p68(0기준 67) 본문 표 r1c1 첫 문단.
const CELL_MARKER: &str = "모든 기록물";
/// 줄 끝 공백 한 칸은 한글도 칸 밖으로 걸치므로 허용한다(반각 ≈ 6.7px + 여유).
const TRAILING_SPACE_ALLOWANCE_PX: f64 = 8.0;

#[test]
fn issue_6389_manual_p68_stored_ladder_cell_stays_inside_cell() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let page = core.build_page_render_tree(67).expect("p68 render tree");

    let mut overflow = Vec::new();
    let mut checked = 0usize;
    walk(&page.root, None, false, &mut checked, &mut overflow);

    assert!(
        checked > 0,
        "대상 셀({CELL_MARKER})의 텍스트를 찾지 못했다 — 쪽 배분이 바뀌었는지 확인하라"
    );
    assert!(
        overflow.is_empty(),
        "저장 사다리 셀의 줄이 칸 밖으로 나간다 — {}건 (칸 우변 초과 px, 텍스트): {:?}",
        overflow.len(),
        overflow
    );
}

/// 대상 셀(marker 텍스트를 담은 셀) 안 TextRun 중 칸 우변을 넘는 것을 모은다.
fn walk(
    node: &RenderNode,
    cell_right: Option<f64>,
    in_target_cell: bool,
    checked: &mut usize,
    out: &mut Vec<(String, String)>,
) {
    let (cell_right, in_target_cell) = match &node.node_type {
        RenderNodeType::TableCell(_) => {
            let holds_marker = subtree_contains(node, CELL_MARKER);
            (
                holds_marker.then_some(node.bbox.x + node.bbox.width),
                holds_marker,
            )
        }
        _ => (cell_right, in_target_cell),
    };
    if let (Some(right), true, RenderNodeType::TextRun(run)) =
        (cell_right, in_target_cell, &node.node_type)
    {
        if !run.text.trim().is_empty() {
            *checked += 1;
            let run_right = node.bbox.x + node.bbox.width;
            if run_right > right + TRAILING_SPACE_ALLOWANCE_PX {
                out.push((format!("+{:.1}", run_right - right), run.text.clone()));
            }
        }
    }
    for child in &node.children {
        walk(child, cell_right, in_target_cell, checked, out);
    }
}

fn subtree_contains(node: &RenderNode, needle: &str) -> bool {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains(needle) {
            return true;
        }
    }
    node.children.iter().any(|c| subtree_contains(c, needle))
}
