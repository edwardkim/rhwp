//! #6788: 속성 변경은 선택 안의 각 원본 글자 모양을 보존해야 한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::paragraph::{CharShapeRef, Paragraph};
use rhwp::model::style::CharShape;
use rhwp::wasm_api::HwpDocument;

const TEXT: &str = "가나다라마바";
const YELLOW: u32 = 0x0000_ffff;
const PURPLE: u32 = 0x00c0_20a0;
const HIGHLIGHT: &str = r##"{"shadeColor":"#ffff00"}"##;
const COLOR: &str = r##"{"textColor":"#a020c0"}"##;

fn blank() -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    doc
}

fn body(text: &str) -> HwpDocument {
    let mut doc = blank();
    doc.insert_text_native(0, 0, 0, text).unwrap();
    doc.apply_char_format_native(0, 0, 2, 4, COLOR).unwrap();
    doc
}

fn shapes(doc: &HwpDocument, para: &Paragraph) -> Vec<CharShape> {
    (0..para.text.chars().count())
        .map(|offset| {
            doc.document().doc_info.char_shapes[para.char_shape_id_at(offset).unwrap() as usize]
                .clone()
        })
        .collect()
}

fn body_shapes(doc: &HwpDocument) -> Vec<CharShape> {
    shapes(doc, &doc.document().sections[0].paragraphs[0])
}

fn assert_highlight_only(before: &[CharShape], after: &[CharShape], start: usize, end: usize) {
    assert_eq!(before.len(), after.len());
    for (offset, (old, actual)) in before.iter().zip(after).enumerate() {
        let mut expected = old.clone();
        if (start..end).contains(&offset) {
            expected.shade_color = YELLOW;
            expected.raw_data = None;
        }
        assert_eq!(
            *actual, expected,
            "offset {offset}: 미지정 속성/선택 밖 모양 보존"
        );
    }
}

#[test]
fn highlight_preserves_mixed_colors_and_reuses_repeated_shapes() {
    let mut doc = body(TEXT);
    let before = body_shapes(&doc);
    assert_eq!(
        before.iter().map(|s| s.text_color).collect::<Vec<_>>(),
        [0, 0, PURPLE, PURPLE, 0, 0]
    );
    let count = doc.document().doc_info.char_shapes.len();
    let lines = serde_json::to_value(&doc.document().sections[0].paragraphs[0].line_segs).unwrap();
    doc.apply_char_format_native(0, 0, 0, 6, HIGHLIGHT).unwrap();
    assert_highlight_only(&before, &body_shapes(&doc), 0, 6);
    assert_eq!(doc.document().doc_info.char_shapes.len(), count + 2);
    assert_eq!(
        serde_json::to_value(&doc.document().sections[0].paragraphs[0].line_segs).unwrap(),
        lines
    );
    doc.apply_char_format_native(0, 0, 0, 6, HIGHLIGHT).unwrap();
    assert_eq!(
        doc.document().doc_info.char_shapes.len(),
        count + 2,
        "반복 적용은 새 모양을 계속 만들면 안 된다"
    );
}

#[test]
fn highlight_preserves_font_size_bold_and_font_differences() {
    let mut doc = body(TEXT);
    let font_id = doc.find_or_create_font_id_native("Arial");
    assert!(font_id >= 0);
    let props =
        serde_json::json!({"fontId":font_id,"fontSize":1800,"bold":true,"italic":true}).to_string();
    doc.apply_char_format_native(0, 0, 2, 4, &props).unwrap();
    let before = body_shapes(&doc);
    assert_ne!(before[0].font_ids, before[2].font_ids);
    assert_ne!(before[0].base_size, before[2].base_size);
    assert!(!before[0].bold && before[2].bold);
    doc.apply_char_format_native(0, 0, 1, 5, HIGHLIGHT).unwrap();
    assert_highlight_only(&before, &body_shapes(&doc), 1, 5);
}

#[test]
fn partial_ranges_preserve_surrogates_control_offsets_and_outside_shapes() {
    let text = "가😀나다라마바";
    for (start, end) in [(0, 7), (1, 6), (2, 4), (3, 5), (0, 99)] {
        let mut doc = body(text);
        let para = &doc.document().sections[0].paragraphs[0];
        assert!(para.char_offsets[0] > 0, "실제 blank2010의 선행 컨트롤 축");
        assert_eq!(
            para.char_offsets[2] - para.char_offsets[1],
            2,
            "보조 평면 문자 폭"
        );
        let offsets = para.char_offsets.clone();
        let before = body_shapes(&doc);
        doc.apply_char_format_native(0, 0, start, end, HIGHLIGHT)
            .unwrap();
        assert_highlight_only(&before, &body_shapes(&doc), start, end);
        assert_eq!(
            doc.document().sections[0].paragraphs[0].char_offsets,
            offsets
        );
    }
}

#[test]
fn empty_and_out_of_bounds_ranges_do_not_create_shapes_or_change_refs() {
    let mut doc = body(TEXT);
    let before =
        serde_json::to_value(&doc.document().sections[0].paragraphs[0].char_shapes).unwrap();
    let count = doc.document().doc_info.char_shapes.len();
    for (start, end) in [(0, 0), (4, 2), (6, 6), (99, 100)] {
        doc.apply_char_format_native(0, 0, start, end, HIGHLIGHT)
            .unwrap();
        assert_eq!(doc.document().doc_info.char_shapes.len(), count);
        assert_eq!(
            serde_json::to_value(&doc.document().sections[0].paragraphs[0].char_shapes).unwrap(),
            before
        );
    }
}

#[test]
fn direct_shape_id_replacement_still_unifies_the_selected_range() {
    let mut doc = body(TEXT);
    let para = &mut doc.document_mut().sections[0].paragraphs[0];
    let purple_id = para.char_shape_id_at(2).unwrap();
    let black_id = para.char_shape_id_at(0).unwrap();
    para.apply_char_shape_range(1, 5, purple_id);
    for offset in 0..6 {
        assert_eq!(
            para.char_shape_id_at(offset),
            Some(if (1..5).contains(&offset) {
                purple_id
            } else {
                black_id
            })
        );
    }
}

#[test]
fn terminal_shape_boundary_survives_highlighting_to_text_end() {
    let mut doc = body(TEXT);
    let para = &mut doc.document_mut().sections[0].paragraphs[0];
    let terminal_id = para.char_shape_id_at(2).unwrap();
    let end = para.char_offsets.last().unwrap() + 1;
    para.char_shapes.push(CharShapeRef {
        start_pos: end,
        char_shape_id: terminal_id,
    });
    let before = body_shapes(&doc);
    doc.apply_char_format_native(0, 0, 0, 6, HIGHLIGHT).unwrap();
    assert_highlight_only(&before, &body_shapes(&doc), 0, 6);
    assert_eq!(
        doc.document().sections[0].paragraphs[0].char_shape_id_at(6),
        Some(terminal_id)
    );
}

#[test]
fn bold_and_font_size_changes_preserve_other_mixed_properties() {
    for props in [r#"{"bold":true}"#, r#"{"fontSize":2000}"#] {
        let mut doc = body(TEXT);
        let before = body_shapes(&doc);
        doc.apply_char_format_native(0, 0, 0, 6, props).unwrap();
        for (mut expected, actual) in before.into_iter().zip(body_shapes(&doc)) {
            expected.raw_data = None;
            if props.contains("bold") {
                expected.bold = true;
            } else {
                expected.base_size = 2000;
            }
            assert_eq!(actual, expected);
        }
    }
}

fn table_target(doc: &HwpDocument) -> (usize, usize) {
    doc.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .find_map(|(p, para)| {
            para.controls
                .iter()
                .position(|c| matches!(c, Control::Table(_)))
                .map(|c| (p, c))
        })
        .unwrap()
}

fn cell(doc: &HwpDocument, p: usize, c: usize, nested: bool) -> &Paragraph {
    let Control::Table(table) = &doc.document().sections[0].paragraphs[p].controls[c] else {
        panic!("table")
    };
    let para = &table.cells[0].paragraphs[0];
    if !nested {
        return para;
    }
    let Control::Table(inner) = &para.controls[0] else {
        panic!("nested table")
    };
    &inner.cells[0].paragraphs[0]
}

fn table_doc(nested: bool) -> (HwpDocument, usize, usize) {
    let mut doc = blank();
    doc.create_table_native(0, 0, 0, 1, 2).unwrap();
    let (p, c) = table_target(&doc);
    doc.insert_text_in_cell_native(0, p, c, 0, 0, 0, TEXT)
        .unwrap();
    doc.apply_char_format_in_cell_native(0, p, c, 0, 0, 2, 4, COLOR)
        .unwrap();
    if nested {
        let mut ir = doc.document().clone();
        let Control::Table(table) = &mut ir.sections[0].paragraphs[p].controls[c] else {
            panic!("table")
        };
        let inner = table.clone();
        table.cells[0].paragraphs[0]
            .controls
            .push(Control::Table(inner));
        doc.set_document(ir);
    }
    (doc, p, c)
}

#[test]
fn flat_and_by_path_cell_formatting_preserve_mixed_runs() {
    for by_path in [false, true] {
        let (mut doc, p, c) = table_doc(false);
        let before = shapes(&doc, cell(&doc, p, c, false));
        if by_path {
            doc.apply_char_format_in_cell_by_path(0, p, &[(c, 0, 0)], 1, 5, HIGHLIGHT)
                .unwrap();
        } else {
            doc.apply_char_format_in_cell_native(0, p, c, 0, 0, 1, 5, HIGHLIGHT)
                .unwrap();
        }
        assert_highlight_only(&before, &shapes(&doc, cell(&doc, p, c, false)), 1, 5);
    }
}

#[test]
fn nested_cell_formatting_preserves_runs_and_outer_cell() {
    let (mut doc, p, c) = table_doc(true);
    let before = shapes(&doc, cell(&doc, p, c, true));
    let outer = shapes(&doc, cell(&doc, p, c, false));
    doc.apply_char_format_in_cell_by_path(0, p, &[(c, 0, 0), (0, 0, 0)], 0, 6, HIGHLIGHT)
        .unwrap();
    assert_highlight_only(&before, &shapes(&doc, cell(&doc, p, c, true)), 0, 6);
    assert_eq!(shapes(&doc, cell(&doc, p, c, false)), outer);
}

fn hf(doc: &HwpDocument, is_header: bool) -> &Paragraph {
    doc.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|p| &p.controls)
        .find_map(|c| match c {
            Control::Header(h) if is_header => h.paragraphs.first(),
            Control::Footer(f) if !is_header => f.paragraphs.first(),
            _ => None,
        })
        .unwrap()
}

#[test]
fn header_and_footer_formatting_preserve_mixed_runs() {
    for is_header in [true, false] {
        let mut doc = blank();
        doc.create_header_footer_native(0, is_header, 0).unwrap();
        doc.insert_text_in_header_footer_native(0, is_header, 0, 0, 0, TEXT)
            .unwrap();
        doc.apply_char_format_in_header_footer_native(0, is_header, 0, 0, 2, 0, 4, COLOR)
            .unwrap();
        let before = shapes(&doc, hf(&doc, is_header));
        doc.apply_char_format_in_header_footer_native(0, is_header, 0, 0, 1, 0, 5, HIGHLIGHT)
            .unwrap();
        assert_highlight_only(&before, &shapes(&doc, hf(&doc, is_header)), 1, 5);
    }
}

#[test]
fn hwp_and_hwpx_roundtrips_keep_mixed_colors_and_highlight() {
    let mut doc = body(TEXT);
    doc.apply_char_format_native(0, 0, 0, 6, HIGHLIGHT).unwrap();
    for bytes in [
        doc.export_hwp_native().unwrap(),
        doc.export_hwpx_native().unwrap(),
    ] {
        let reopened = HwpDocument::from_bytes(&bytes).unwrap();
        let colors: Vec<_> = body_shapes(&reopened)
            .iter()
            .map(|s| (s.text_color, s.shade_color))
            .collect();
        assert_eq!(colors, [0, 0, PURPLE, PURPLE, 0, 0].map(|c| (c, YELLOW)));
    }
}
