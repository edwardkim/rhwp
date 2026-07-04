//! [Task #1748] scattered header RowBreak 표 — 컷 행에 걸친 rowspan 셀 렌더 회귀 가드.
//!
//! `samples/table_scattered_header_rowbreak.hwp`(260×9 RowBreak 표)의 p6/p7 경계에서
//! 페이지 경계가 rowspan 블록 내부를 per-row 분할한다(p6 rows=100..140 end_cut=[1,2]).
//! 컷 부기(end_cut/start_cut)는 컷 행의 row_span==1 셀만 담으므로, 경계에 걸친
//! rowspan 셀은 수정 전:
//! - 컷 페이지(p6): 전체 줄을 무제한 렌더 → 셀/표 박스 아래로 3줄 흘러넘침
//!   (표 경계 1077.6px 대비 잉크 1122.5px, 한글 2024 PDF 대비 Δbot +13px)
//! - 연속 페이지(p7): 처음부터 전체 재렌더 → 중복 + 아래 행 영역 침범
//!   (셀 박스 155.6px 대비 잉크 217.5px)
//!
//! 수정 후 걸친 rowspan 셀은 높이 기반 유닛 컷으로 컷 페이지에 들어가는 줄까지만
//! 렌더하고 연속 페이지는 다음 줄부터 이어그린다 (한글 정합).

use rhwp::renderer::render_tree::RenderNode;
use rhwp::renderer::render_tree::RenderNodeType;
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/table_scattered_header_rowbreak.hwp";
/// p6/p7 (0-based 페이지 인덱스). p6 rows=100..140 end_cut=[1,2], p7 start_cut=[1,2].
const CUT_PAGE: u32 = 5;
const CONT_PAGE: u32 = 6;

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let sample_path = Path::new(repo_root).join(SAMPLE);
    let bytes = fs::read(&sample_path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e:?}"))
}

/// 모든 TableCell 하위 TextLine 이 셀 bbox 하단을 넘지 않는지 수집.
/// (초과량, 셀 bottom, 라인 bottom) 목록 반환.
fn collect_cell_text_overflows(root: &RenderNode, cell_bottom: Option<f64>, out: &mut Vec<f64>) {
    let cell_bottom = if matches!(root.node_type, RenderNodeType::TableCell(_)) {
        Some(root.bbox.y + root.bbox.height)
    } else {
        cell_bottom
    };
    if matches!(root.node_type, RenderNodeType::TextLine(_)) {
        if let Some(cb) = cell_bottom {
            let over = root.bbox.y + root.bbox.height - cb;
            if over > 0.5 {
                out.push(over);
            }
        }
    }
    for child in &root.children {
        collect_cell_text_overflows(child, cell_bottom, out);
    }
}

fn find_table_bbox(root: &RenderNode) -> Option<(f64, f64)> {
    if matches!(root.node_type, RenderNodeType::Table(_)) {
        return Some((root.bbox.y, root.bbox.y + root.bbox.height));
    }
    root.children.iter().find_map(find_table_bbox)
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

/// 컷 페이지(p6): 걸친 rowspan 셀의 텍스트가 셀 박스를 넘지 않는다.
/// 수정 전 최대 44.9px 초과(잉크 1122.5 vs 셀 1077.6).
#[test]
fn cut_page_straddling_rowspan_cell_text_stays_inside_cells() {
    let doc = load_doc();
    let tree = doc
        .build_page_render_tree(CUT_PAGE)
        .unwrap_or_else(|e| panic!("render page {}: {e}", CUT_PAGE + 1));
    let mut overflows = Vec::new();
    collect_cell_text_overflows(&tree.root, None, &mut overflows);
    assert!(
        overflows.is_empty(),
        "p{} 셀 하단 초과 TextLine {}건 (최대 {:.1}px) — 컷 걸침 rowspan 셀 가시범위 미적용 회귀",
        CUT_PAGE + 1,
        overflows.len(),
        overflows.iter().cloned().fold(0.0, f64::max),
    );
}

/// 컷 페이지(p6): 표 프래그먼트 하단 아래에 텍스트 잉크가 없다 (Δbot +13px 의 직접 형상).
#[test]
fn cut_page_no_text_ink_below_table_fragment() {
    let doc = load_doc();
    let tree = doc
        .build_page_render_tree(CUT_PAGE)
        .unwrap_or_else(|e| panic!("render page {}: {e}", CUT_PAGE + 1));
    let (_, table_bottom) = find_table_bbox(&tree.root).expect("p6 table should render");
    let max_bottom = max_text_line_bottom(&tree.root);
    assert!(
        max_bottom <= table_bottom + 0.5,
        "p{} 표 하단({table_bottom:.1}) 아래 텍스트 잉크({max_bottom:.1}) — 컷 페이지 over-fill 회귀",
        CUT_PAGE + 1,
    );
}

/// 연속 페이지(p7): 걸친 rowspan 셀이 처음부터 재렌더되지 않고(중복 없음)
/// 텍스트가 셀 박스 안에 머문다. 수정 전 최대 61.9px 초과(잉크 217.5 vs 셀 155.6).
#[test]
fn continuation_page_straddling_rowspan_cell_text_stays_inside_cells() {
    let doc = load_doc();
    let tree = doc
        .build_page_render_tree(CONT_PAGE)
        .unwrap_or_else(|e| panic!("render page {}: {e}", CONT_PAGE + 1));
    let mut overflows = Vec::new();
    collect_cell_text_overflows(&tree.root, None, &mut overflows);
    assert!(
        overflows.is_empty(),
        "p{} 셀 하단 초과 TextLine {}건 (최대 {:.1}px) — 걸친 rowspan 셀 전체 재렌더(중복) 회귀",
        CONT_PAGE + 1,
        overflows.len(),
        overflows.iter().cloned().fold(0.0, f64::max),
    );
}
