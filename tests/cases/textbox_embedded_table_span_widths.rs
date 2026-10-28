//! 글상자 내장 표도 병합 셀의 선언 폭으로 미지 열 폭을 풀어야 한다.
//! 공개 합성 IR만 사용하며, 기대 기하는 HWPUNIT 선언값(96 dpi에서 /75)이다.

use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::{CommonObjAttr, DrawingObjAttr, RectangleShape, ShapeObject, TextBox};
use rhwp::model::style::ParaShape;
use rhwp::model::table::{Cell, Table};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn cell(row: u16, col: u16, span: u16, width: u32) -> Cell {
    Cell {
        row,
        col,
        col_span: span,
        row_span: 1,
        width,
        height: 3_000,
        paragraphs: vec![Paragraph::default()],
        ..Default::default()
    }
}

fn document(cells: Vec<Cell>, row_count: u16) -> Document {
    let table = Table {
        row_count,
        col_count: 3,
        cells,
        common: CommonObjAttr {
            width: 27_000,
            height: u32::from(row_count) * 3_000,
            treat_as_char: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let shape = ShapeObject::Rectangle(RectangleShape {
        common: CommonObjAttr {
            width: 30_000,
            height: 12_000,
            treat_as_char: true,
            ..Default::default()
        },
        drawing: DrawingObjAttr {
            text_box: Some(TextBox {
                max_width: 30_000,
                paragraphs: vec![Paragraph {
                    controls: vec![Control::Table(Box::new(table))],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        },
        ..Default::default()
    });
    let mut doc = Document::default();
    doc.doc_info.para_shapes = vec![ParaShape::default()];
    doc.sections.push(Section {
        paragraphs: vec![Paragraph {
            controls: vec![Control::Shape(Box::new(shape))],
            ..Default::default()
        }],
        ..Default::default()
    });
    doc
}

fn collect_cells(node: &RenderNode, cells: &mut Vec<(u16, u16, f64)>) {
    if let RenderNodeType::TableCell(cell) = &node.node_type {
        cells.push((cell.row, cell.col, node.bbox.width));
    }
    for child in &node.children {
        collect_cells(child, cells);
    }
}

fn assert_widths(doc: Document, expected: &[(u16, u16, f64)]) {
    let mut core = DocumentCore::new_empty();
    core.set_document(doc);
    let tree = core.build_page_render_tree(0).expect("첫 페이지 렌더 트리");
    let mut actual = Vec::new();
    collect_cells(&tree.root, &mut actual);
    assert_eq!(
        actual.len(),
        expected.len(),
        "셀 누락 또는 중복: {actual:?}"
    );
    for &(row, col, width) in expected {
        let rendered = actual.iter().find(|c| c.0 == row && c.1 == col).unwrap().2;
        assert!(
            (rendered - width).abs() < 0.01,
            "셀 ({row},{col}): 선언 {width}px, 렌더 {rendered}px"
        );
    }
}

#[test]
fn merged_cell_resolves_two_columns_without_single_span_cells() {
    assert_widths(
        document(vec![cell(0, 0, 1, 24_000), cell(0, 1, 2, 3_000)], 1),
        &[(0, 0, 320.0), (0, 1, 40.0)],
    );
}

#[test]
fn overlapping_spans_resolve_the_remaining_column() {
    assert_widths(
        document(
            vec![
                cell(0, 0, 1, 24_000),
                cell(0, 1, 2, 3_000),
                cell(1, 0, 2, 25_500),
                cell(1, 2, 1, 1_500),
            ],
            2,
        ),
        &[(0, 0, 320.0), (0, 1, 40.0), (1, 0, 340.0), (1, 2, 20.0)],
    );
}
