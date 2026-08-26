//! [Issue #6128] 어울림(Square) 표 옆에서 2줄로 접힌 문단의 흐름이 1줄만 전진해
//! 다음 문단이 그 위에 겹쳐 그려진다 (156653004 4쪽).
//!
//! 근인: 어울림 문단들은 **저장 vpos** 로 배치되는데(`layout_wrap_around_paras`),
//! 표 뒤 흐름 커서는 표 바닥(과 host 문단 바닥)에서 멈춘다. host 문단만 보는
//! #1218 규칙으로는 **뒤따르는 어울림 문단**이 표보다 아래로 내려온 몫을
//! 회수하지 못해, 다음 일반 문단이 그 줄 위에 겹쳤다.
//!
//! 조판(typeset)은 같은 계약을 `extend_square_band_to_source_bottom` 으로 이미
//! 갖고 있다 — 페인트도 같은 저장 좌표로 커서를 끌어올린다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6128/156653004_privacy_day_ceremony.hwpx";
/// 결함이 나타나는 쪽(0-based).
const PAGE: u32 = 3;
/// 어울림 표 옆에서 접힌 문단의 둘째 줄.
const WRAPPED_LINE: &str = "산·학·관 관계자";
/// 그 아래로 와야 하는 다음 문단.
const FOLLOWING_LINE: &str = "대통령실";

#[test]
fn issue_6128_following_paragraph_clears_the_wrapped_line() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    assert_eq!(core.page_count(), 7, "한글 오라클과 같은 7쪽이어야 한다");

    let page = core
        .build_page_render_tree(PAGE)
        .expect("page 4 render tree");
    let wrapped = line_bounds(&page.root, WRAPPED_LINE).expect("접힌 둘째 줄");
    let following = line_bounds(&page.root, FOLLOWING_LINE).expect("다음 문단 줄");

    assert!(
        following.0 >= wrapped.1 - 0.5,
        "다음 문단이 접힌 줄 위에 겹쳤다: 다음 줄 위끝={:.1}, 접힌 줄 아래끝={:.1}",
        following.0,
        wrapped.1
    );
}

/// `needle` 을 포함한 첫 텍스트 줄의 (위끝, 아래끝).
fn line_bounds(node: &RenderNode, needle: &str) -> Option<(f64, f64)> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains(needle) {
            return Some((node.bbox.y, node.bbox.y + node.bbox.height));
        }
    }
    node.children
        .iter()
        .find_map(|child| line_bounds(child, needle))
}
