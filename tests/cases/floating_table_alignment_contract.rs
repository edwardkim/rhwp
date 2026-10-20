//! 합성 상자의 정렬 관계 계약이다. 실물 시각 일치는 독립 Print와 별도로 검증한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::{
    control::Control,
    document::{Document, Section},
    page::PageDef,
    paragraph::Paragraph,
    shape::{HorzAlign, HorzRelTo, TextWrap},
    style::{CharShape, ParaShape},
    table::{Cell, Table},
};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const PAPER_WIDTH: u32 = 60_000;
const MARGIN: u32 = 5_000;

fn table_bounds(width: u32, alignment: HorzAlign, offset: i32) -> (f64, f64) {
    let mut table = Table {
        row_count: 1,
        col_count: 1,
        cells: vec![Cell {
            row_span: 1,
            col_span: 1,
            width,
            height: 3_000,
            paragraphs: vec![Paragraph {
                text: "ANCHOR".to_owned(),
                char_count: 7,
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    };
    table.common.width = width;
    table.common.height = 3_000;
    table.common.treat_as_char = false;
    table.common.horz_rel_to = HorzRelTo::Page;
    table.common.horz_align = alignment;
    table.common.horizontal_offset = offset as u32;
    table.common.text_wrap = TextWrap::TopAndBottom;
    table.rebuild_grid();
    let mut section = Section::default();
    section.section_def.page_def = PageDef {
        width: PAPER_WIDTH,
        height: 80_000,
        margin_left: MARGIN,
        margin_right: MARGIN,
        margin_top: MARGIN,
        margin_bottom: MARGIN,
        ..Default::default()
    };
    section.paragraphs.push(Paragraph {
        controls: vec![Control::Table(Box::new(table))],
        ..Default::default()
    });
    let mut doc = Document::default();
    doc.doc_properties.section_count = 1;
    doc.doc_info.para_shapes.push(ParaShape::default());
    doc.doc_info.char_shapes.push(CharShape::default());
    doc.sections.push(section);
    let bytes = rhwp::serializer::hwpx::serialize_hwpx(&doc).expect("source HWPX");
    let core = DocumentCore::from_bytes(&bytes).expect("public layout path");
    let page = core.build_page_render_tree(0).expect("page tree");
    fn find(node: &RenderNode) -> Option<&RenderNode> {
        if matches!(node.node_type, RenderNodeType::Table(_)) {
            Some(node)
        } else {
            node.children.iter().find_map(find)
        }
    }
    fn text(node: &RenderNode, value: &mut String) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            value.push_str(&run.text);
        }
        for child in &node.children {
            text(child, value);
        }
    }
    let table = find(&page.root).expect("source table must not disappear");
    let mut content = String::new();
    text(table, &mut content);
    assert_eq!(content, "ANCHOR", "정렬 과정에서 원래 내용을 보존한다");
    (table.bbox.x, table.bbox.width)
}

#[test]
fn floating_table_keeps_the_reference_midpoint_and_edges_for_all_source_widths() {
    // 아래 값은 원본 HU 상자이며 실물 페이지의 고정 픽셀 위치가 아니다.
    let scale = 96.0 / 7200.0;
    let reference_left = f64::from(MARGIN) * scale;
    let reference_right = f64::from(PAPER_WIDTH - MARGIN) * scale;
    for width in [24_000, PAPER_WIDTH - MARGIN * 2, 56_000] {
        for offset in [-700, 0, 700] {
            for align in [HorzAlign::Left, HorzAlign::Center, HorzAlign::Right] {
                let (left, painted_width) = table_bounds(width, align, offset);
                assert!((painted_width - f64::from(width) * scale).abs() < 1e-6);
                let offset = f64::from(offset) * scale;
                let (actual, expected) = match align {
                    HorzAlign::Left => (left, reference_left + offset),
                    HorzAlign::Center => (
                        left + painted_width / 2.0,
                        (reference_left + reference_right) / 2.0 + offset,
                    ),
                    HorzAlign::Right => (left + painted_width, reference_right - offset),
                    _ => unreachable!(),
                };
                assert!(
                    (actual - expected).abs() < 1e-6,
                    "{align:?}, source width {width}"
                );
            }
        }
    }
}
