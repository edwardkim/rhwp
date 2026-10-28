//! [Issue #6132] 저장 vpos 가 쪽 본문을 넘는 문단을 현재 쪽 잔여에 욱여넣어
//! '참고3' 제목 상자와 그 아래 ※ 두 줄이 7쪽 하단에 붙는다 (156482639).
//!
//! 저장 사다리가 두 신호를 함께 준다.
//!
//! ```text
//! pi=101  vpos=71584
//! pi=102  vpos=73760   ← 본문 73,335HU(977.8px) 초과 = 이 쪽에 있을 수 없다
//! pi=103  vpos=3790    ← 되감김 = 다음 쪽 첫머리
//! ```
//!
//! 한글 2020 은 pi=102 부터 8쪽에 두고 7쪽은 표(연번 11)로 끝낸다. rhwp 는 두
//! 신호를 모두 무시하고 pi=102..104 를 7쪽 잔여(974.5px)에 채워, 8쪽이 그만큼
//! (56.8px) 비었다.
//!
//! 기존 #3837 되감김 규칙은 되감긴 **다음** 문단(pi=103)에만 걸리고, 그 문단이
//! 잔여에 들어가면 분할 루프가 현재 쪽에 그대로 놓는다. 넘긴 주체인 pi=102 는
//! 표 문단이라 그 판정을 아예 지나지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6132/156482639_startup_ir_contest.hwp";
/// '참고3' 상자가 있어야 할 쪽(0-based) — 한글 쪽번호 `- 8 -`.
const EXPECTED_PAGE: u32 = 7;
/// 결함 상태에서 상자가 붙어 있던 쪽(0-based).
const DEFECT_PAGE: u32 = 6;
const NEEDLE: &str = "재인알앤피";

#[test]
fn issue_6132_stored_vpos_overflow_starts_next_page() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    // 한글 2020 오라클도 10쪽 — 이 수정은 쪽 수를 바꾸지 않는다.
    assert_eq!(core.page_count(), 10, "한글 오라클과 같은 10쪽이어야 한다");

    // 7쪽에는 '참고3 ㈜재인알앤피 개요' 가 없어야 한다. 한글 7쪽은 표(연번 11)로
    // 끝나고 마지막 텍스트가 y1≈847.8px 이다.
    let defect_page = core
        .build_page_render_tree(DEFECT_PAGE)
        .expect("7쪽 render tree");
    assert!(
        !page_contains(&defect_page.root, NEEDLE),
        "7쪽 하단에 '참고3' 상자가 남아 있다 — 저장 vpos 초과 신호를 무시한 상태다"
    );

    // 8쪽 첫머리에 있어야 한다. 한글 PDF 실측 y0=62.9pt(≈83.9px).
    let expected_page = core
        .build_page_render_tree(EXPECTED_PAGE)
        .expect("8쪽 render tree");
    assert!(
        page_contains(&expected_page.root, NEEDLE),
        "8쪽에 '참고3 ㈜재인알앤피 개요' 가 있어야 한다"
    );
}

fn page_contains(node: &RenderNode, needle: &str) -> bool {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains(needle) {
            return true;
        }
    }
    node.children
        .iter()
        .any(|child| page_contains(child, needle))
}
