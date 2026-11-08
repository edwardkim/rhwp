//! 표 셀 안에 둔 쪽 번호 위치 컨트롤도 쪽 번호를 켠다.
//! 보도자료 제목표처럼 첫 문단 표의 셀에 쪽 번호 위치를 넣은 문서에서 쪽 번호가 빠졌다.
//! 공개 합성 IR이다.
use rhwp::model::control::{Control, PageNumberPos};
use rhwp::model::document::{Document, Section, SectionDef};
use rhwp::model::page::PageDef;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::{CommonObjAttr, TextWrap};
use rhwp::model::style::ParaShape;
use rhwp::model::table::{Cell, Table};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn fixture() -> Document {
    let cell_para = Paragraph {
        char_count: 9,
        controls: vec![Control::PageNumberPos(PageNumberPos {
            position: 5,
            prefix_char: '-',
            suffix_char: '-',
            dash_char: '-',
            ..Default::default()
        })],
        ..Default::default()
    };
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
            paragraphs: vec![cell_para],
            ..Default::default()
        }],
        ..Default::default()
    };
    let mut doc = Document::default();
    doc.doc_info.para_shapes = vec![ParaShape::default()];
    doc.sections.push(Section {
        paragraphs: vec![
            Paragraph {
                controls: vec![Control::Table(Box::new(table))],
                char_count: 9,
                ..Default::default()
            },
            Paragraph {
                text: "BODY".into(),
                char_count: 5,
                char_offsets: vec![0, 1, 2, 3],
                ..Default::default()
            },
        ],
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

fn text(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(run.display_or_text());
    }
    for child in &node.children {
        text(child, out);
    }
}

#[test]
fn page_number_pos_inside_table_cell_paints_page_number() {
    let mut core = DocumentCore::new_empty();
    core.set_document(fixture());
    let mut painted = String::new();
    text(&core.build_page_render_tree(0).unwrap().root, &mut painted);
    let painted: String = painted.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(painted.contains("-1-"), "쪽 번호 누락: {painted}");
}

#[test]
fn legacy_paginator_reads_page_number_pos_inside_table_cell() {
    use rhwp::renderer::style_resolver::resolve_styles;
    use rhwp::renderer::{composer::compose_section, pagination::Paginator};
    let doc = fixture();
    let section = &doc.sections[0];
    let (result, _) = Paginator::new(96.0).paginate(
        &section.paragraphs,
        &compose_section(section),
        &resolve_styles(&doc.doc_info, 96.0),
        &section.section_def.page_def,
        &Default::default(),
        0,
    );
    assert!(result.pages[0].page_number_pos.is_some());
}
