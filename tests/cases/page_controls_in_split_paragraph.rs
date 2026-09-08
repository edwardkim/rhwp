//! 쪽을 가로지르는 문단의 감추기와 새 번호는 각 컨트롤의 소스 줄에 적용한다.
//! 실제 사용자 문서의 문자열이나 객체를 포함하지 않는 공개 합성 IR이다.
use rhwp::model::control::{AutoNumberType, Control, NewNumber, PageHide, PageNumberPos};
use rhwp::model::document::{Document, Section, SectionDef};
use rhwp::model::page::PageDef;
use rhwp::model::paragraph::{ColumnBreakType, LineSeg, Paragraph};
use rhwp::model::shape::{CommonObjAttr, RectangleShape, ShapeObject, TextWrap};
use rhwp::model::style::ParaShape;
use rhwp::model::table::{Cell, Table};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn line(start: u32, y: i32, height: i32) -> LineSeg {
    LineSeg {
        text_start: start,
        vertical_pos: y,
        line_height: height,
        text_height: height,
        baseline_distance: height * 4 / 5,
        line_spacing: 0,
        segment_width: 40_000,
        tag: 393216,
        ..Default::default()
    }
}

fn fixture(hide_after_shapes: bool) -> Document {
    let mut paragraphs: Vec<_> = (0..20)
        .map(|i| Paragraph {
            char_count: 1,
            // Keep the fallback paginator's prefix safely below the page-tail
            // boundary. The stored vpos ladder still places the following
            // control paragraph near the source page bottom, while small font
            // metric changes cannot turn this compatibility assertion into an
            // unrelated overflow test.
            line_segs: vec![line(0, i * 2_500, 2_000)],
            ..Default::default()
        })
        .collect();
    paragraphs[0]
        .controls
        .push(Control::PageNumberPos(PageNumberPos {
            position: 5,
            prefix_char: '-',
            suffix_char: '-',
            dash_char: '-',
            ..Default::default()
        }));
    paragraphs[0].char_count = 9;
    let table = Table {
        row_count: 1,
        col_count: 1,
        common: CommonObjAttr {
            width: 20_000,
            height: 4_000,
            treat_as_char: true,
            text_wrap: TextWrap::TopAndBottom,
            ..Default::default()
        },
        cells: vec![Cell {
            row_span: 1,
            col_span: 1,
            width: 20_000,
            height: 4_000,
            paragraphs: vec![Paragraph::default()],
            ..Default::default()
        }],
        ..Default::default()
    };
    let shape = || {
        Control::Shape(Box::new(ShapeObject::Rectangle(RectangleShape {
            common: CommonObjAttr {
                width: 10_000,
                height: 2_000,
                treat_as_char: true,
                text_wrap: TextWrap::InFrontOfText,
                ..Default::default()
            },
            ..Default::default()
        })))
    };
    let hide = Control::PageHide(PageHide {
        hide_page_num: true,
        ..Default::default()
    });
    let mut controls = vec![Control::Table(Box::new(table)), shape(), shape()];
    let starts = if hide_after_shapes {
        controls.push(hide);
        [0, 8, 16]
    } else {
        controls.insert(0, hide);
        [0, 16, 24]
    };
    controls.push(Control::NewNumber(NewNumber {
        number_type: AutoNumberType::Page,
        number: 7,
    }));
    paragraphs.push(Paragraph {
        controls,
        char_count: 41,
        line_segs: vec![
            line(starts[0], 50_000, 4_000),
            line(starts[1], 0, 2_000),
            line(starts[2], 2_000, 2_000),
        ],
        ..Default::default()
    });
    paragraphs.push(Paragraph {
        text: "NEXT".into(),
        char_count: 5,
        char_offsets: vec![0, 1, 2, 3],
        column_type: ColumnBreakType::Page,
        line_segs: vec![line(0, 0, 1_000)],
        ..Default::default()
    });
    let mut doc = Document::default();
    doc.doc_info.para_shapes = vec![ParaShape::default()];
    doc.sections.push(Section {
        paragraphs,
        section_def: SectionDef {
            page_def: PageDef {
                width: 44_000,
                height: 60_000,
                margin_left: 2_000,
                margin_right: 2_000,
                margin_top: 2_000,
                margin_bottom: 2_000,
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    });
    doc
}

fn core(hide_after_shapes: bool) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.set_document(fixture(hide_after_shapes));
    assert_eq!(core.page_count(), 3, "{}", core.dump_page_items(None));
    core
}

fn page_number(core: &DocumentCore, page: u32) -> u64 {
    let info: serde_json::Value =
        serde_json::from_str(&core.get_page_info_native(page).unwrap()).unwrap();
    info["pageNumber"].as_u64().unwrap()
}

fn text(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(run.display_or_text());
    }
    for child in &node.children {
        text(child, out);
    }
}

fn painted_text(core: &DocumentCore, page: u32) -> String {
    let mut out = String::new();
    text(&core.build_page_render_tree(page).unwrap().root, &mut out);
    out.chars().filter(|c| !c.is_whitespace()).collect()
}

#[test]
fn restart_after_shapes_belongs_to_the_second_physical_page() {
    let core = core(false);
    assert_eq!(page_number(&core, 1), 7, "{}", core.dump_page_items(None));
    assert_eq!(page_number(&core, 2), 8);
}

#[test]
fn page_hide_before_table_does_not_hide_the_later_shapes_page() {
    let core = core(false);
    assert!(
        painted_text(&core, 1).contains("-7-"),
        "쪽 번호 누락: {}",
        core.dump_page_items(Some(1))
    );
}

#[test]
fn page_hide_after_shapes_only_hides_its_own_page() {
    let core = core(true);
    assert!(!painted_text(&core, 1).contains("-7-"));
    assert!(painted_text(&core, 2).contains("-8-"));
}

#[test]
fn legacy_paginator_preserves_unsplit_paragraph_controls() {
    use rhwp::renderer::style_resolver::resolve_styles;
    use rhwp::renderer::{composer::compose_section, pagination::Paginator};
    let doc = fixture(false);
    let section = &doc.sections[0];
    let (result, _) = Paginator::new(96.0).paginate(
        &section.paragraphs,
        &compose_section(section),
        &resolve_styles(&doc.doc_info, 96.0),
        &section.section_def.page_def,
        &Default::default(),
        0,
    );
    // 이 경로는 저장 vpos reset에서 이 문단을 나누지 않는다. 모든 컨트롤이
    // 같은 쪽에 있을 때 기존의 한 번 재시작/한 쪽 감추기 계약을 보존한다.
    assert_eq!(result.pages.len(), 2);
    assert_eq!(result.pages[0].page_number, 7);
    assert_eq!(result.pages[1].page_number, 8);
    assert!(result.pages[0].page_hide.as_ref().unwrap().hide_page_num);
    assert!(result.pages[1].page_hide.is_none());
}

#[test]
fn nested_table_restart_still_belongs_to_the_first_table_page() {
    let mut doc = fixture(false);
    let para = &mut doc.sections[0].paragraphs[20];
    let restart = para.controls.pop().unwrap();
    para.char_count -= 8;
    let Control::Table(table) = &mut para.controls[1] else {
        panic!("fixture table missing");
    };
    table.cells[0].paragraphs[0].controls.push(restart);
    let mut core = DocumentCore::new_empty();
    core.set_document(doc);
    assert_eq!(core.page_count(), 3);
    assert_eq!(page_number(&core, 0), 7);
    assert_eq!(page_number(&core, 1), 8);
    assert_eq!(page_number(&core, 2), 9);
}
