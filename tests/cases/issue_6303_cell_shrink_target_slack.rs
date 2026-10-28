//! [Issue #6303] 칸 폭 자동 축소(#6196) 의 목표 폭이 1~2% 헐거우면
//! 긴 행 꼬리가 괘선 밖으로 나가 잘린다.
//!
//! #6196 은 저장 사다리 한 줄 칸에서 자간 압축을 다시 켰다. 압축 자체는
//! 동작하지만 선형 `slack/N` 한 번이면 실측 폭이 안쪽 폭에 못 미친다.
//! 짧은 행(축소 불필요)은 그대로 두고, 긴 행만 괘선 안에 들어가게 한다.
//!
//! 표본은 #6196 과 같다 — 그 문서 4쪽 마지막 열에도 잔여 +0.68pt 가 남는다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6196/cell_char_spacing_fit.hwp";
/// 마지막 열 칸의 좌변 — 이 x 이상에서 시작하는 칸만 본다.
const LAST_COL_MIN_X: f64 = 470.0;
/// 한글은 긴 행을 괘선에 맞춘다. 0.5px 는 #6196 잠금보다 타이트하고,
/// 잔여 +0.68pt(~0.9px) 를 놓치지 않는다.
const BORDER_SLACK_PX: f64 = 0.25;

#[test]
fn issue_6303_long_cell_row_tail_stays_inside_border() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let page = core.build_page_render_tree(0).expect("page 1 render tree");

    let mut overflow = Vec::new();
    let mut checked = 0usize;
    let mut max_over = 0.0f64;
    walk(&page.root, None, &mut checked, &mut overflow, &mut max_over);

    assert!(
        checked > 0,
        "마지막 열 칸의 텍스트를 찾지 못했다 — 표본이 바뀌었는지 확인하라"
    );
    assert!(
        overflow.is_empty(),
        "긴 행 꼬리가 괘선을 넘는다 — {}건 max {:+.2}px: {:?}",
        overflow.len(),
        max_over,
        overflow
    );
}

fn walk(
    node: &RenderNode,
    cell: Option<f64>,
    checked: &mut usize,
    out: &mut Vec<(String, String)>,
    max_over: &mut f64,
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
            let over = node.bbox.x + node.bbox.width - right;
            if over > *max_over {
                *max_over = over;
            }
            if over > BORDER_SLACK_PX {
                out.push((format!("{:+.2}", over), run.text.clone()));
            }
        }
    }
    for child in &node.children {
        walk(child, cell, checked, out, max_over);
    }
}
