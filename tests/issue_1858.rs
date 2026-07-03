//! [Task #1858] 페이지-절대 앵커(vert=용지/Paper) 개체 flow 소비로 인한 페이지 폭발 회귀 게이트.
//!
//! `vert=용지`(Paper) 자리차지(TopAndBottom) 표는 용지 절대좌표에 그려지며 flow 를 소비하지
//! 않아야 한다. 그러나 절대배치 가드(typeset.rs `is_paper_topbottom_block`)가 host 문단 vpos
//! 기준 `target_y > current_height` 로만 동기화하는데, 같은 host 문단에 co-anchored 된 Paper
//! 상자들은 target_y 가 모두 동일(host vpos)해 첫 상자만 통과하고 나머지는 flow 경로로 빠져
//! 각자 높이를 소비, 페이지가 폭발했다.
//!
//! fixture: samples/issue1858_paper_anchor_float_stack.hwpx
//!   = 실문서 3143097 (과태료 납부 독촉, 고용노동부). pi=2 문단에 `vert=용지` 서식 상자 22개
//!     (ci=14..35, 독촉장 필드 상자)가 co-anchored. 모두 1쪽 용지좌표 안에 있으므로 한컴은 1쪽.
//!   수정 전: 첫 상자만 절대배치, 나머지 21개가 flow 소비 → 4쪽(#1853 후 3쪽).
//!   수정 후: 후속 co-anchored Paper 상자도 절대배치(0 flow) → 1쪽.

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue1858_paper_anchor_float_stack.hwpx";

fn load_doc(sample: &str) -> rhwp::wasm_api::HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join(sample);
    let bytes = fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", sample, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", sample, e))
}

/// vert=용지 co-anchored 상자 22개가 모두 1쪽 용지좌표에 있으므로 문서는 1쪽이어야 한다.
/// 후속 상자가 flow 를 소비해 페이지가 늘면(수정 전 4쪽) 회귀다.
#[test]
fn paper_anchored_coanchored_boxes_do_not_inflate_pages() {
    let doc = load_doc(SAMPLE);
    assert_eq!(
        doc.page_count(),
        1,
        "vert=용지 co-anchored 자리차지 상자는 flow 를 소비하지 않고 1쪽 용지좌표에 절대배치되어야 \
         한다; 후속 상자가 flow 를 소비해 페이지가 늘면 회귀(수정 전 4쪽)",
    );
}
