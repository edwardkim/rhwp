//! 검사 유형: 관계형·고정형.
//! 주석 갱신: 2026-11-06T11:54:54+09:00; 직전 파일 커밋: 910d4e96208fdc8acd5419a2baba3321c7174f42 (2026-11-05T05:12:52+09:00).
//! 2026-11-06 Visual Sweep: 96dpi·PDFium·색상 경계 보정 전 2px raw 실루엣.
//! 같은 입력의 저장 버전에 맞는 Desktop Print(2020→compat 2022, 2024→compat 2024)와 대조.
//! short_row_grid_r21: 전 1쪽 Native 99.99030% / fresh optimized WASM 100.00000%; 전쪽 최저 99.99030% / 100.00000%.
//! short_row_grid_r4: 전 1쪽 Native 99.99207% / fresh optimized WASM 100.00000%; 전쪽 최저 99.99207% / 100.00000%.
//! 증거: mydocs/pr/assets/hyunmyungjanelee-20261106/{native,wasm}-<입력-stem>.tsv,
//! input-reference-pairs.json·측정 JSON·대표 PNG 및 해당 원 PR review 문서.
//! Native/fresh WASM의 직접 확대 판독과 입력·PDF·제품 해시는 공통 기록에 연결한다.
//!
//! #8584: 짧은 행의 칸은 다른 행이 고정한 열 경계 뒤까지 넓어진다.
//!
//! 재현물은 공개 샘플(test-image.hwpx, issue3738)로 만든 23열 표다. 0행 머리 칸
//! (15열 병합·2행 병합, 선언 폭 39397HU)의 행은 칸 폭 합이 표 폭보다 짧고, 19행 칸들이
//! 열 경계 14 를 42959HU 에 고정한다. 칸 폭 등식이 서로 모순되는 표다.
//! - `short_row_grid_r21.hwpx`: 웹한글기안기 인쇄에서 머리 칸 끝 ≈ 43050HU(경계 14 뒤),
//!   머리 글(20pt 21자)이 한 줄. `pdf/issue8584/short_row_grid_r21-webhwp-print.pdf`
//! - `short_row_grid_r4.hwpx`(대조군, 0~3행만): 경계 14 를 고정하는 행이 없어 한컴도
//!   머리 칸을 선언 폭(≈39400HU)으로 두고 두 줄로 꺾는다.
//!   `pdf/issue8584/short_row_grid_r4-webhwp-print.pdf`
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::DocumentCore;

const HU_PER_PX: f64 = 7200.0 / 96.0;

fn core(name: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue8584")
        .join(name);
    DocumentCore::from_bytes(&std::fs::read(path).expect("required fixture")).expect("HWPX parse")
}

fn nodes(node: &RenderNode) -> Vec<&RenderNode> {
    let mut out = vec![node];
    for child in &node.children {
        out.extend(nodes(child));
    }
    out
}

struct Cell<'a> {
    row: u16,
    col: u16,
    col_span: u16,
    bbox: BoundingBox,
    node: &'a RenderNode,
}

fn cells(root: &RenderNode) -> Vec<Cell<'_>> {
    nodes(root)
        .into_iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::TableCell(c) => Some(Cell {
                row: c.row,
                col: c.col,
                col_span: c.col_span,
                bbox: n.bbox,
                node: n,
            }),
            _ => None,
        })
        .collect()
}

fn cell<'a>(cells: &'a [Cell<'a>], row: u16, col: u16) -> &'a Cell<'a> {
    cells
        .iter()
        .find(|c| c.row == row && c.col == col)
        .unwrap_or_else(|| panic!("r{row}c{col} 칸이 있어야 한다"))
}

fn text_lines(cell: &RenderNode) -> usize {
    nodes(cell)
        .into_iter()
        .filter(|n| matches!(n.node_type, RenderNodeType::TextLine(_)))
        .count()
}

/// 같은 열 경계를 쓰는 칸 가장자리는 행과 무관하게 한 x 에 놓이고, 경계 번호가 커지면
/// x 도 커진다.
fn assert_monotonic_boundaries(cells: &[Cell<'_>]) {
    let mut edges: Vec<(usize, f64)> = Vec::new();
    for c in cells {
        edges.push((usize::from(c.col), c.bbox.x));
        edges.push((usize::from(c.col + c.col_span), c.bbox.x + c.bbox.width));
    }
    edges.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)));
    for pair in edges.windows(2) {
        let ((ia, xa), (ib, xb)) = (pair[0], pair[1]);
        if ia == ib {
            assert!(
                (xa - xb).abs() < 0.5,
                "경계 {ia} 가 행마다 다르다: {xa:.2} vs {xb:.2}"
            );
        } else {
            assert!(
                xb > xa,
                "경계 {ia}({xa:.2}) 뒤의 경계 {ib}({xb:.2}) 가 앞에 있다"
            );
        }
    }
}

#[test]
fn short_head_cell_widens_past_boundary_fixed_by_lower_row() {
    let core = core("short_row_grid_r21.hwpx");
    let tree = core.build_page_render_tree(0).expect("page tree");
    let cells = cells(&tree.root);
    let head = cell(&cells, 0, 0);
    let fixed = cell(&cells, 19, 14);
    let head_right = head.bbox.x + head.bbox.width;
    assert!(
        head_right > fixed.bbox.x + 0.5,
        "머리 칸 끝 {head_right:.2} 이 19행이 고정한 경계 14({:.2}) 뒤에 있어야 한다",
        fixed.bbox.x
    );
    // 밀린 경계는 고정 경계에서 최소 열 폭만큼만 떨어진다(한컴 ≈ 90~140HU).
    assert!(
        (head_right - fixed.bbox.x) * HU_PER_PX < 300.0,
        "머리 칸이 고정 경계보다 지나치게 넓다: {:.0}HU",
        (head_right - fixed.bbox.x) * HU_PER_PX
    );
    assert_monotonic_boundaries(&cells);
    assert_eq!(
        text_lines(head.node),
        1,
        "넓어진 머리 칸의 글은 한 줄이어야 한다"
    );
}

#[test]
fn control_without_fixing_row_keeps_declared_head_cell() {
    let core = core("short_row_grid_r4.hwpx");
    let tree = core.build_page_render_tree(0).expect("page tree");
    let cells = cells(&tree.root);
    let head = cell(&cells, 0, 0);
    let width_hu = head.bbox.width * HU_PER_PX;
    assert!(
        (width_hu - 39397.0).abs() < 60.0,
        "대조군 머리 칸은 선언 폭(39397HU) 그대로여야 한다: {width_hu:.0}HU"
    );
    assert_monotonic_boundaries(&cells);
    assert_eq!(
        text_lines(head.node),
        2,
        "대조군 머리 칸의 글은 한컴처럼 두 줄로 꺾인다"
    );
}

#[test]
#[cfg(all(feature = "pdf-import", target_pointer_width = "64"))]
fn contradictory_grid_reopen_refuses_the_remaining_request_budget_and_recovers() {
    use rhwp::{parser::pdf::PdfImportPolicy, PdfImportSession};
    const SOURCE: &[u8] = include_bytes!("../../samples/issue8584/short_row_grid_r21.hwpx");
    let mut healthy = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let expected = healthy.reopen_hwpx_export(SOURCE, 1024 * 1024).unwrap();
    let used = healthy.resource_usage().decoded_bytes;
    // Native 64-bit, minimal-font corrected public reopen used 3,065,061
    // decoded bytes. An input policy one byte below that observation must
    // refuse. Later producers may reject earlier; this is not a counter pin.
    let cap = 3_065_061 - 1;
    assert!(
        used > cap,
        "contradictory grid scratch bypassed the request"
    );
    let mut policy = PdfImportPolicy::default();
    policy.backend.decoded_bytes = cap;
    let mut refused = PdfImportSession::new(policy).unwrap();
    let error = refused
        .reopen_hwpx_export(SOURCE, 1024 * 1024)
        .err()
        .unwrap();
    assert!(error.to_string().contains("PDF_RESOURCE_LIMIT:"), "{error}");
    let failed = refused.resource_usage();
    assert!(failed.failure.is_some());
    assert!(failed.decoded_bytes <= cap);
    assert!(refused.reopen_hwpx_export(SOURCE, 1024 * 1024).is_err());
    assert_eq!(refused.resource_usage().decoded_bytes, failed.decoded_bytes);
    let mut policy = PdfImportPolicy::default();
    policy.backend.decoded_bytes = used;
    let mut fresh = PdfImportSession::new(policy).unwrap();
    let actual = fresh.reopen_hwpx_export(SOURCE, 1024 * 1024).unwrap();
    assert_eq!(actual.page_count(), expected.page_count());
    assert_eq!(
        actual.export_hwpx_native().unwrap(),
        expected.export_hwpx_native().unwrap()
    );
}
