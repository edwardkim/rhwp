//! 관계형·고정형: 발신명의의 실제 glyph 원점과 셀 소유를 독립 Print로 보호한다.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! HWP 179쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! HWPX 179쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.
//! 두 원본 전쪽 Native/fresh WASM 최저 98% 이상인 조건에서 정한 ±2px 기대값이다.
//! 근거: mydocs/plans/issue_8153_positive_terminal_tracking_20261025.md.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::{
    layout::{EmbeddedTextMeasurer, TextMeasurer},
    render_tree::{RenderNode, RenderNodeType},
};
use rhwp::DocumentCore;

fn find<'a>(
    node: &'a RenderNode,
    owner: Option<&'a RenderNode>,
    found: &mut Vec<(&'a RenderNode, &'a RenderNode)>,
) {
    let owner = if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        Some(node)
    } else {
        owner
    };
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text == "발신명의" {
            found.push((node, owner.expect("발신명의의 원본 소유 셀")));
        }
    }
    for child in &node.children {
        find(child, owner, found);
    }
}

fn near(actual: f64, expected: f64) {
    assert!(
        actual.is_finite() && (actual - expected).abs() <= 2.0,
        "독립 96dpi Print 원점: 실제 {actual}, 기대 {expected}, 허용 ±2px"
    );
}

fn verify(extension: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{extension}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 383);
    let tree = core.build_page_render_tree(178).unwrap();
    let mut found = Vec::new();
    find(&tree.root, None, &mut found);
    assert_eq!(found.len(), 1, "179쪽 발신명의의 내용·소유·수량 보존");
    let (node, cell) = found[0];
    let RenderNodeType::TextRun(run) = &node.node_type else {
        unreachable!()
    };
    let positions = run
        .layout_positions
        .clone()
        .unwrap_or_else(|| EmbeddedTextMeasurer.compute_char_positions(&run.text, &run.style));
    assert_eq!(positions.len(), 5);
    // 독립 PDF의 네 실제 glyph 원점(pt)이다. 글자 사이 전진은 유지한다.
    for (offset, point) in positions
        .iter()
        .take(4)
        .zip([235.799988, 255.479996, 275.040009, 294.720001])
    {
        near(node.bbox.x + offset, point * 96.0 / 72.0);
    }
    near(node.bbox.y + run.baseline, 550.919983 * 96.0 / 72.0);
    assert!(node.bbox.x >= cell.bbox.x);
    assert!(node.bbox.x + positions[3] < cell.bbox.x + cell.bbox.width);
    assert!(node.bbox.y >= cell.bbox.y);
    assert!(node.bbox.y + node.bbox.height <= cell.bbox.y + cell.bbox.height);
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn hwp_centers_visible_sender_glyphs_without_terminal_tracking() {
    verify("hwp");
}

#[test]
fn hwpx_centers_visible_sender_glyphs_without_terminal_tracking() {
    verify("hwpx");
}
