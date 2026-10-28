//! 관계형: 다른 행의 가상 열로 좌표를 바꿔 공유 괘선에 구멍을 만들지 않는다.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! HWP 64쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! HWPX 64쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.
//! 독립 Print의64쪽 제목 셀 상단은 연속된 회색 획이다. 셀 전체 포함 관계를 검사한다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

fn nodes<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        nodes(child, out);
    }
}

fn check(extension: &str) {
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{extension}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    let tree = core.build_page_render_tree(63).unwrap();
    let mut all = Vec::new();
    nodes(&tree.root, &mut all);
    let tables: Vec<_> = all
        .into_iter()
        .filter(|n| {
            matches!(&n.node_type,
        RenderNodeType::Table(t) if t.row_count==13 && t.col_count==12)
        })
        .collect();
    assert_eq!(tables.len(), 1, "원본 제목 표의 소유");
    let table = tables[0];
    let mut all = Vec::new();
    nodes(table, &mut all);
    let cell = all
        .iter()
        .copied()
        .find(|n| {
            matches!(&n.node_type,
        RenderNodeType::TableCell(c) if c.row==9 && c.col==1)
        })
        .unwrap();
    let mut intervals = Vec::new();
    for n in all {
        if let RenderNodeType::Line(line) = &n.node_type {
            if line.style.color == 0x7f7f7f
                && (line.y1 - line.y2).abs() < 1e-6
                && (line.y1 - cell.bbox.y).abs() <= line.style.width
            {
                intervals.push((line.x1.min(line.x2), line.x1.max(line.x2), line.style.width));
            }
        }
    }
    assert!(!intervals.is_empty(), "독립 Print 제목 셀의 회색 상단 획");
    intervals.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut covered = cell.bbox.x;
    let mut pen = 0.0_f64;
    for (left, right, width) in intervals {
        if right < cell.bbox.x || left > cell.bbox.x + cell.bbox.width {
            continue;
        }
        assert!(
            left - covered <= width,
            "{extension}: 셀 상단 획 내부의 구멍 {}..{left}",
            covered
        );
        covered = covered.max(right);
        pen = pen.max(width);
    }
    assert!(
        covered + pen >= cell.bbox.x + cell.bbox.width,
        "상단 획은 셀 오른쪽까지 이어진다"
    );
}
#[test]
fn hwp_shared_title_border_covers_its_source_cell() {
    check("hwp");
}
#[test]
fn hwpx_shared_title_border_covers_its_source_cell() {
    check("hwpx");
}
