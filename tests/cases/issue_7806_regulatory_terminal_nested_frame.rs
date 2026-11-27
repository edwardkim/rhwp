//! 관계형·고정형 — #7806 21쪽 소속, 중첩 표의 원본 아래 여백과 이어받기 내용.
//! 주석 갱신 2026-11-28T04:50:06+09:00. 직전 파일 커밋 1c5b9928b96c1aba5654d62029a9f9aabd75efba 2026-11-28T03:38:16+09:00.
//! Native Skia·fresh WASM Print를 독립 한컴 PDF에 96 DPI·PDFium·2px raw 실루엣으로 대조했다.
//! 측정 코드 1e773b0314979e9198435eb4a6249e00430c6f58.
//! regulatory-impact-56345 · 기준 pdf/issue6111/56345_regulatory_impact_analysis-hwp-2020.pdf · 전체 21쪽.
//! 전쪽 최저 Native 99.58% / fresh WASM 99.79%.
//! 20쪽 Native 100.00% / fresh WASM 99.99%.
//! 21쪽 Native 100.00% / fresh WASM 100.00%.
//! 전쪽 TSV와 대표 PNG는 mydocs/pr/assets/issues7789_7812/regulatory-impact-56345/에 있다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::renderer::{hwpunit_to_px, DEFAULT_DPI};

const PARENT_PARA: usize = 359;
const TERMINAL_CELL: usize = 11;

fn matching_nodes<'a>(
    node: &'a RenderNode,
    predicate: &impl Fn(&RenderNode) -> bool,
    output: &mut Vec<&'a RenderNode>,
) {
    if predicate(node) {
        output.push(node);
    }
    for child in &node.children {
        matching_nodes(child, predicate, output);
    }
}

fn parent_table(page: &RenderNode) -> &RenderNode {
    let mut tables = Vec::new();
    matching_nodes(
        page,
        &|node| {
            matches!(&node.node_type, RenderNodeType::Table(table)
            if table.para_index == Some(PARENT_PARA)
                && table.control_index == Some(0)
                && table.cell_context.is_none())
        },
        &mut tables,
    );
    let [parent] = tables.as_slice() else {
        panic!("한 쪽에 현재 부모 표의 조각 하나가 있어야 한다");
    };
    parent
}

fn nested_table(parent: &RenderNode) -> &RenderNode {
    let mut tables = Vec::new();
    matching_nodes(
        parent,
        &|node| {
            matches!(&node.node_type, RenderNodeType::Table(table)
            if table.para_index == Some(0)
                && table.control_index == Some(1)
                && table.cell_context.as_ref().is_some_and(|context|
                    context.parent_para_index == PARENT_PARA
                        && context.outermost_cell() == Some(TERMINAL_CELL)))
        },
        &mut tables,
    );
    let [child] = tables.as_slice() else {
        panic!("마지막 셀의 자식 표 조각을 한 번 그려야 한다");
    };
    child
}

fn compact_text(node: &RenderNode) -> String {
    let mut runs = Vec::new();
    matching_nodes(
        node,
        &|node| matches!(&node.node_type, RenderNodeType::TextRun(_)),
        &mut runs,
    );
    runs.into_iter()
        .filter_map(|run| match &run.node_type {
            RenderNodeType::TextRun(text) => Some(text.display_or_text()),
            _ => None,
        })
        .flat_map(str::chars)
        .filter(|char| !char.is_whitespace())
        .collect()
}

#[test]
fn regulatory_terminal_nested_table_keeps_parent_padding_at_the_saved_cut() {
    let bytes = std::fs::read("samples/issue6111/56345_regulatory_impact_analysis.hwp")
        .expect("원본 규제영향분석서");
    let mut core = DocumentCore::from_bytes(&bytes).expect("원본 HWP의 저장 조판");
    core.set_hangul2024_compat(false);
    assert_eq!(core.page_count(), 21, "독립 한컴 Print와 같은 쪽수");

    let source = &core.document().sections[0].paragraphs[PARENT_PARA];
    let Some(Control::Table(parent_source)) = source.controls.first() else {
        panic!("지정된 원본 문단이 부모 표를 소유해야 한다");
    };
    let padding = parent_source.cells[TERMINAL_CELL].effective_padding(&parent_source.padding);
    let bottom_padding = hwpunit_to_px(i32::from(padding.bottom), DEFAULT_DPI);
    let opening = core.build_page_render_tree(19).expect("20쪽 첫 조각");
    let continued = core.build_page_render_tree(20).expect("21쪽 이어받기");
    let opening_parent = parent_table(&opening.root);
    let opening_child = nested_table(opening_parent);
    let continued_child = nested_table(parent_table(&continued.root));
    let parent_bottom = opening_parent.bbox.y + opening_parent.bbox.height;
    let child_bottom = opening_child.bbox.y + opening_child.bbox.height;
    let gap = parent_bottom - child_bottom;
    assert!(
        gap >= bottom_padding - hwpunit_to_px(2, DEFAULT_DPI),
        "자식 조각 뒤에는 원본 부모의 아래 안 여백이 있어야 한다: {gap} < {bottom_padding}"
    );

    let opening_text = compact_text(opening_child);
    let continued_text = compact_text(continued_child);
    assert!(opening_text.contains("기업집단의주주등이정보를얻음으로써"));
    assert_eq!(opening_text.matches("수는없음").count(), 0);
    assert_eq!(continued_text.matches("수는없음").count(), 1);
}
