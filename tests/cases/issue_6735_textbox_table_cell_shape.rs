//! Issue #6735: 글상자 내장 표 셀의 `Control::Shape`가 렌더 트리에서 누락된다.
//!
//! `shape_layout::layout_textbox_content`가 글상자 안 표를
//! `table_cell_content::layout_embedded_table`로 보내지만, 그 경로는 셀 문단의
//! `Control::Picture`만 하위화했다. 공개 IR로 `TextBox → Table → Cell → Shape`
//! 구조를 만들고 가장 안쪽 도형의 글상자 문구가 페이지 렌더 트리에 남는지 고정한다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::{
    CommonObjAttr, DrawingObjAttr, RectangleShape, ShapeObject, TextBox, TextWrap,
};
use rhwp::model::style::ParaShape;
use rhwp::model::table::{Cell, Table};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::serializer::hwpx::serialize_hwpx;

const INNER_LABEL: &str = "NESTED-CELL-SHAPE";

fn paragraph(text: &str) -> Paragraph {
    Paragraph {
        text: text.to_string(),
        char_count: text.chars().count() as u32,
        char_offsets: (0..text.chars().count() as u32).collect(),
        ..Default::default()
    }
}

fn rectangle_textbox(
    width: u32,
    height: u32,
    treat_as_char: bool,
    paragraphs: Vec<Paragraph>,
) -> ShapeObject {
    ShapeObject::Rectangle(RectangleShape {
        common: CommonObjAttr {
            width,
            height,
            treat_as_char,
            text_wrap: TextWrap::InFrontOfText,
            ..Default::default()
        },
        drawing: DrawingObjAttr {
            text_box: Some(TextBox {
                max_width: width,
                paragraphs,
                ..Default::default()
            }),
            ..Default::default()
        },
        ..Default::default()
    })
}

fn document_with_shape_inside_textbox_table_cell(inner_treat_as_char: bool) -> Document {
    let inner_shape = rectangle_textbox(
        9_000,
        2_500,
        inner_treat_as_char,
        vec![paragraph(INNER_LABEL)],
    );
    let cell_para = Paragraph {
        controls: vec![Control::Shape(Box::new(inner_shape))],
        ..Default::default()
    };
    let cell = Cell {
        row_span: 1,
        col_span: 1,
        width: 24_000,
        height: 6_000,
        paragraphs: vec![cell_para],
        ..Default::default()
    };
    let mut embedded_table = Table {
        row_count: 1,
        col_count: 1,
        cells: vec![cell],
        ..Default::default()
    };
    embedded_table.common.width = 24_000;
    embedded_table.common.height = 6_000;
    embedded_table.common.treat_as_char = true;

    let textbox_para = Paragraph {
        controls: vec![Control::Table(Box::new(embedded_table))],
        ..Default::default()
    };
    let outer_shape = rectangle_textbox(30_000, 12_000, true, vec![textbox_para]);

    let mut doc = Document::default();
    doc.doc_info.para_shapes = vec![ParaShape::default()];
    doc.sections.push(Section {
        paragraphs: vec![Paragraph {
            controls: vec![Control::Shape(Box::new(outer_shape))],
            ..Default::default()
        }],
        ..Default::default()
    });
    doc
}

fn collect_text(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(run.display_or_text());
    }
    for child in &node.children {
        collect_text(child, out);
    }
}

fn inner_label_context(node: &RenderNode) -> Option<(bool, usize)> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.display_or_text().contains(INNER_LABEL) {
            return run
                .cell_context
                .as_ref()
                .map(|context| (context.in_textbox, context.path.len()));
        }
    }
    node.children.iter().find_map(inner_label_context)
}

fn assert_nested_shape_content(inner_treat_as_char: bool) {
    let document = document_with_shape_inside_textbox_table_cell(inner_treat_as_char);
    if let Ok(dir) = std::env::var("RHWP_6735_EVIDENCE_DIR") {
        fs::create_dir_all(&dir).expect("합성 시각 증적 디렉터리 생성");
        let bytes = serialize_hwpx(&document).expect("합성 HWPX 직렬화");
        let mode = if inner_treat_as_char {
            "inline"
        } else {
            "floating"
        };
        let output =
            Path::new(&dir).join(format!("issue_6735_textbox_table_cell_{mode}_shape.hwpx"));
        fs::write(output, bytes).expect("합성 시각 증적 저장");
    }

    let mut core = DocumentCore::new_empty();
    core.set_document(document);

    let tree = core
        .build_page_render_tree(0)
        .expect("합성 문서 첫 페이지 render tree");
    let mut text = String::new();
    collect_text(&tree.root, &mut text);

    assert!(
        text.contains(INNER_LABEL),
        "글상자 내장 표 셀의 도형과 도형 내부 텍스트가 누락됐다: {text:?}"
    );
    assert_eq!(
        inner_label_context(&tree.root),
        Some((true, 3)),
        "바깥 글상자, 내장 표 셀, 안쪽 도형 글상자의 전체 경로를 보존해야 한다"
    );
}

#[test]
fn textbox_embedded_table_cell_inline_shape_keeps_its_textbox_content() {
    assert_nested_shape_content(true);
}

#[test]
fn textbox_embedded_table_cell_floating_shape_keeps_its_textbox_content() {
    assert_nested_shape_content(false);
}
