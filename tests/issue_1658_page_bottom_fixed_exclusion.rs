//! [Task #1658 v3] 페이지 하단 고정 표(vert=쪽·valign=Bottom 자리차지, 결재/서명 틀)의
//! "하단 배타 예약" 배치 회귀 가드. #1653 RCA 패턴 B.
//!
//! 한글 실측(stage1): 이런 틀은 본문 하단에 절대배치(다수 시 겹침 허용)되고, 본문
//! 텍스트는 하단 배타 영역(= 최대 틀 높이) 위까지만 흐른다. 수정 전 rhwp 는
//! (a) 틀 높이를 문서순 flow 로 소비해 틀 2개 문서가 over-pagination(+1쪽) 되고
//! (b) 1×1 래퍼 unwrap 이 외곽의 절대 y 를 소실시켜 틀이 본문 상단에 렌더됐다.
//!
//! 반대 방향 가드: `36387725_footer_page_bottom.hwpx`(#1611, 한글 2쪽 — 본문이 배타
//! 영역을 침범하면 틀을 다음 쪽으로 이월)는 `issue_1611_footer_page_bottom_pagination`
//! 이 담당한다. 두 테스트가 함께 배타 예약(과소)과 flow 소비(과대)의 양쪽을 잠근다.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

const GWANAK: &str =
    "samples/hwpx/opengov/36389312_결재문서본문_특정소방대상물 화재발생 알림(화재번호 2026-177).hwpx";
const PC_SHUTDOWN: &str =
    "samples/hwpx/opengov/36398366_결재문서본문_PC 셧다운 제외 및 초과근무 인정 요청(데이터전략과).hwpx";

fn load_doc(sample: &str) -> rhwp::wasm_api::HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(sample);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {sample}: {e}"));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {sample}: {e:?}"))
}

/// 한글 정답 1쪽 — 하단 고정 틀 2개(247px+357px)가 flow 소비되면 2쪽으로 over-pagination.
#[test]
fn gwanak_two_bottom_fixed_frames_fit_one_page() {
    let doc = load_doc(GWANAK);
    assert_eq!(
        doc.page_count(),
        1,
        "하단 고정 틀 2개 문서(한글 1쪽)가 flow 소비로 over-pagination"
    );
}

/// PC 셧다운 계열(디지털도시국, #1653 RCA 원본) — 한글 1쪽.
#[test]
fn pc_shutdown_bottom_fixed_frames_fit_one_page() {
    let doc = load_doc(PC_SHUTDOWN);
    assert_eq!(
        doc.page_count(),
        1,
        "PC 셧다운 계열(한글 1쪽)이 flow 소비로 over-pagination"
    );
}

fn max_text_line_bottom(root: &RenderNode) -> f64 {
    let own = if matches!(root.node_type, RenderNodeType::TextLine(_)) {
        root.bbox.y + root.bbox.height
    } else {
        f64::MIN
    };
    root.children
        .iter()
        .map(max_text_line_bottom)
        .fold(own, f64::max)
}

fn find_body_bbox(root: &RenderNode) -> Option<(f64, f64)> {
    if matches!(root.node_type, RenderNodeType::Body { .. }) {
        return Some((root.bbox.y, root.bbox.y + root.bbox.height));
    }
    root.children.iter().find_map(find_body_bbox)
}

/// 하단 고정 틀의 내용(시행문/전화 등)이 본문 **하단부**(하단 40% 영역)까지 내려와
/// 렌더되는지 — unwrap 절대 y 소실 시 틀 내용이 본문 상단에 그려져 최대 텍스트
/// bottom 이 페이지 상반부에 머문다 (수정 전: p2 상단 59~271pt).
#[test]
fn gwanak_bottom_fixed_frame_renders_at_page_bottom() {
    let doc = load_doc(GWANAK);
    let tree = doc
        .build_page_render_tree(0)
        .unwrap_or_else(|e| panic!("render p1: {e}"));
    let (body_top, body_bottom) = find_body_bbox(&tree.root).expect("body");
    let max_bottom = max_text_line_bottom(&tree.root);
    let threshold = body_top + (body_bottom - body_top) * 0.6;
    assert!(
        max_bottom > threshold,
        "하단 고정 틀 내용이 본문 하단부에 렌더돼야 한다: max_text_bottom={max_bottom:.1} ≤ {threshold:.1} (body {body_top:.1}..{body_bottom:.1})"
    );
    assert!(
        max_bottom <= body_bottom + 0.5,
        "틀 내용이 본문 하단을 넘으면 안 된다: {max_bottom:.1} > {body_bottom:.1}"
    );
}
