//! 이어받는 표 조각의 컷 선택·예약·그리기가 같은 시작 유닛을 쓴다.
//!
//! # 무엇이 깨져 있었나
//!
//! 행 컷 선택기(`advance_row_cut_inner`)는 이어받는 조각이 빈 spacer 유닛으로 시작하면
//! 그 유닛을 **높이 없이** 소비한다. 그런데 조각 상자를 재는 `row_cut_content_height` 와
//! 그리기는 넘겨받은 시작 컷부터 유닛 높이를 그대로 실었다. 측정과 배치가 다른 높이를
//! 소비하므로 컷은 예산 안인데 조각은 본문을 넘었다.
//!
//! `samples/task2430/1382000_domestic_violence_survey.hwp` 의 1×1 RowBreak 표(`pi=93`)는
//! 앞 조각이 빈 문단 `p77`(유닛 117) 앞에서 끝나, 24쪽(0-based 23) 조각이 그 빈 문단으로
//! 시작했다.
//!
//! ```text
//!   컷 선택   유닛 117..148  h 900.0(p77 0) − 되감김 앞 줄간격 14.7 = 885.3  (예산 886.9)
//!   조각 상자 유닛 117..148  합 929.3(p77 29.3) − 14.7 + 안 여백 3.8 = 918.4  (본문 890.7)
//! ```
//!
//! 재시도도 같은 선택기를 쓰므로 수렴하지 않고, 조각 끝 글줄이 본문 바닥(1009.2)을
//! 넘어 꼬리말 쪽번호(1029.1)와 2.7px 겹쳤다.
//!
//! # 고친 것
//!
//! 앞 조각이 다음 조각에 넘기는 시작 컷을 선택기와 같은 술어
//! (`is_free_leading_spacer`)로 그 spacer 뒤로 옮긴다
//! (`skip_leading_free_spacers_in_row_cut`). 컷 위치는 그대로이고, 다음 조각의 선택·예약·
//! 그리기가 같은 유닛에서 시작한다.
//!
//! # 독립 기대값
//!
//! - 본문 영역은 `body_area` 가 정한다. 조각 상자와 그 안의 글줄은 본문 바닥 안에 있어야
//!   한다 — 저장 정보와 무관한 쪽 불변식이다.
//! - 저장 사다리에서 `p77`(vpos 64462)은 앞 쪽 프레임에 있고 다음 쪽은 `p78`(vpos 0)에서
//!   다시 시작한다. 따라서 24쪽 조각의 첫 글자 줄은 칸 위 안 여백 바로 아래에 놓인다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/task2430/1382000_domestic_violence_survey.hwp";
const PARA: usize = 93;
/// 빈 문단으로 시작하던 조각의 쪽(0-based).
const LEADING_SPACER_PAGE: u32 = 23;

fn load() -> DocumentCore {
    let full = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&full).unwrap_or_else(|e| panic!("재현체 {}: {e}", full.display()));
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

fn body_bottom(node: &RenderNode) -> Option<f64> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node.bbox.y + node.bbox.height);
    }
    node.children.iter().find_map(body_bottom)
}

fn has_text(node: &RenderNode) -> bool {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if !run.display_or_text().trim().is_empty() {
            return true;
        }
    }
    node.children.iter().any(has_text)
}

/// 표 조각 `(상자 위, 상자 바닥, 글자 있는 첫 글줄 위, 글자 있는 글줄의 가장 낮은 바닥)`.
fn fragments(node: &RenderNode, out: &mut Vec<(f64, f64, f64, f64)>) {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(PARA) && table.cell_context.is_none() {
            fn lines(node: &RenderNode, top: &mut f64, bottom: &mut f64) {
                if matches!(node.node_type, RenderNodeType::TextLine(_)) && has_text(node) {
                    *top = top.min(node.bbox.y);
                    *bottom = bottom.max(node.bbox.y + node.bbox.height);
                }
                for child in &node.children {
                    lines(child, top, bottom);
                }
            }
            let (mut top, mut bottom) = (f64::MAX, f64::MIN);
            lines(node, &mut top, &mut bottom);
            out.push((node.bbox.y, node.bbox.y + node.bbox.height, top, bottom));
            return;
        }
    }
    for child in &node.children {
        fragments(child, out);
    }
}

/// `pi=93` 의 모든 조각이 본문 안에 있다. 수정 전 24쪽 조각은 상자·끝 글줄 바닥 1031.8 로
/// 본문 바닥 1009.1 을 넘었다.
#[test]
fn leading_spacer_fragment_stays_inside_the_body() {
    let core = load();
    let mut checked = 0;
    let mut over = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("렌더 트리");
        let Some(bottom) = body_bottom(&tree.root) else {
            continue;
        };
        let mut found = Vec::new();
        fragments(&tree.root, &mut found);
        for (_, box_bottom, _, line_bottom) in found {
            checked += 1;
            if box_bottom > bottom + 0.5 || line_bottom > bottom + 0.5 {
                over.push(format!(
                    "{page}쪽 상자 {box_bottom:.1} 글줄 {line_bottom:.1} > 본문 {bottom:.1}"
                ));
            }
        }
    }
    // 이 표는 여섯 쪽에 걸친다. 조각을 못 찾으면 검사가 빈 것이다.
    assert!(checked >= 6, "pi={PARA} 조각을 {checked}개만 찾았습니다");
    assert!(over.is_empty(), "본문을 넘는 조각: {over:?}");
}

/// 24쪽 조각은 저장 사다리처럼 `p78` 로 시작한다 — 첫 글자 줄이 상자 위 안 여백 바로
/// 아래다. 수정 전에는 그리지 않는 빈 문단 29.3px 아래(상자 위 + 33.1)에서 시작했다.
#[test]
fn continuation_starts_where_the_stored_ladder_restarts() {
    let core = load();
    let tree = core
        .build_page_render_tree(LEADING_SPACER_PAGE)
        .expect("렌더 트리");
    let mut found = Vec::new();
    fragments(&tree.root, &mut found);
    let [(box_top, _, first_line, _)] = found.as_slice() else {
        panic!("{LEADING_SPACER_PAGE}쪽 pi={PARA} 조각이 하나가 아닙니다: {found:?}");
    };
    // 칸 위 안 여백(141HU = 1.9px)과 줄 상자 위 여백 안이다. 빈 줄 하나(29.3px)보다 작다.
    assert!(
        first_line - box_top < 5.0,
        "첫 글자 줄 {first_line:.1} 이 상자 위 {box_top:.1} 에서 {:.1}px 떨어져 있습니다",
        first_line - box_top
    );
}

/// 컷 위치는 바뀌지 않는다 — 쪽수와 이 표의 조각 수가 그대로다.
#[test]
fn cut_positions_are_unchanged() {
    let core = load();
    assert_eq!(core.page_count(), 39);
    let mut count = 0;
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("렌더 트리");
        let mut found = Vec::new();
        fragments(&tree.root, &mut found);
        count += found.len();
    }
    assert_eq!(count, 6, "pi={PARA} 조각 수");
}
