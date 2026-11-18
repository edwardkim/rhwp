//! 관계형·고정형(독립 PDF 전체 쪽수) 회귀 — [#8641] 같은 저장 줄 TAC 표의 공유 기준선, rowspan 초과분이 받는 행.
//! 주석 갱신: 2026-11-06T07:00:53+09:00.
//! 2026-11-06 측정 raw(96dpi, visual_sweep --silhouette-only, silhouette_raw_match_percent; 빌드 해시는 근거 문서):
//! inspection_request_stamp_a 1쪽 Native 98.02% / fresh WASM 98.00%, 전1쪽.
//! inspection_request_stamp_b 1쪽 Native 97.67% / fresh WASM 97.66%, 전1쪽.
//! labor_standards_report 13쪽 Native 99.92% / fresh WASM 99.90%, 전53쪽 최저 94.57% / 95.61%.
//! 근거: mydocs/pr/assets/issue_8641/shared_baseline_rowspan/README.md.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const DPI: f64 = 96.0;

fn hu_to_px(hu: i32) -> f64 {
    f64::from(hu) * DPI / 7200.0
}

fn core(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|error| panic!("parse {sample}: {error:?}"))
}

/// 최외곽 표(칸 안 중첩 표 제외): (문단, 컨트롤, 상자).
fn top_tables(node: &RenderNode, out: &mut Vec<(usize, usize, BoundingBox)>) {
    if let RenderNodeType::Table(table) = &node.node_type {
        if let (Some(para), Some(control)) = (table.para_index, table.control_index) {
            out.push((para, control, node.bbox));
        }
        return;
    }
    for child in &node.children {
        top_tables(child, out);
    }
}

fn page_table(core: &DocumentCore, page: u32, para: usize, control: usize) -> BoundingBox {
    let tree = core.build_page_render_tree(page).expect("렌더 트리");
    let mut tables = Vec::new();
    top_tables(&tree.root, &mut tables);
    tables
        .iter()
        .find(|(p, c, _)| *p == para && *c == control)
        .unwrap_or_else(|| panic!("표 {para}/{control} 없음: {tables:?}"))
        .2
}

/// 문단 글줄 상자: 쪽 위에서부터.
fn para_text_lines(node: &RenderNode, para: usize, out: &mut Vec<BoundingBox>) {
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if line.para_index == Some(para) {
            out.push(node.bbox);
        }
        return;
    }
    for child in &node.children {
        para_text_lines(child, para, out);
    }
}

/// 같은 저장 줄의 표 둘은 기준선을 공유한다: 높이의 85% 지점이 같은 높이에 온다.
#[test]
fn same_line_tables_share_the_owner_baseline() {
    for sample in [
        "samples/issue8641/inspection_request_stamp_a.hwpx",
        "samples/issue8641/inspection_request_stamp_b.hwpx",
    ] {
        let core = core(sample);
        assert_eq!(core.page_count(), 1, "{sample}: 독립 PDF 전체 쪽수");
        let owner = page_table(&core, 0, 0, 2);
        let sibling = page_table(&core, 0, 0, 3);
        let owner_baseline = owner.y + owner.height * 0.85;
        let sibling_baseline = sibling.y + sibling.height * 0.85;
        assert!(
            (owner_baseline - sibling_baseline).abs() <= 1.0,
            "{sample}: 기준선 {owner_baseline:.1} / {sibling_baseline:.1}"
        );
    }
}

/// 병합 칸 선언 초과분은 rs=1 칸이 정한 행이 아니라 병합 칸으로만 정해지는 행이 받는다.
/// 그러면 표 전체 높이가 저장 선언과 같고 뒤 문단이 표 아래에 바로 이어진다.
#[test]
fn rowspan_overflow_does_not_grow_a_row_pinned_by_a_single_row_cell() {
    let core = core("samples/issue8641/labor_standards_report.hwp");
    assert_eq!(core.page_count(), 53, "독립 PDF 전체 쪽수");
    let page = 12;
    let table = page_table(&core, page, 148, 0);
    let declared = hu_to_px(55442);
    assert!(
        (table.height - declared).abs() <= 1.0,
        "표 높이 {:.1}, 저장 높이 {declared:.1}",
        table.height
    );
    let tree = core.build_page_render_tree(page).expect("렌더 트리");
    let mut following = Vec::new();
    para_text_lines(&tree.root, 149, &mut following);
    let first = following.first().expect("뒤 문단 글줄");
    assert!(first.y >= table.y + table.height, "뒤 문단이 표와 겹침");
}
