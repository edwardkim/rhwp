use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section, SectionDef};
use rhwp::model::page::PageDef;
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::model::table::{Cell, Table};
use rhwp::model::Padding;
use rhwp::renderer::style_resolver::ResolvedCharStyle;
use rhwp::wasm_api::HwpDocument;

fn make_char_offsets(text: &str) -> Vec<u32> {
    let mut offsets = Vec::new();
    let mut pos = 0;
    for ch in text.chars() {
        offsets.push(pos);
        pos += if (ch as u32) > 0xFFFF { 2 } else { 1 };
    }
    offsets
}

fn create_doc_with_table() -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    let mut document = Document::default();
    let cell_padding = Padding {
        left: 100,
        right: 100,
        top: 100,
        bottom: 100,
    };
    let make_cell = |col, row, text: &str| Cell {
        col,
        row,
        col_span: 1,
        row_span: 1,
        width: 21000,
        height: 3000,
        padding: cell_padding,
        paragraphs: vec![Paragraph {
            text: text.to_string(),
            char_count: text.chars().count() as u32,
            char_offsets: make_char_offsets(text),
            line_segs: vec![LineSeg {
                line_height: 400,
                baseline_distance: 320,
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    };
    let table = Table {
        row_count: 1,
        col_count: 1,
        padding: cell_padding,
        cells: vec![make_cell(0, 0, "셀A")],
        ..Default::default()
    };
    document.sections.push(Section {
        section_def: SectionDef {
            page_def: PageDef {
                width: 59528,
                height: 84188,
                margin_left: 8504,
                margin_right: 8504,
                margin_top: 5669,
                margin_bottom: 4252,
                margin_header: 4252,
                margin_footer: 4252,
                ..Default::default()
            },
            ..Default::default()
        },
        paragraphs: vec![Paragraph {
            controls: vec![Control::Table(Box::new(table))],
            line_segs: vec![LineSeg {
                line_height: 400,
                baseline_distance: 320,
                ..Default::default()
            }],
            ..Default::default()
        }],
        raw_stream: None,
    });
    doc.set_document(document);
    doc
}

#[test]
fn replace_all_reflows_table_cell_line_segments() {
    let mut doc = create_doc_with_table();
    let replacement = "근로계약서본문셀내용 ".repeat(35);
    let result = doc
        .replace_all_native("셀A", &replacement, true)
        .expect("replace all");
    assert!(result.contains("\"count\":1"));

    let Control::Table(table) = &doc.document().sections[0].paragraphs[0].controls[0] else {
        panic!("table control not found");
    };
    let cell_para = &table.cells[0].paragraphs[0];
    assert_eq!(cell_para.text, replacement);
    assert!(
        cell_para.line_segs.len() > 1,
        "replace_all_native must reflow wrapped table cell text; got {:?}",
        cell_para.line_segs
    );
    assert!(
        doc.render_page_svg_native(0).is_ok(),
        "render after cell replaceAll must stay valid"
    );
}

#[test]
fn number_only_runs_use_body_font_when_generic_differs() {
    let serif_body_sans_latin = ResolvedCharStyle {
        font_family: "함초롬바탕".to_string(),
        font_families: vec!["함초롬바탕".to_string(), "Arial".to_string()],
        ..Default::default()
    };
    assert_eq!(
        serif_body_sans_latin.font_family_for_run(1, "123,456"),
        "함초롬바탕"
    );
    assert_eq!(
        serif_body_sans_latin.font_family_for_run(1, "ABC123"),
        "Arial"
    );

    let sans_body_sans_latin = ResolvedCharStyle {
        font_family: "함초롬돋움".to_string(),
        font_families: vec!["함초롬돋움".to_string(), "Arial".to_string()],
        ..Default::default()
    };
    assert_eq!(
        sans_body_sans_latin.font_family_for_run(1, "123,456"),
        "Arial"
    );
}
