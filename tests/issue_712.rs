//! Issue #712: wrap=TopAndBottom 음수 vert offset 12x5 표가 직전 inline TAC 1x3 제목 표
//! 안쪽으로 침범하는 결함 — `samples/2022년 국립국어원 업무계획.hwp` 31 페이지 상단.
//!
//! 결함 메커니즘:
//! - pi=585: 1x3 인라인 TAC 제목 표 ("붙임 / / 과제별 추진일정"), wrap=TopAndBottom
//! - pi=586: 12x5 일정 표, treat_as_char=false, wrap=TopAndBottom, vert=문단(-1796 HU)
//! - pi=586 의 LINE_SEG 가 ls[0].vpos=69196 → ls[1].vpos=0 으로 vpos-reset 패턴
//! - 정상 동작: pi=586 외곽 상단 y ≥ pi=585 외곽 하단 y (≈ 140.87 px)
//! - 결함 동작: pi=586 외곽 상단 y = 124.93 px → pi=585 안쪽으로 ~15.94 px 침범
//!
//! 권위 자료: `pdf/2022년 국립국어원 업무계획-2022.pdf` (한글 2022 편집기 PDF)

use std::fs;
use std::path::Path;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/2022년 국립국어원 업무계획.hwp";
// pi=585 / pi=586 가 등장하는 페이지 인덱스는 빌드의 pagination 결과에 의존:
// - Task #643 미적용 (stream/devel, 본 회귀 테스트 작성 시점): page_index 35
// - Task #643 적용 (PR #644 merge 후): page_index 30
// 페이지 인덱스를 하드코딩하지 않고 pi=585/586 를 가진 페이지를 동적으로 탐색한다.

/// para_index/control_index 일치하는 첫 Table 노드의 bbox 를 (y_top, y_bottom) 으로 반환.
fn find_table_bbox(root: &RenderNode, target_pi: usize, target_ci: usize) -> Option<(f64, f64)> {
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if let RenderNodeType::Table(t) = &node.node_type {
            if t.para_index == Some(target_pi) && t.control_index == Some(target_ci) {
                let bbox = &node.bbox;
                return Some((bbox.y, bbox.y + bbox.height));
            }
        }
        for child in &node.children {
            stack.push(child);
        }
    }
    None
}


#[test]
fn issue_712_pi586_table_does_not_invade_pi585_outer_box() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join(SAMPLE);
    let bytes = fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e));

    let page_count = doc.page_count();

    // pi=585 (1x3 TAC 제목 표) 와 pi=586 (12x5 일정 표 첫 분할) 이 동시에 등장하는 페이지 탐색
    let mut target_page: Option<u32> = None;
    let mut target_tree = None;
    for pn in 0..page_count {
        let t = doc.build_page_render_tree(pn).expect("build_page_render_tree");
        let p585 = find_table_bbox(&t.root, 585, 0);
        let p586 = find_table_bbox(&t.root, 586, 0);
        if let (Some(_), Some(_)) = (p585, p586) {
            target_page = Some(pn);
            target_tree = Some(t);
            break;
        }
    }
    let target_page = target_page.unwrap_or_else(||
        panic!("pi=585 + pi=586 동시 등장 페이지를 찾지 못함 (page_count={})", page_count)
    );
    let tree = target_tree.unwrap();

    let (pi585_top, pi585_bottom) = find_table_bbox(&tree.root, 585, 0)
        .expect("pi=585 ci=0 (1x3 TAC 제목 표) Table 노드 누락");
    let (pi586_top, pi586_bottom) = find_table_bbox(&tree.root, 586, 0)
        .expect("pi=586 ci=0 (12x5 일정 표) Table 노드 누락");

    eprintln!(
        "[issue_712] page_index={} (page_count={}) pi585=[{:.2}..{:.2}] pi586=[{:.2}..{:.2}]",
        target_page, page_count, pi585_top, pi585_bottom, pi586_top, pi586_bottom,
    );

    // pi=586 외곽 상단이 pi=585 외곽 하단 아래(=같거나 더 아래)에 있어야 침범 0.
    // 0.5 px 허용 오차 (rounding/sub-pixel 정합).
    assert!(
        pi586_top >= pi585_bottom - 0.5,
        "pi=586 12x5 표가 pi=585 1x3 표 안쪽으로 침범. \
         pi585=[{:.2}..{:.2}] pi586=[{:.2}..{:.2}] 침범={:.2} px",
        pi585_top, pi585_bottom, pi586_top, pi586_bottom,
        pi585_bottom - pi586_top,
    );
}
