//! 검사 유형: 관계형·고정형.
//! 주석 갱신: 2026-11-06T16:21:00+09:00.
//! 2026-11-06 Visual Sweep: 96dpi·PDFium·색상 경계 보정 전 2px raw 실루엣, MCP engine 2020 Print PDF와 대조.
//! rental_approval_form: 전 1쪽 Native 100.00% / fresh WASM 99.91%; 전쪽 최저 100.00% / 99.91%.
//! artifact_loan_form: 전 1쪽 Native 96.91% / fresh WASM 100.00%; 전쪽 최저 96.91% / 100.00%.
//! ai_procurement_appendix: 3쪽 Native 97.29% / fresh WASM 97.41%; 전 4쪽 최저 97.29% / 97.41%.
//! 근거: mydocs/pr/assets/issue_8682/README.md.
//!
//! 행 선언(cellSz) 합이 표 선언(`hp:sz`)을 넘는 비-TAC 표에서 한/글은 앞 행을 저장값대로 두고
//! 마지막 행의 빈 공간에서만 초과분을 거둔다. 전 행을 비례 축소하면 행 경계가 아래로 갈수록
//! 위로 어긋나도 표 전체 높이는 같아서 쪽수로는 드러나지 않는다.
//!
//! 수정 전 `artifact_loan_form` 의 7행 표는 선언 66.1·76.1·76.1·62.9·55.3·85.5·372.8px 을
//! 65.1·74.9·74.9·61.9·54.5·84.2·367.0px 로 줄였다. 한/글 괘선은 앞 6행이 선언과 같고
//! 마지막 행만 360.5px 이다.
//!
//! 마지막 행이 중첩 표를 담아 회수를 입증하지 못하는 표는 종전 비례 축소를 유지한다
//! (`ai_procurement_appendix` 3쪽 pi=12).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::table::Table;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const HU_PER_PX: f64 = 7200.0 / 96.0;

fn load(path: &str) -> DocumentCore {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("재현체 {path}: {e}"));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("로드 {path}: {e}"))
}

/// 행별 저장 선언(px): `row_span == 1` 칸의 최댓값.
fn declared_rows(table: &Table) -> Vec<f64> {
    let mut rows = vec![0.0f64; table.row_count as usize];
    for cell in table.cells.iter().filter(|cell| cell.row_span == 1) {
        let height = f64::from(cell.height) / HU_PER_PX;
        rows[cell.row as usize] = rows[cell.row as usize].max(height);
    }
    rows
}

fn table_at(core: &DocumentCore, para: usize) -> &Table {
    core.document().sections[0].paragraphs[para]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) => Some(table.as_ref()),
            _ => None,
        })
        .expect("표")
}

/// 선언 행 합이 표 선언을 넘는 첫 표.
fn over_declared_para(core: &DocumentCore) -> usize {
    core.document().sections[0]
        .paragraphs
        .iter()
        .position(|paragraph| {
            paragraph.controls.iter().any(|control| match control {
                Control::Table(table) => {
                    declared_rows(table).iter().sum::<f64>()
                        > f64::from(table.common.height) / HU_PER_PX + 1.0
                }
                _ => false,
            })
        })
        .expect("선언 모순 표")
}

fn collect_table<'a>(node: &'a RenderNode, para: usize, out: &mut Option<&'a RenderNode>) {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(para) && out.is_none() {
            *out = Some(node);
        }
    }
    for child in &node.children {
        collect_table(child, para, out);
    }
}

/// 그려진 행 높이: `row_span == 1` 칸 상자의 높이.
fn rendered_rows(core: &DocumentCore, page: u32, para: usize, row_count: usize) -> Vec<f64> {
    let tree = core.build_page_render_tree(page).expect("렌더 트리");
    let mut table = None;
    collect_table(&tree.root, para, &mut table);
    let mut rows = vec![0.0f64; row_count];
    for child in &table.expect("그려진 표").children {
        if let RenderNodeType::TableCell(cell) = &child.node_type {
            if cell.row_span == 1 {
                rows[cell.row as usize] = child.bbox.height;
            }
        }
    }
    rows
}

fn lines_below_cell(node: &RenderNode, bottom: Option<f64>, out: &mut usize) {
    let mut limit = bottom;
    if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        limit = Some(node.bbox.y + node.bbox.height);
    }
    if matches!(node.node_type, RenderNodeType::TextLine(_))
        && limit.is_some_and(|limit| node.bbox.y + node.bbox.height > limit + 0.5)
    {
        *out += 1;
    }
    for child in &node.children {
        lines_below_cell(child, limit, out);
    }
}

fn assert_tail_absorbs_excess(path: &str) {
    let core = load(path);
    assert_eq!(core.page_count(), 1, "{path}: 쪽수");
    let para = over_declared_para(&core);
    let table = table_at(&core, para);
    let declared = declared_rows(table);
    let rows = rendered_rows(&core, 0, para, declared.len());
    let last = declared.len() - 1;
    for row in 0..last {
        assert!(
            (rows[row] - declared[row]).abs() <= 0.5,
            "{path}: {row}행은 저장 선언 그대로여야 한다 — 그림 {:.2}, 선언 {:.2}",
            rows[row],
            declared[row]
        );
    }
    let target = f64::from(table.common.height) / HU_PER_PX;
    assert!(
        (rows.iter().sum::<f64>() - target).abs() <= 0.5,
        "{path}: 표는 선언 높이를 지킨다"
    );
    assert!(
        rows[last] < declared[last] - 1.0,
        "{path}: 초과분은 마지막 행이 거둔다"
    );
    let tree = core.build_page_render_tree(0).expect("렌더 트리");
    let mut outside = 0;
    lines_below_cell(&tree.root, None, &mut outside);
    assert_eq!(outside, 0, "{path}: 회수가 글줄을 칸 밖으로 밀면 안 된다");
}

#[test]
fn issue_8682_tail_row_absorbs_rental_approval_form() {
    assert_tail_absorbs_excess("samples/issue8682/rental_approval_form.hwp");
}

#[test]
fn issue_8682_tail_row_absorbs_artifact_loan_form() {
    assert_tail_absorbs_excess("samples/issue8682/artifact_loan_form.hwp");
}

/// 반례: 마지막 행이 중첩 표를 담으면 회수를 입증할 수 없어 앞 행도 함께 줄어든다.
#[test]
fn issue_8682_nested_tail_keeps_proportional_shrink() {
    const PARA: usize = 12;
    let core = load("samples/issue8682/ai_procurement_appendix.hwp");
    assert_eq!(core.page_count(), 4);
    let table = table_at(&core, PARA);
    let declared = declared_rows(table);
    let rows = rendered_rows(&core, 2, PARA, declared.len());
    let target = f64::from(table.common.height) / HU_PER_PX;
    assert!(
        (rows.iter().sum::<f64>() - target).abs() <= 0.5,
        "표는 선언 높이를 지킨다"
    );
    let last = declared.len() - 1;
    assert!(
        (0..last).any(|row| rows[row] < declared[row] - 0.1),
        "마지막 행을 회수하지 못하면 앞 행도 줄어든다 — 그림 {rows:?}, 선언 {declared:?}"
    );
}
