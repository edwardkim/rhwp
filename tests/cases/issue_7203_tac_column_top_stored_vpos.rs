//! [#7203] 단 맨 위 어울림(TAC) 표가 호스트의 저장 첫 줄 `vertical_pos` 를 버리지 않는다.
//!
//! # 무엇이 깨져 있었나
//!
//! `LINE_SEG.vertical_pos` 는 쪽(단) 상단 기준 절대값이다. 호스트 문단이 단 맨 위에
//! 오면 흐름 커서가 곧 단 상단이므로 저장값이 그대로 위 여백이 된다. 본문 문단은
//! `paragraph_layout` 의 column-top 계약(Task #1811, `vpos ≤ spacing_before` 면 가산)이
//! 이 값을 싣는데, **빈 앵커 문단의 TAC 표**는 `PageItem::FullParagraph` 가 발행되지
//! 않아 그 블록을 아예 타지 않는다. 그래서 표가 단 상단 + 바깥 위여백에 붙었다.
//!
//! # 독립 기대값 — 한/글 정본이 두 갈래를 갈라 준다
//!
//! `pdf/hwpctl_API_v2.4-hwp-2020.pdf` 에서 폭 427.9px 가로 괘선을 표 높이로 위·아래변을
//! 짝지어 잠그고, 단 맨 위(`y≈136`)에 놓인 표 15건을 전수로 보면 어긋남이 저장 `vpos`
//! 한 값에 걸려 있다.
//!
//! ```text
//!   저장 vpos    tac     n   정본 윗변   수정 전 rhwp   dy
//!   500 HU      true    8    142.56      136.00      +6.56   ← 전부 같은 값
//!   0           true    1    136.01      136.00      +0.01
//!   사다리 절대  false   6    136.01      136.00      +0.01
//! ```
//!
//! `500 HU = 6.67px` 이고 어긋남이 `+6.56px` 다. 수정 뒤 16쪽 표는 `142.70`(정본과
//! 0.14px), 반례인 90쪽 `vpos=0` 표는 `136.00` 으로 그대로다.
//!
//! 이 검사는 두 갈래를 함께 잠근다 — 저장값이 있으면 싣고, 없으면 종전대로 둔다.
//! 같은 문서의 쪽-내 드리프트(#4599 축)까지 해결했다고 주장하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const FIXTURE: &str = "samples/hwpctl_API_v2.4.hwp";
/// 코드 상자 표의 폭(32079 HU = 427.7px).
const CODE_BOX_WIDTH_PX: f64 = 427.7;
/// 한/글 2020 정본 16쪽 — 저장 `vpos=500HU` 인 단 맨 위 표의 윗변.
const ORACLE_STORED_VPOS_TOP: f64 = 142.56;
/// 수정 전 관측값 — 저장 `vpos` 를 버리고 단 상단 + 바깥 위여백에 붙었다.
const BEFORE_FIX_STORED_VPOS_TOP: f64 = 136.00;
/// 한/글 2020 정본 90쪽 — 저장 `vpos=0` 인 단 맨 위 표(반례)의 윗변.
const ORACLE_ZERO_VPOS_TOP: f64 = 136.01;

fn core() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    DocumentCore::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드")
}

/// 이 쪽에서 가장 위에 있는 코드 상자 표의 윗변.
fn top_code_box_top(root: &RenderNode) -> f64 {
    fn walk(node: &RenderNode, out: &mut Vec<f64>) {
        if matches!(node.node_type, RenderNodeType::Table(_))
            && (node.bbox.width - CODE_BOX_WIDTH_PX).abs() < 1.5
        {
            out.push(node.bbox.y);
        }
        for child in &node.children {
            walk(child, out);
        }
    }
    let mut tops = Vec::new();
    walk(root, &mut tops);
    tops.into_iter()
        .min_by(|a, b| a.partial_cmp(b).expect("유한값"))
        .expect("코드 상자 표")
}

fn page_top(core: &DocumentCore, page_index: u32) -> f64 {
    let tree = core
        .build_page_render_tree(page_index)
        .expect("쪽 렌더 트리");
    top_code_box_top(&tree.root)
}

/// 저장 `vpos` 가 있으면 단 맨 위 TAC 표는 그만큼 아래다.
#[test]
fn column_top_tac_table_keeps_stored_vertical_pos() {
    let core = core();
    assert_eq!(core.page_count(), 105, "정답지와 같은 105쪽");
    let top = page_top(&core, 15);
    assert!(
        (top - ORACLE_STORED_VPOS_TOP).abs() < 1.0,
        "16쪽 단 맨 위 표 윗변은 정본 {ORACLE_STORED_VPOS_TOP} 근처여야 한다\
         (수정 전 {BEFORE_FIX_STORED_VPOS_TOP}) — got {top:.2}"
    );
}

/// 반례 — 저장 `vpos=0` 인 단 맨 위 TAC 표는 종전 자리 그대로다.
#[test]
fn column_top_tac_table_without_stored_vertical_pos_does_not_move() {
    let core = core();
    let top = page_top(&core, 89);
    assert!(
        (top - ORACLE_ZERO_VPOS_TOP).abs() < 1.0,
        "90쪽 저장 vpos=0 표는 정본 {ORACLE_ZERO_VPOS_TOP} 에 그대로 있어야 한다 — got {top:.2}"
    );
}
