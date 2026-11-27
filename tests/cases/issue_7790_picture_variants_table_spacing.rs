//! 관계형·고정형 — #7790 그림 위치가 다른 두 원본의 쪽수와 이어지는 표의 원본 여백.
//! 주석 갱신 2026-11-28T03:36:12+09:00.
//! Native Skia·fresh WASM Print를 독립 한컴 PDF에 96 DPI·PDFium·2px raw 실루엣으로 대조했다.
//! 측정 코드 f3dae4a6483f62e5854417ef73cac111ec5dde10.
//! pic-in-head · 기준 pdf/pic-in-head-01-review-20261009-2020.pdf · 전체 22쪽.
//! 전쪽 최저 Native 99.86% / fresh WASM 99.86%.
//! 10쪽 Native 100.00% / fresh WASM 100.00%.
//! 전쪽 TSV와 대표 PNG는 mydocs/pr/assets/issues7789_7812/pic-in-head/에 있다.
//! pic-in-table · 기준 pdf/pic-in-table-01-review-20261009-2020.pdf · 전체 22쪽.
//! 전쪽 최저 Native 99.86% / fresh WASM 99.86%.
//! 10쪽 Native 100.00% / fresh WASM 100.00%.
//! 전쪽 TSV와 대표 PNG는 mydocs/pr/assets/issues7789_7812/pic-in-table/에 있다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::renderer::{hwpunit_to_px, DEFAULT_DPI};
use rhwp::DocumentCore;

fn collect_owned_tables<'tree>(node: &'tree RenderNode, ordered: &mut Vec<&'tree RenderNode>) {
    if matches!(&node.node_type, RenderNodeType::Table(table)
        if table.section_index == Some(0) && table.para_index == Some(32)
            && matches!(table.control_index, Some(0 | 1)) && table.cell_context.is_none())
    {
        ordered.push(node);
    }
    for child in &node.children {
        collect_owned_tables(child, ordered);
    }
}

#[test]
fn picture_variants_keep_the_saved_gap_between_inline_and_occupied_tables() {
    for path in ["samples/pic-in-head-01.hwp", "samples/pic-in-table-01.hwp"] {
        let bytes = std::fs::read(path).expect("그림 위치만 다른 원본");
        let mut core = DocumentCore::from_bytes(&bytes).expect("원본 열기");
        core.set_hangul2024_compat(false);
        assert_eq!(core.page_count(), 22, "{path}: 독립 Print 쪽수");

        let host = &core.document().sections[0].paragraphs[32];
        let [Control::Table(inline), Control::Table(occupying), ..] = host.controls.as_slice()
        else {
            panic!("{path}: 한 문단의 두 원본 표");
        };
        assert!(inline.common.treat_as_char);
        assert!(!occupying.common.treat_as_char);
        let gap_hu = host.line_segs[0].line_spacing
            + i32::from(inline.outer_margin_bottom)
            + i32::from(occupying.outer_margin_top);
        let expected = hwpunit_to_px(gap_hu, DEFAULT_DPI);

        let page = core.build_page_render_tree(9).expect("독립 Print 10쪽");
        let mut ordered = Vec::new();
        collect_owned_tables(&page.root, &mut ordered);
        let [first, following] = ordered.as_slice() else {
            panic!("{path}: 두 표가 같은 문단과 쪽에 한 번씩 그려져야 한다");
        };
        let (RenderNodeType::Table(first_table), RenderNodeType::Table(following_table)) =
            (&first.node_type, &following.node_type)
        else {
            unreachable!();
        };
        assert_eq!(first_table.control_index, Some(0));
        assert_eq!(following_table.control_index, Some(1));
        let actual = following.bbox.y - (first.bbox.y + first.bbox.height);
        assert!(
            (actual - expected).abs() <= hwpunit_to_px(2, DEFAULT_DPI),
            "{path}: 표 사이에는 원본 저장 줄간격과 두 바깥여백만 둔다: expected={expected}, actual={actual}"
        );
    }
}
