//! [#5809 실측 ①] 어울림(Square) 묶음 개체를 소유한 빈 문단의 세로 자리가
//! 렌더러에서 유실돼 이후 본문 전체가 22.6px 위로 밀리던 회귀 가드.
//!
//! 156518601(모바일 운전면허증 보도자료) 1쪽 실측 — 수정 전: 빈 Square host
//! 문단(pi=7, 저장 h=33.9 = sb 4.0 + 줄 29.9)의 줄을 렌더러 textless 분기가
//! 버려 `경찰청(청장…` 줄 상단이 597.0(저장 정답 619.7 대비 −22.7)이었고,
//! 어긋남이 쪽 끝까지 전파됐다. typeset 은 저장 사다리 스냅으로 정답을 내
//! desync 상태였다. 수정: Square 계열 호스트도 비합성 저장 사다리에 예약
//! 여부를 묻고, 예약 증언 시 저장 델타(sb+lh+ls 전량)로 전진한다.
//!
//! 재현물은 원본(11.8MB)에서 1쪽 문단 14개만 남기고 그림을 1px 더미로 바꾼
//! 축소본(21KB) — 결함 좌표(619.7)가 그대로 재현됨을 확인하고 동봉했다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue5809/156518601_p1_square_host.hwpx";

fn find_line_y(node: &RenderNode, prefix: &str) -> Option<f64> {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        let text: String = node
            .children
            .iter()
            .filter_map(|c| match &c.node_type {
                RenderNodeType::TextRun(run) => Some(run.text.as_str()),
                _ => None,
            })
            .collect();
        if text.trim_start().starts_with(prefix) {
            return Some(node.bbox.y);
        }
    }
    node.children
        .iter()
        .find_map(|child| find_line_y(child, prefix))
}

#[test]
fn issue_5809_square_host_empty_para_keeps_stored_ladder_charge() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(path).expect("read #5809 fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse #5809 fixture");

    assert_eq!(core.page_count(), 1, "축소 재현물은 1쪽이다");

    let page = core.build_page_render_tree(0).expect("render p1");
    let y = find_line_y(&page.root, "경찰청(청장").expect("경찰청 문단 첫 줄");
    // 저장 사다리 정답: 본문 상단 94.5 + vpos 39389HU(525.2) = 619.7.
    // 수정 전엔 빈 Square host 문단의 자리 유실로 597.0 이었다.
    assert!(
        (y - 619.7).abs() <= 2.0,
        "`경찰청(청장…` 줄 상단은 저장 사다리 정답 619.7 이어야 함 (수정 전 597.0); got {y:.1}"
    );
}
