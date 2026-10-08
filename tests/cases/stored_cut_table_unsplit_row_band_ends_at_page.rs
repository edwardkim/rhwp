//! 저장 쪽 컷을 기록한 RowBreak 표에서, 저장 되감김이 없는 행이 쪽 끝에 걸리면 그 행의
//! 남은 빈 밴드는 쪽 경계에서 끝난다 — 다음 쪽 첫 행은 표 머리 바로 아래에서 시작한다.
//!
//! # 적용 경로
//!
//! `scan_ordinary_row_step` 의 일반 행 밴드 컷 갈래(`row_step.rs`, #5585)에서
//! `storage_records_page_cuts`(표의 어느 칸이든 저장 쪽 되감김을 가짐 · 저장 조판 프로필 ·
//! 편집·재흐름 없음) 이고 현재 행에 저장 되감김이 없으면, 남은 물리 높이를 끝 행 높이로
//! 소비하고(`end_row_height_override = rest`) 밴드를 다음 조각 시작 행으로 넘기지 않는다.
//!
//! # 비적용 경로
//!
//! - 저장 쪽 컷이 없는 표: 밴드를 다음 쪽 첫머리로 넘긴다(`issue_5585_ordinary_row_band_cut`).
//! - 현재 행에 저장 되감김이 있는 행: 되감김 컷 경로가 처리한다(`issue_6761_*`).
//!
//! # 독립 기대값 — 한/글 정본 PDF
//!
//! `pdf/2025 행정업무운영 편람(최종)-hwp-kopub-2024.pdf` · `-hwpx-kopub-2024.pdf` 의 같은
//! 쪽 글줄 위끝(PyMuPDF `get_text('dict')` 줄 bbox, pt→px ×96/72). 이 갈래가 발화하는 다섯
//! 경계(구역 11 부록 103×2 표 행 30·37·56·77, 구역 12 11행 표 행 8)의 다음 쪽 첫 본문 글줄이다.
//! PDF 글줄 위끝은 렌더 트리 `TextLine.y` 보다 0.6px 아래에 잡힌다(같은 쪽 반복 머리 세 줄
//! 100.7/121.1/137.9 ↔ 100.1/120.7/137.5). 밴드를 넘기면 첫 행이 21.5px(행 30) 내려간다.
//!
//! 표본 1,209건·코퍼스 9,991건에서 이 갈래는 19번 발화하고 그중 12번이 출력을 바꾼다
//! (편람 hwp·hwpx 각 5, 코퍼스 `41135`·`156560387` 각 1). 코퍼스 둘도 한/글 2020 정본에서
//! 밴드를 끝낸 쪽이 더 가깝다(다음 쪽 글줄 중앙값 |Δy| 0.8 vs 12.3px, 1.3 vs 6.1px).
//! 나머지 7번은 밴드를 넘겨도 출력이 같다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// PDF 줄 위끝 − 렌더 트리 `TextLine.y` (반복 머리 세 줄로 잰 값).
const PDF_LINE_TOP_OFFSET: f64 = 0.6;

/// `(0-based 쪽, 표 행 수, 정본 첫 본문 글줄 위끝 px)` — 반복 머리(y < 150) 아래 첫 줄.
const BOUNDARIES: [(u32, u16, f64); 5] = [
    (332, 103, 163.4),
    (335, 103, 163.3),
    (346, 103, 163.3),
    (358, 103, 163.3),
    (370, 11, 102.6),
];

fn load(path: &str) -> DocumentCore {
    let full = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let bytes = std::fs::read(&full).unwrap_or_else(|e| panic!("재현체 {}: {e}", full.display()));
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

fn has_text(node: &RenderNode) -> bool {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if !run.display_or_text().trim().is_empty() {
            return true;
        }
    }
    node.children.iter().any(has_text)
}

/// 행 수가 `rows` 인 바깥 표 안에서, 반복 머리 아래(`min_y` 이상) 첫 글줄의 위끝.
fn first_body_line(node: &RenderNode, rows: u16, min_y: f64) -> Option<f64> {
    fn lines(node: &RenderNode, out: &mut Vec<f64>) {
        if matches!(node.node_type, RenderNodeType::TextLine(_)) && has_text(node) {
            out.push(node.bbox.y);
        }
        for child in &node.children {
            lines(child, out);
        }
    }
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.row_count == rows && table.cell_context.is_none() {
            let mut ys = Vec::new();
            lines(node, &mut ys);
            return ys.into_iter().filter(|y| *y >= min_y).reduce(f64::min);
        }
    }
    node.children
        .iter()
        .find_map(|child| first_body_line(child, rows, min_y))
}

fn check(path: &str) {
    let core = load(path);
    let mut misses = Vec::new();
    for (page, rows, oracle_top) in BOUNDARIES {
        let tree = core.build_page_render_tree(page).expect("렌더 트리");
        // 103행 표는 머리 세 줄(≤137.5)을 반복한다. 11행 표는 머리 반복이 없다.
        let min_y = if rows == 103 { 150.0 } else { 0.0 };
        let Some(y) = first_body_line(&tree.root, rows, min_y) else {
            misses.push(format!("{page}쪽: {rows}행 표 글줄 없음"));
            continue;
        };
        let expected = oracle_top - PDF_LINE_TOP_OFFSET;
        if (y - expected).abs() > 1.0 {
            misses.push(format!(
                "{page}쪽: 첫 본문 글줄 {y:.1} ≠ 정본 {expected:.1}"
            ));
        }
    }
    assert!(misses.is_empty(), "{path}: {misses:?}");
}

#[test]
fn handbook_hwp_unsplit_row_band_ends_at_page_boundary() {
    check("samples/2025 행정업무운영 편람(최종).hwp");
}

#[test]
fn handbook_hwpx_unsplit_row_band_ends_at_page_boundary() {
    check("samples/2025 행정업무운영 편람(최종).hwpx");
}
