//! [Issue #6167] 공백만으로 시작하는 문단의 자리차지(TAC) 표가 **그 앞 공백 폭만큼**
//! 오른쪽으로 밀려 본문 우단·용지 밖으로 나가 오른쪽 열이 잘린다.
//!
//! 저장 `linesegarray` 는 이미 표에 **자기 줄**을 줬다:
//!
//! ```text
//! ctrl_pos=18
//! ls[0] text_start=0   vpos=2992  lh=1000   col_start=0
//! ls[1] text_start=18  vpos=3992  lh=63617  col_start=0   ← 표가 이 줄 머리에서 시작
//! ```
//!
//! 그런데 표가 **블록 취급**(`is_tac_table_inline_in_para` 가 `own_line_evidence` 로
//! false)이라 `composed.tac_controls` 에 없고, `compute_tac_leading_width` 는 그 경우
//! **line 0 의 run 전부**를 leading 으로 합산한다 — 공백 18자 = 120.0px.
//! 즉 "표가 자기 줄을 가진다"는 같은 증거를 한쪽은 쓰고 한쪽은 안 봤다.
//!
//! | | rhwp(수정 전) | rhwp(수정 후) | 한글 2020·2024 |
//! |---|---|---|---|
//! | 표 좌변 | 195.6 | 75.6 | 75.32 |
//! | 표 우변 | 801.7 (용지 793.7 밖) | 681.7 | 682.03 |
//!
//! 통제군 `samples/복학원서.hwp` pi=16 은 `ctrl_pos=99` ≠ `ls[1].text_start=198` —
//! 표가 `ls[0]` **안**에 있어 종전 leading 축(#1195)이 그대로 유효하다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6167/leading_space_tac_table.hwpx";
/// 본문 좌단(px) — 저장 `col_start=0` 이 가리키는 자리.
const BODY_LEFT_PX: f64 = 75.6;
/// 본문 우단(px).
const BODY_RIGHT_PX: f64 = 718.1;

#[test]
fn issue_6167_stored_own_line_tac_table_starts_at_column_left() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let page = core.build_page_render_tree(0).expect("page 1 render tree");

    let (x, w) = first_table(&page.root).expect("자리차지 표");
    let right = x + w;

    assert!(
        (x - BODY_LEFT_PX).abs() <= 2.0,
        "저장 사다리가 표에 자기 줄(col_start=0)을 줬으므로 표는 본문 좌단\
         ({BODY_LEFT_PX})에서 시작해야 한다 — 실측 {x:.1}. \
         앞 공백 18자(120.0px)를 leading 으로 실으면 195.6 이 된다."
    );
    assert!(
        right <= BODY_RIGHT_PX + 1.0,
        "표가 본문 우단({BODY_RIGHT_PX})을 넘어 오른쪽 열이 잘린다 — \
         x={x:.1} w={w:.1} 우변={right:.1}"
    );
}

/// 쪽에서 처음 만나는 표의 `(x, width)`.
fn first_table(node: &RenderNode) -> Option<(f64, f64)> {
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        return Some((node.bbox.x, node.bbox.width));
    }
    node.children.iter().find_map(first_table)
}
