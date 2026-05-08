//! Issue #713: RowBreak 표가 인트라-로우 분할되어 PDF 권위 자료와 시각 불일치.
//!
//! 결함 표: `samples/2022년 국립국어원 업무계획.hwp` 의 12x5 일정 표 (`pi=586 ci=0`),
//! `쪽나눔=RowBreak` (행 경계에서만 분할 가능, 인트라-로우 분할 금지).
//!
//! 결함 메커니즘: `src/renderer/typeset.rs` 의 인트라-로우 분할 분기가 page_break
//! 모드를 점검하지 않아 RowBreak 표도 인트라-로우 분할 적용. 행 8 (`한국어교육 내실화`
//! + `ㅇ국외 한국어교육 지원 사업 수요조사...`) 이 페이지 경계에서 17.6 px 분할.
//!
//! 정상 동작: 행 8 전체가 다음 페이지 상단에 위치, `clip: false` (분할 표시 없음).

use std::fs;
use std::path::Path;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/2022년 국립국어원 업무계획.hwp";
const TARGET_PI: usize = 586;
const TARGET_CI: usize = 0;
const TARGET_ROW: u16 = 8;

/// 트리에서 (target_pi, target_ci) 표의 자식 셀 중 row==target_row 인 셀을 모두 수집.
fn collect_row_cells<'a>(
    root: &'a RenderNode,
    target_pi: usize,
    target_ci: usize,
    target_row: u16,
    out: &mut Vec<&'a RenderNode>,
) {
    if let RenderNodeType::Table(t) = &root.node_type {
        if t.para_index == Some(target_pi) && t.control_index == Some(target_ci) {
            // 이 표의 자식 셀 중 row==target_row 추출
            for child in &root.children {
                collect_target_row_cells(child, target_row, out);
            }
            return;
        }
    }
    for child in &root.children {
        collect_row_cells(child, target_pi, target_ci, target_row, out);
    }
}

fn collect_target_row_cells<'a>(
    node: &'a RenderNode,
    target_row: u16,
    out: &mut Vec<&'a RenderNode>,
) {
    if let RenderNodeType::TableCell(tc) = &node.node_type {
        if tc.row == target_row {
            out.push(node);
        }
        return; // TableCell 안쪽은 row 정보 없음
    }
    for child in &node.children {
        collect_target_row_cells(child, target_row, out);
    }
}

#[test]
fn issue_713_rowbreak_table_no_intra_row_split() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join(SAMPLE);
    let bytes = fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e));

    let page_count = doc.page_count();

    // 모든 페이지에서 pi=586 ci=0 표의 row 8 셀을 수집.
    // RowBreak 모드라면 행 8 의 모든 셀이 단일 페이지에 위치하고 clip=false 여야 함.
    let mut all_row8_cells: Vec<(u32, bool)> = Vec::new(); // (page_index, clip)
    for pn in 0..page_count {
        let tree = doc.build_page_render_tree(pn).expect("build_page_render_tree");
        let mut cells = Vec::new();
        collect_row_cells(&tree.root, TARGET_PI, TARGET_CI, TARGET_ROW, &mut cells);
        for cell in cells {
            if let RenderNodeType::TableCell(tc) = &cell.node_type {
                all_row8_cells.push((pn, tc.clip));
            }
        }
    }

    eprintln!(
        "[issue_713] page_count={} row {} cells found across {} (page, clip) entries",
        page_count, TARGET_ROW, all_row8_cells.len(),
    );
    let split_pages: std::collections::BTreeSet<u32> =
        all_row8_cells.iter().map(|(p, _)| *p).collect();
    let with_clip: Vec<&(u32, bool)> = all_row8_cells.iter().filter(|(_, c)| *c).collect();
    eprintln!(
        "[issue_713] row {} cells appear on pages={:?} clipped_cells={}",
        TARGET_ROW, split_pages, with_clip.len(),
    );

    // 단언 1: row 8 셀들이 단일 페이지에만 등장 (RowBreak 명세상 행은 분할 불가)
    assert!(
        split_pages.len() == 1,
        "RowBreak 표 행 {} 가 {} 페이지에 분할 등장: pages={:?}",
        TARGET_ROW, split_pages.len(), split_pages,
    );

    // 단언 2: row 8 셀의 clip 플래그가 모두 false (분할 클리핑 없음)
    assert!(
        with_clip.is_empty(),
        "RowBreak 표 행 {} 의 셀 {} 개가 clip=true (인트라-로우 분할 검출)",
        TARGET_ROW, with_clip.len(),
    );
}
