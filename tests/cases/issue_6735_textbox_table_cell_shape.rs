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
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::model::shape::{
    CommonObjAttr, DrawingObjAttr, RectangleShape, ShapeObject, TextBox, TextWrap,
};
use rhwp::model::style::{Alignment, ParaShape};
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

fn inline_pair_positions(
    multiline: bool,
    prefix: &str,
    alignment: Alignment,
) -> Vec<(String, f64, f64)> {
    let mut doc = document_with_shape_inside_textbox_table_cell(true);
    doc.doc_info.para_shapes.push(ParaShape {
        alignment,
        ..Default::default()
    });
    let Control::Shape(outer) = &mut doc.sections[0].paragraphs[0].controls[0] else {
        unreachable!()
    };
    let ShapeObject::Rectangle(outer) = outer.as_mut() else {
        unreachable!()
    };
    let Control::Table(table) =
        &mut outer.drawing.text_box.as_mut().unwrap().paragraphs[0].controls[0]
    else {
        unreachable!()
    };
    let para = &mut table.cells[0].paragraphs[0];
    para.controls = ["FIRST", "SECOND"]
        .into_iter()
        .map(|text| {
            Control::Shape(Box::new(rectangle_textbox(
                6000,
                2500,
                true,
                vec![paragraph(text)],
            )))
        })
        .collect();
    para.para_shape_id = 1;
    para.text = prefix.into();
    para.char_offsets = (0..prefix.len() as u32).collect();
    para.char_count = prefix.len() as u32 + 17;
    if multiline {
        para.line_segs = [0, 8]
            .into_iter()
            .enumerate()
            .map(|(i, start)| LineSeg {
                text_start: start,
                vertical_pos: i as i32 * 3000,
                line_height: 2500,
                text_height: 2500,
                baseline_distance: 2000,
                segment_width: 24000,
                ..Default::default()
            })
            .collect();
    }
    let mut core = DocumentCore::new_empty();
    core.set_document(doc);
    fn collect(node: &RenderNode, out: &mut Vec<(String, f64, f64)>) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if matches!(run.display_or_text(), "FIRST" | "SECOND") {
                assert_eq!(run.cell_context.as_ref().unwrap().path.len(), 3);
                out.push((run.display_or_text().into(), node.bbox.x, node.bbox.y));
            }
        }
        for child in &node.children {
            collect(child, out);
        }
    }
    let mut positions = Vec::new();
    collect(
        &core.build_page_render_tree(0).unwrap().root,
        &mut positions,
    );
    assert_eq!(positions.len(), 2);
    positions
}

#[test]
fn embedded_cell_inline_shapes_advance_on_same_line() {
    let positions = inline_pair_positions(false, "", Alignment::Left);
    assert!(positions[1].1 >= positions[0].1 + 79.9, "{positions:?}");
    assert!(
        (positions[1].2 - positions[0].2).abs() < 0.1,
        "{positions:?}"
    );
}

#[test]
fn embedded_cell_inline_shapes_follow_stored_lines() {
    let positions = inline_pair_positions(true, "", Alignment::Left);
    assert!(positions[1].2 >= positions[0].2 + 39.9, "{positions:?}");
    assert!(
        (positions[1].1 - positions[0].1).abs() < 0.1,
        "{positions:?}"
    );
}

#[test]
fn embedded_cell_inline_shapes_align_as_one_group() {
    for (alignment, expected_x) in [(Alignment::Center, 80.0), (Alignment::Right, 160.0)] {
        let positions = inline_pair_positions(false, "", alignment);
        assert!((positions[0].1 - expected_x).abs() < 0.1, "{positions:?}");
        assert!(
            (positions[1].1 - positions[0].1 - 80.0).abs() < 0.1,
            "{positions:?}"
        );
    }
}

#[test]
fn embedded_cell_inline_shapes_follow_preceding_text() {
    let plain = inline_pair_positions(false, "", Alignment::Left);
    let positions = inline_pair_positions(false, "PREFIX", Alignment::Left);
    assert!(positions[0].1 > plain[0].1 + 10.0, "{positions:?}");
    assert!(positions[1].1 >= positions[0].1 + 79.9, "{positions:?}");
}
