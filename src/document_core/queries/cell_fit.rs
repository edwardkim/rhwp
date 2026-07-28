//! [#3480] 셀 맞춤 측정 — "이 값이 이 칸에 들어가는가"를 **렌더 트리에서 직독**한다.
//!
//! `edit set-cell`·`edit fill-fields` 는 값이 칸을 넘쳐도 성공만 보고했다. 에이전트는
//! 렌더 결과를 보지 않으므로, 사람이라면 제출하지 않을 문서를 완성본으로 넘긴다.
//!
//! 이 질의는 **조판 엔진이 있어야만 가능한 답**이다. 폭을 따로 추정하지 않고 실제 렌더
//! 트리의 셀 상자와 그 안의 텍스트 줄을 읽는다 — 측정 경로와 렌더 경로가 갈라지는
//! 알려진 함정(#2237)을 피하기 위해서다.

use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::renderer::render_tree::{RenderNode, RenderNodeType};

/// 한 셀의 렌더 실측치.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CellFit {
    /// 셀 상자 폭 (px)
    pub cell_width_px: f64,
    /// 셀 상자 높이 (px)
    pub cell_height_px: f64,
    /// 셀 안에서 가장 넓은 렌더 줄의 폭 (px)
    pub text_width_px: f64,
    /// 셀 안 렌더 줄 수
    pub lines: usize,
    /// 렌더 줄이 셀 상자 오른쪽을 넘었는가
    pub clipped_horizontally: bool,
}

impl CellFit {
    /// 값이 이 칸에 한 줄로 들어갔는가.
    pub fn fits_on_one_line(&self) -> bool {
        self.lines <= 1 && !self.clipped_horizontally
    }
}

/// 셀을 가리키는 방법 — 격자 좌표 또는 모델 셀 인덱스.
#[derive(Debug, Clone, Copy)]
pub enum CellSelector {
    /// `set-cell` 의 격자 주소
    RowCol { row: u16, col: u16 },
    /// 필드 위치(`NestedEntry::TableCell`)가 주는 모델 셀 인덱스
    ModelIndex(usize),
}

impl DocumentCore {
    /// [#3480] 표 셀의 렌더 실측치를 돌려준다.
    ///
    /// `section_index`/`para_index`/`control_index` 는 표 컨트롤의 위치다
    /// (`table_extract::extract_tables` 가 주는 좌표와 같다).
    ///
    /// 표를 찾지 못하면 `Ok(None)` — 측정 실패는 편집 실패가 아니므로 오류로 올리지 않는다.
    pub fn measure_cell_fit(
        &self,
        section_index: usize,
        para_index: usize,
        control_index: usize,
        selector: CellSelector,
    ) -> Result<Option<CellFit>, HwpError> {
        let total_pages = self.page_count() as usize;
        for page in 0..total_pages {
            let tree = self.build_page_tree_cached(page as u32)?;
            if let Some(fit) = find_cell_fit(
                &tree.root,
                section_index,
                para_index,
                control_index,
                selector,
            ) {
                return Ok(Some(fit));
            }
        }
        Ok(None)
    }
}

fn selector_matches(node: &crate::renderer::render_tree::TableCellNode, s: CellSelector) -> bool {
    match s {
        CellSelector::RowCol { row, col } => node.row == row && node.col == col,
        CellSelector::ModelIndex(idx) => node.model_cell_index == Some(idx as u32),
    }
}

fn find_cell_fit(
    node: &RenderNode,
    sec: usize,
    para: usize,
    ctrl: usize,
    selector: CellSelector,
) -> Option<CellFit> {
    if let RenderNodeType::Table(ref table) = node.node_type {
        if table.section_index == Some(sec)
            && table.para_index == Some(para)
            && table.control_index == Some(ctrl)
        {
            for child in &node.children {
                let RenderNodeType::TableCell(ref cell) = child.node_type else {
                    continue;
                };
                if !selector_matches(cell, selector) {
                    continue;
                }
                return Some(measure(child));
            }
            // 표는 찾았으나 그 셀이 이 쪽에 없다 — 분할 표의 다음 쪽을 계속 본다.
            return None;
        }
    }
    for child in &node.children {
        if let Some(found) = find_cell_fit(child, sec, para, ctrl, selector) {
            return Some(found);
        }
    }
    None
}

/// 셀 노드 아래의 텍스트 줄을 모아 실측치를 만든다.
fn measure(cell_node: &RenderNode) -> CellFit {
    let mut lines = 0usize;
    let mut widest = 0.0f64;
    let mut rightmost = f64::NEG_INFINITY;
    collect_lines(cell_node, &mut lines, &mut widest, &mut rightmost);

    let cell_right = cell_node.bbox.x + cell_node.bbox.width;
    CellFit {
        cell_width_px: cell_node.bbox.width,
        cell_height_px: cell_node.bbox.height,
        text_width_px: if widest.is_finite() { widest } else { 0.0 },
        lines,
        // 0.5px 는 렌더 반올림 여유 — 이보다 작은 초과는 넘침으로 보지 않는다.
        clipped_horizontally: rightmost.is_finite() && rightmost > cell_right + 0.5,
    }
}

fn collect_lines(node: &RenderNode, lines: &mut usize, widest: &mut f64, rightmost: &mut f64) {
    for child in &node.children {
        match child.node_type {
            RenderNodeType::TextLine(_) => {
                *lines += 1;
                // 줄 상자는 흐름 폭을 담을 뿐이라, 실제 글자가 놓인 오른쪽 끝은
                // 자식 런에서 구한다.
                let mut run_right = f64::NEG_INFINITY;
                let mut run_left = f64::INFINITY;
                collect_run_extent(child, &mut run_left, &mut run_right);
                if run_right.is_finite() && run_left.is_finite() {
                    *widest = widest.max(run_right - run_left);
                    *rightmost = rightmost.max(run_right);
                }
                // 줄 안에 중첩 표가 있을 수 있으므로 더 내려가지 않는다.
            }
            // 중첩 표의 줄까지 세면 이 셀의 줄 수가 아니게 된다.
            RenderNodeType::Table(_) => {}
            _ => collect_lines(child, lines, widest, rightmost),
        }
    }
}

fn collect_run_extent(node: &RenderNode, left: &mut f64, right: &mut f64) {
    for child in &node.children {
        if matches!(child.node_type, RenderNodeType::TextRun(_)) {
            *left = left.min(child.bbox.x);
            *right = right.max(child.bbox.x + child.bbox.width);
        }
        collect_run_extent(child, left, right);
    }
}
