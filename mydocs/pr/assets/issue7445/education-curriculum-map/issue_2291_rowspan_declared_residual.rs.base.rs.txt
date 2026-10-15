//! Issue #2291/#2237 (PR #2309) — 거대 rowspan 표 정밀도 회귀 oracle.
//!
//! `samples/task2287/1342000_edu_curriculum_map.hwp` s5 pi8 (244×10, 다층
//! rowspan: c0 rs=67 / c1 rs=28 / c2 rs=9 / c3 rs=3~4) 기준. 정답지:
//! `pdf/task2287/1342000_edu_curriculum_map-2022.pdf` (한글 2022 COM, 415쪽).
//!
//! 고정하는 두 규칙 (행 괘선 실측 기반, #2291 코멘트):
//! 1. **rowspan 병합 셀 선언-잔여의 마지막 걸침 행 가산** — r183: c3(rs=4)
//!    선언 217.8px vs 걸친 행합 201.3px → 한글 행 괘선 실측 r183 =
//!    39.8+16.5 = 56.3px. 회귀(잔여 소실) 시 39.8 로 복귀.
//! 2. **저장-ls-1 폭 초과 문단의 재래핑** — r183 c8 76자 문단(저장 lineseg
//!    1개)이 재래핑 없이 1줄 렌더되면 "…실천 계획 세"에서 절단된다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn core() -> DocumentCore {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join("samples/task2287/1342000_edu_curriculum_map.hwp");
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    DocumentCore::from_bytes(&bytes).expect("parse 1342000_edu_curriculum_map.hwp")
}

/// (row, col) 셀 노드의 bbox 높이. 여러 쪽에 걸친 문서라 페이지 범위를 훑는다.
fn find_cell_height(
    core: &DocumentCore,
    pages: std::ops::Range<u32>,
    row: u32,
    col: u32,
) -> Option<f64> {
    for page in pages {
        let tree = core.build_page_render_tree(page).ok()?;
        if let Some(h) = find_cell_in(&tree.root, row, col) {
            return Some(h);
        }
    }
    None
}

fn find_cell_in(node: &RenderNode, row: u32, col: u32) -> Option<f64> {
    if let RenderNodeType::TableCell(c) = &node.node_type {
        if u32::from(c.row) == row && u32::from(c.col) == col {
            return Some(node.bbox.height);
        }
    }
    node.children.iter().find_map(|c| find_cell_in(c, row, col))
}

fn find_text(node: &RenderNode, needle: &str) -> bool {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains(needle) {
            return true;
        }
    }
    node.children.iter().any(|c| find_text(c, needle))
}

#[test]
fn issue_2291_rowspan_declared_residual_expands_last_spanned_row() {
    let core = core();
    let h = find_cell_height(&core, 130..140, 183, 8)
        .expect("s5 pi8 r183 c8 셀이 p131~140 범위에 렌더돼야 함");
    // 한글 2022 행 괘선 실측 56.3px (= 선언 39.8 + c3 rs=4 잔여 16.5).
    assert!(
        (55.0..=58.0).contains(&h),
        "r183 셀 높이 {h:.1}px — rowspan 선언-잔여 가산 회귀 (한글 56.3, 소실 시 39.8)"
    );
}

#[test]
fn issue_2291_stored_single_seg_paragraph_rewraps_full_text() {
    let core = core();
    let mut found = false;
    for page in 130..140u32 {
        if let Ok(tree) = core.build_page_render_tree(page) {
            if find_text(&tree.root, "실천 계획 세우기") {
                found = true;
                break;
            }
        }
    }
    assert!(
        found,
        "r183 c8 문단 꼬리('…실천 계획 세우기') 부재 — 저장-ls-1 재래핑 회귀 (절단 '…실천 계획 세')"
    );
}
