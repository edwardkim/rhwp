//! Issue #1692: HWP3 글자색 인덱스가 CharShape.text_color로 보존되는지 검증한다.

use rhwp::model::control::Control;
use rhwp::model::footnote::Endnote;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::style::{Alignment, HeadType};
use rhwp::parser::ole_container::is_hmapsi_ole_container;
use rhwp::parser::parse_document;
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;
use std::path::Path;

fn load(path: &str) -> rhwp::model::document::Document {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    parse_document(&bytes).unwrap_or_else(|e| panic!("parse {path}: {e:?}"))
}

fn load_wasm_doc(path: &str) -> HwpDocument {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse wasm {path}: {e:?}"))
}

fn collect_endnotes<'a>(paragraphs: &'a [Paragraph], out: &mut Vec<&'a Endnote>) {
    for paragraph in paragraphs {
        collect_endnotes_in_controls(&paragraph.controls, out);
    }
}

fn collect_endnotes_in_controls<'a>(controls: &'a [Control], out: &mut Vec<&'a Endnote>) {
    for control in controls {
        match control {
            Control::Endnote(endnote) => {
                out.push(endnote);
                collect_endnotes(&endnote.paragraphs, out);
            }
            Control::Footnote(footnote) => {
                collect_endnotes(&footnote.paragraphs, out);
            }
            Control::Table(table) => {
                for cell in &table.cells {
                    collect_endnotes(&cell.paragraphs, out);
                }
                if let Some(caption) = &table.caption {
                    collect_endnotes(&caption.paragraphs, out);
                }
            }
            Control::Picture(picture) => {
                if let Some(caption) = &picture.caption {
                    collect_endnotes(&caption.paragraphs, out);
                }
            }
            Control::Shape(shape) => {
                if let Some(drawing) = shape.drawing() {
                    if let Some(caption) = &drawing.caption {
                        collect_endnotes(&caption.paragraphs, out);
                    }
                    if let Some(text_box) = &drawing.text_box {
                        collect_endnotes(&text_box.paragraphs, out);
                    }
                }
            }
            Control::Header(header) => {
                collect_endnotes(&header.paragraphs, out);
            }
            Control::Footer(footer) => {
                collect_endnotes(&footer.paragraphs, out);
            }
            _ => {}
        }
    }
}

fn first_header_paragraph<'a>(
    doc: &'a rhwp::model::document::Document,
    needle: &str,
) -> &'a Paragraph {
    doc.sections[0]
        .paragraphs
        .iter()
        .flat_map(|paragraph| paragraph.controls.iter())
        .find_map(|control| match control {
            Control::Header(header) => header
                .paragraphs
                .iter()
                .find(|paragraph| paragraph.text.contains(needle)),
            _ => None,
        })
        .unwrap_or_else(|| panic!("header paragraph containing {needle}"))
}

fn page_render_tree(doc: &HwpDocument, page: u32) -> Value {
    let json = doc
        .get_page_render_tree(page)
        .unwrap_or_else(|err| panic!("page render tree {page}: {err:?}"));
    serde_json::from_str(&json).unwrap_or_else(|err| panic!("parse render tree {page}: {err}"))
}

fn text_width_in_tree(node: &Value, ancestor_type: &str, text: &str) -> Option<f64> {
    fn walk(node: &Value, ancestor_type: &str, text: &str, in_ancestor: bool) -> Option<f64> {
        let node_type = node.get("type").and_then(Value::as_str).unwrap_or("");
        let now_in_ancestor = in_ancestor || node_type == ancestor_type;
        if now_in_ancestor
            && node_type == "TextRun"
            && node.get("text").and_then(Value::as_str) == Some(text)
        {
            return node
                .get("bbox")
                .and_then(|bbox| bbox.get("w"))
                .and_then(Value::as_f64);
        }

        node.get("children")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .find_map(|child| walk(child, ancestor_type, text, now_in_ancestor))
    }

    walk(node, ancestor_type, text, false)
}

fn text_bbox_in_tree(node: &Value, text: &str) -> Option<(f64, f64, f64, f64)> {
    fn walk(node: &Value, text: &str) -> Option<(f64, f64, f64, f64)> {
        let node_type = node.get("type").and_then(Value::as_str).unwrap_or("");
        if node_type == "TextRun" && node.get("text").and_then(Value::as_str) == Some(text) {
            let bbox = node.get("bbox")?;
            return Some((
                bbox.get("x")?.as_f64()?,
                bbox.get("y")?.as_f64()?,
                bbox.get("w")?.as_f64()?,
                bbox.get("h")?.as_f64()?,
            ));
        }

        node.get("children")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .find_map(|child| walk(child, text))
    }

    walk(node, text)
}

fn text_bbox_containing_in_tree(node: &Value, needle: &str) -> Option<(f64, f64, f64, f64)> {
    fn walk(node: &Value, needle: &str) -> Option<(f64, f64, f64, f64)> {
        let node_type = node.get("type").and_then(Value::as_str).unwrap_or("");
        if node_type == "TextRun"
            && node
                .get("text")
                .and_then(Value::as_str)
                .is_some_and(|text| text.contains(needle))
        {
            let bbox = node.get("bbox")?;
            return Some((
                bbox.get("x")?.as_f64()?,
                bbox.get("y")?.as_f64()?,
                bbox.get("w")?.as_f64()?,
                bbox.get("h")?.as_f64()?,
            ));
        }

        node.get("children")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .find_map(|child| walk(child, needle))
    }

    walk(node, needle)
}

fn first_bbox_by_type(node: &Value, node_type: &str) -> Option<(f64, f64, f64, f64)> {
    fn walk(node: &Value, needle_type: &str) -> Option<(f64, f64, f64, f64)> {
        if node.get("type").and_then(Value::as_str) == Some(needle_type) {
            let bbox = node.get("bbox")?;
            return Some((
                bbox.get("x")?.as_f64()?,
                bbox.get("y")?.as_f64()?,
                bbox.get("w")?.as_f64()?,
                bbox.get("h")?.as_f64()?,
            ));
        }

        node.get("children")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .find_map(|child| walk(child, needle_type))
    }

    walk(node, node_type)
}

fn text_concat_in_tree(node: &Value, ancestor_type: &str) -> String {
    fn walk(node: &Value, ancestor_type: &str, in_ancestor: bool, out: &mut String) {
        let node_type = node.get("type").and_then(Value::as_str).unwrap_or("");
        let now_in_ancestor = in_ancestor || node_type == ancestor_type;
        if now_in_ancestor && node_type == "TextRun" {
            if let Some(text) = node.get("text").and_then(Value::as_str) {
                out.push_str(text);
            }
        }

        if let Some(children) = node.get("children").and_then(Value::as_array) {
            for child in children {
                walk(child, ancestor_type, now_in_ancestor, out);
            }
        }
    }

    let mut out = String::new();
    walk(node, ancestor_type, false, &mut out);
    out
}

fn parse_leading_note_marker(text: &str) -> Option<u32> {
    let trimmed = text.trim_start();
    let digit_len = trimmed
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .map(char::len_utf8)
        .sum::<usize>();
    if digit_len == 0 {
        return None;
    }

    let rest = &trimmed[digit_len..];
    let after_marker = rest.strip_prefix(')')?;
    if after_marker
        .chars()
        .next()
        .is_some_and(|ch| !ch.is_whitespace())
    {
        return None;
    }

    trimmed[..digit_len].parse().ok()
}

fn endnote_marker_numbers_on_page(node: &Value) -> Vec<u32> {
    fn walk(node: &Value, out: &mut Vec<u32>) {
        if node.get("type").and_then(Value::as_str) == Some("TextRun") {
            let x = node
                .get("bbox")
                .and_then(|bbox| bbox.get("x"))
                .and_then(Value::as_f64);
            let h = node
                .get("bbox")
                .and_then(|bbox| bbox.get("h"))
                .and_then(Value::as_f64);
            let at_note_column_start =
                x.is_some_and(|x| (113.4..=133.4).contains(&x) || (406.3..=426.3).contains(&x));
            let note_marker_sized = h.is_some_and(|h| h <= 14.0);
            if at_note_column_start && note_marker_sized {
                if let Some(text) = node.get("text").and_then(Value::as_str) {
                    if let Some(number) = parse_leading_note_marker(text) {
                        if out.last().copied() != Some(number) {
                            out.push(number);
                        }
                    }
                }
            }
        }

        if let Some(children) = node.get("children").and_then(Value::as_array) {
            for child in children {
                walk(child, out);
            }
        }
    }

    let mut out = Vec::new();
    walk(node, &mut out);
    out
}

fn contains_node_type(node: &Value, node_type: &str) -> bool {
    if node.get("type").and_then(Value::as_str) == Some(node_type) {
        return true;
    }

    node.get("children")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|child| contains_node_type(child, node_type))
}

fn contains_node_type_under(node: &Value, ancestor_type: &str, node_type: &str) -> bool {
    fn walk(node: &Value, ancestor_type: &str, node_type: &str, in_ancestor: bool) -> bool {
        let current_type = node.get("type").and_then(Value::as_str).unwrap_or("");
        let now_in_ancestor = in_ancestor || current_type == ancestor_type;
        if now_in_ancestor && current_type == node_type {
            return true;
        }

        node.get("children")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .any(|child| walk(child, ancestor_type, node_type, now_in_ancestor))
    }

    walk(node, ancestor_type, node_type, false)
}

fn first_picture<'a>(paragraphs: &'a [Paragraph]) -> Option<&'a rhwp::model::image::Picture> {
    for paragraph in paragraphs {
        for control in &paragraph.controls {
            match control {
                Control::Picture(picture) => return Some(picture),
                Control::Table(table) => {
                    for cell in &table.cells {
                        if let Some(picture) = first_picture(&cell.paragraphs) {
                            return Some(picture);
                        }
                    }
                    if let Some(caption) = &table.caption {
                        if let Some(picture) = first_picture(&caption.paragraphs) {
                            return Some(picture);
                        }
                    }
                }
                Control::Header(header) => {
                    if let Some(picture) = first_picture(&header.paragraphs) {
                        return Some(picture);
                    }
                }
                Control::Footer(footer) => {
                    if let Some(picture) = first_picture(&footer.paragraphs) {
                        return Some(picture);
                    }
                }
                Control::Shape(shape) => {
                    if let Some(drawing) = shape.drawing() {
                        if let Some(text_box) = &drawing.text_box {
                            if let Some(picture) = first_picture(&text_box.paragraphs) {
                                return Some(picture);
                            }
                        }
                        if let Some(caption) = &drawing.caption {
                            if let Some(picture) = first_picture(&caption.paragraphs) {
                                return Some(picture);
                            }
                        }
                    }
                }
                Control::Endnote(endnote) => {
                    if let Some(picture) = first_picture(&endnote.paragraphs) {
                        return Some(picture);
                    }
                }
                Control::Footnote(footnote) => {
                    if let Some(picture) = first_picture(&footnote.paragraphs) {
                        return Some(picture);
                    }
                }
                _ => {}
            }
        }
    }

    None
}

#[test]
fn issue_1692_so_sueop_answer_endnote_pages_match_pdf_ranges() {
    let cases = [
        ("HWP3", load_wasm_doc("samples/SO-SUEOP.hwp")),
        ("HWPX", load_wasm_doc("samples/SO-SUEOP.hwpx")),
    ];
    let expected = [
        (42, 43, 1, 58),
        (43, 44, 59, 129),
        (44, 45, 130, 191),
        (45, 46, 192, 223),
    ];

    for (label, doc) in cases {
        for (page_index, page_number, first, last) in expected {
            let tree = page_render_tree(&doc, page_index);
            let markers = endnote_marker_numbers_on_page(&tree);
            assert_eq!(
                markers.first().copied(),
                Some(first),
                "{label} page {page_number} first endnote marker"
            );
            assert_eq!(
                markers.last().copied(),
                Some(last),
                "{label} page {page_number} last endnote marker"
            );
            assert_eq!(
                markers.len() as u32,
                last - first + 1,
                "{label} page {page_number} endnote marker count: {markers:?}"
            );
        }
    }
}

#[test]
fn issue_1692_so_sueop_hwp3_preserves_blue_text_color_like_hwpx_reference() {
    let hwp3_doc = load("samples/SO-SUEOP.hwp");
    let hwpx_doc = load("samples/SO-SUEOP.hwpx");

    let blue = 0x00FF0000;
    let hwp3_blue_count = hwp3_doc
        .doc_info
        .char_shapes
        .iter()
        .filter(|cs| cs.text_color == blue)
        .count();
    let hwpx_blue_count = hwpx_doc
        .doc_info
        .char_shapes
        .iter()
        .filter(|cs| cs.text_color == blue)
        .count();

    assert!(
        hwp3_blue_count > 0,
        "SO-SUEOP.hwp must preserve HWP3 blue text_color into CharShape.text_color"
    );
    assert!(
        hwpx_blue_count > 0,
        "SO-SUEOP.hwpx reference must contain blue CharShape.text_color"
    );
    assert!(
        hwp3_doc
            .doc_info
            .char_shapes
            .iter()
            .any(|cs| cs.text_color == 0),
        "existing black text CharShape must remain available"
    );
}

#[test]
fn issue_1692_so_sueop_hwp3_line_box_reflects_para_margins() {
    let hwp3_doc = load("samples/SO-SUEOP.hwp");
    let section = &hwp3_doc.sections[0];

    let para_57 = &section.paragraphs[57];
    let ps_57 = &hwp3_doc.doc_info.para_shapes[para_57.para_shape_id as usize];
    assert_eq!(
        (ps_57.margin_left, ps_57.margin_right, ps_57.indent),
        (2000, 1000, 1000),
        "paragraph 57 ParaShape must use the common HWP5/HWPX IR scale"
    );

    let assert_line_box = |para_idx: usize, expected_start: i32, expected_width: i32| {
        let seg = section.paragraphs[para_idx]
            .line_segs
            .first()
            .unwrap_or_else(|| panic!("paragraph {para_idx} must have a first line segment"));
        assert_eq!(
            (seg.column_start, seg.segment_width),
            (expected_start, expected_width),
            "paragraph {para_idx} line box"
        );
    };

    assert_line_box(57, 1000, 41020);
    assert_line_box(77, 3000, 39520);
    assert_line_box(1000, 1000, 40520);
}

#[test]
fn issue_1692_so_sueop_hwp3_endnotes_follow_hwpx_numbering_and_width() {
    let hwp3_doc = load("samples/SO-SUEOP.hwp");
    let hwpx_doc = load("samples/SO-SUEOP.hwpx");

    let mut hwp3_endnotes = Vec::new();
    let mut hwpx_endnotes = Vec::new();
    collect_endnotes(&hwp3_doc.sections[0].paragraphs, &mut hwp3_endnotes);
    collect_endnotes(&hwpx_doc.sections[0].paragraphs, &mut hwpx_endnotes);

    assert_eq!(hwp3_endnotes.len(), hwpx_endnotes.len());
    assert_eq!(hwp3_endnotes.len(), 223);
    assert_eq!(hwp3_endnotes.first().unwrap().number, 1);
    assert_eq!(hwp3_endnotes.last().unwrap().number, 223);
    assert!(
        hwp3_endnotes
            .iter()
            .all(|endnote| endnote.after_decoration_letter == ')' as u16),
        "HWP3 endnote markers must use the same ')' suffix as the HWPX reference"
    );

    let hwp3_initial_column = hwp3_doc.sections[0].paragraphs[0]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::ColumnDef(column_def) => Some(column_def),
            _ => None,
        })
        .expect("HWP3 section must restore the initial one-column body definition");
    assert_eq!(hwp3_initial_column.column_count, 1);

    let hwp3_first_seg = hwp3_endnotes[0].paragraphs[0]
        .line_segs
        .first()
        .expect("HWP3 first endnote paragraph line segment");
    let hwpx_first_seg = hwpx_endnotes[0].paragraphs[0]
        .line_segs
        .first()
        .expect("HWPX first endnote paragraph line segment");
    assert_eq!(
        hwp3_first_seg.segment_width, hwpx_first_seg.segment_width,
        "HWP3 endnote paragraph width must match the HWPX two-column note width"
    );

    let hwp3_shape = &hwp3_doc.sections[0].section_def.endnote_shape;
    let hwpx_shape = &hwpx_doc.sections[0].section_def.endnote_shape;
    assert_eq!(hwp3_shape.suffix_char, hwpx_shape.suffix_char);
    assert_eq!(
        hwp3_shape.separator_margin_top,
        hwpx_shape.separator_margin_top
    );
    assert_eq!(hwp3_shape.note_spacing, hwpx_shape.note_spacing);
    assert_eq!(
        hwp3_shape.separator_line_width,
        hwpx_shape.separator_line_width
    );

    let hwp3_answer = hwp3_doc.sections[0]
        .paragraphs
        .iter()
        .rev()
        .find(|paragraph| paragraph.text.contains("해답"))
        .expect("HWP3 answer heading paragraph");
    let hwpx_answer = hwpx_doc.sections[0]
        .paragraphs
        .iter()
        .rev()
        .find(|paragraph| paragraph.text.contains("해답"))
        .expect("HWPX answer heading paragraph");
    let hwp3_column = hwp3_answer
        .controls
        .iter()
        .find_map(|control| match control {
            Control::ColumnDef(column_def) => Some(column_def),
            _ => None,
        })
        .expect("HWP3 answer heading must restore the two-column note zone");
    let hwpx_column = hwpx_answer
        .controls
        .iter()
        .find_map(|control| match control {
            Control::ColumnDef(column_def) => Some(column_def),
            _ => None,
        })
        .expect("HWPX answer heading column definition");
    assert_eq!(hwp3_column.column_count, hwpx_column.column_count);
    assert_eq!(hwp3_column.spacing, hwpx_column.spacing);
    assert_eq!(
        hwp3_answer.line_segs[0].segment_width,
        hwpx_answer.line_segs[0].segment_width
    );

    assert_eq!(hwp3_answer.text, hwpx_answer.text);
    assert!(!hwp3_answer.text.starts_with('-'));
    assert!(!hwp3_answer.text.contains('\u{FFFC}'));

    let hwp3_answer_shape = &hwp3_doc.doc_info.para_shapes[hwp3_answer.para_shape_id as usize];
    let hwpx_answer_shape = &hwpx_doc.doc_info.para_shapes[hwpx_answer.para_shape_id as usize];
    assert_eq!(hwp3_answer_shape.head_type, HeadType::Number);
    assert_eq!(
        hwp3_answer_shape.numbering_id,
        hwpx_answer_shape.numbering_id
    );
    assert_eq!(hwp3_answer_shape.para_level, hwpx_answer_shape.para_level);
    assert_eq!(hwp3_doc.doc_info.numberings[0].level_formats[0], "^1.");
    assert_eq!(hwp3_doc.doc_info.numberings[0].level_formats[1], "^2)");
    assert_eq!(hwp3_doc.doc_info.numberings[0].level_formats[2], "(^3)");
}

#[test]
fn issue_1692_so_sueop_header_footer_page5_matches_reference_contract() {
    let hwp3_model = load("samples/SO-SUEOP.hwp");
    let hwpx_model = load("samples/SO-SUEOP.hwpx");

    let hwp3_header = first_header_paragraph(&hwp3_model, "수업용소설해설");
    let hwpx_header = first_header_paragraph(&hwpx_model, "수업용소설해설");
    assert_eq!(
        hwp3_model.doc_info.para_shapes[hwp3_header.para_shape_id as usize].alignment,
        Alignment::Justify,
        "HWP3 원본 머리말은 단일 줄 Justify이며 렌더 단계에서 머리말 폭으로 분배해야 한다"
    );
    assert_eq!(
        hwpx_model.doc_info.para_shapes[hwpx_header.para_shape_id as usize].alignment,
        Alignment::Justify,
        "HWPX DISTRIBUTE_SPACE 머리말 문단은 공백 기반 Justify로 파싱되어야 한다"
    );

    let hwp3_doc = load_wasm_doc("samples/SO-SUEOP.hwp");
    let hwpx_doc = load_wasm_doc("samples/SO-SUEOP.hwpx");
    assert_eq!(hwp3_doc.page_count(), 46);
    assert_eq!(hwpx_doc.page_count(), 46);

    let hwp3_tree = page_render_tree(&hwp3_doc, 4);
    let hwpx_tree = page_render_tree(&hwpx_doc, 4);

    for tree in [&hwp3_tree, &hwpx_tree] {
        assert!(
            contains_node_type_under(tree, "Header", "Path")
                || contains_node_type_under(tree, "Header", "Line"),
            "SO-SUEOP header underline must render"
        );

        let footer_text = text_concat_in_tree(tree, "Footer");
        assert!(
            footer_text.contains("협성고등학교"),
            "page 5 footer school label must render"
        );
        assert!(
            footer_text.contains('5'),
            "page 5 footer AutoNumber(Page) must render the current page number"
        );
    }

    let hwp3_header_width = text_width_in_tree(&hwp3_tree, "Header", "수업용소설해설 박전현선생")
        .expect("HWP3 page 5 distributed header text width");
    assert!(
        hwp3_header_width > 500.0,
        "HWP3 justified header should span the header width, got {hwp3_header_width}"
    );

    let hwpx_header_width = text_width_in_tree(&hwpx_tree, "Header", "수업용소설해설 박전현선생")
        .expect("HWPX page 5 distributed header text width");
    assert!(
        hwpx_header_width > 500.0,
        "HWPX distributed header should span the header width, got {hwpx_header_width}"
    );
}

#[test]
fn issue_1692_so_sueop_hwpx_title_ole_renders_from_embedded_preview() {
    let hwpx_model = load("samples/SO-SUEOP.hwpx");
    let ole_content = hwpx_model
        .bin_data_content
        .first()
        .expect("SO-SUEOP HWPX must load ole1.ole as BinData #1");
    assert!(
        is_hmapsi_ole_container(&ole_content.data),
        "SO-SUEOP title OLE must be identified as HMapsi fallback content"
    );

    let hwpx_doc = load_wasm_doc("samples/SO-SUEOP.hwpx");
    let tree = page_render_tree(&hwpx_doc, 0);
    assert!(
        contains_node_type(&tree, "RawSvg") || contains_node_type(&tree, "Image"),
        "SO-SUEOP page 1 title OLE must render as image-like content"
    );
    assert!(
        !contains_node_type(&tree, "Placeholder"),
        "SO-SUEOP page 1 title OLE must not fall back to Placeholder"
    );
}

#[test]
fn issue_1692_so_sueop_hwp3_title_external_link_renders_from_sample_dir() {
    let hwp3_model = load("samples/SO-SUEOP.hwp");
    let picture = first_picture(&hwp3_model.sections[0].paragraphs)
        .expect("SO-SUEOP HWP3 page 1 title picture");
    assert_eq!(
        picture.image_attr.external_path.as_deref(),
        Some("00000000.OOO"),
        "HWP3 title picture must keep the linked external object basename"
    );

    let mut hwp3_doc = load_wasm_doc("samples/SO-SUEOP.hwp");
    let loaded = hwp3_doc.populate_external_images_from_dir(Path::new("samples"));
    assert!(
        loaded > 0,
        "samples/00000000.OOO must be loaded as the HWP3 title image"
    );

    let tree = page_render_tree(&hwp3_doc, 0);
    assert!(
        contains_node_type(&tree, "Image"),
        "SO-SUEOP HWP3 page 1 title must render as an image after external file loading"
    );
    assert!(
        !contains_node_type(&tree, "Placeholder"),
        "SO-SUEOP HWP3 page 1 title must not fall back to Placeholder"
    );
}

#[test]
fn issue_1692_so_sueop_hwp3_page1_school_label_matches_hwpx_y() {
    let hwp3_doc = load_wasm_doc("samples/SO-SUEOP.hwp");
    let hwpx_doc = load_wasm_doc("samples/SO-SUEOP.hwpx");

    let hwp3_tree = page_render_tree(&hwp3_doc, 0);
    let hwpx_tree = page_render_tree(&hwpx_doc, 0);
    let hwp3_school =
        text_bbox_in_tree(&hwp3_tree, " 협성고등학교").expect("HWP3 page 1 school label bbox");
    let hwpx_school =
        text_bbox_in_tree(&hwpx_tree, " 협성고등학교").expect("HWPX page 1 school label bbox");

    assert!(
        (hwp3_school.1 - hwpx_school.1).abs() < 1.0,
        "HWP3 page 1 school label y must match HWPX reference: hwp3={hwp3_school:?}, hwpx={hwpx_school:?}"
    );
}

#[test]
fn issue_1692_so_sueop_hwp3_page22_relationship_box_uses_table_flow() {
    let hwp3_model = load("samples/SO-SUEOP.hwp");
    let para = &hwp3_model.sections[0].paragraphs[574];
    let control_positions = para.control_text_positions();
    let expected_endnote_pos = para
        .text
        .find("가문의 영예(")
        .map(|byte_pos| para.text[..byte_pos].chars().count() + "가문의 영예(".chars().count() + 5)
        .expect("HWP3 p22 first explanation must contain endnote anchor text");
    assert_eq!(
        control_positions.get(1).copied(),
        Some(expected_endnote_pos),
        "HWP3 p22 endnote 118 marker must be anchored inside the parentheses"
    );
    assert!(
        matches!(para.controls.first(), Some(Control::Table(_))),
        "HWP3 obj_type=1 relationship box must remain a 1x1 table so TopAndBottom flow is reserved"
    );
    let Control::Table(table) = para.controls.first().expect("HWP3 p22 relationship table") else {
        unreachable!();
    };
    let cell = table
        .cells
        .first()
        .expect("HWP3 p22 relationship table cell");
    assert!(
        cell.paragraphs[0]
            .text
            .contains("① 윤직원\u{F081A}\u{F081A}"),
        "HWP3 p22 relationship diagram must restore circled number 1 and horizontal connectors"
    );
    assert!(
        cell.paragraphs[0].text.contains("② 윤창식\u{F0811}")
            && cell.paragraphs[0].text.contains("③ 윤종수")
            && cell.paragraphs[0].text.contains("⑤ 윤경손"),
        "HWP3 p22 relationship diagram must restore the first-line family labels"
    );
    assert!(
        cell.paragraphs[1].text.contains("\u{F0817}\u{F081A}")
            && cell.paragraphs[1].text.contains("④ 윤종학"),
        "HWP3 p22 relationship diagram must restore the lower branch to 윤종학"
    );

    let hwp3_doc = load_wasm_doc("samples/SO-SUEOP.hwp");
    let hwpx_doc = load_wasm_doc("samples/SO-SUEOP.hwpx");
    let hwp3_tree = page_render_tree(&hwp3_doc, 21);
    let hwpx_tree = page_render_tree(&hwpx_doc, 21);

    let hwp3_table = first_bbox_by_type(&hwp3_tree, "Table")
        .expect("HWP3 page 22 relationship diagram table bbox");
    let hwp3_body = text_bbox_containing_in_tree(&hwp3_tree, "윤두꺼비 시절 부친 말대가리")
        .expect("HWP3 page 22 first body line bbox");
    let hwpx_body = text_bbox_containing_in_tree(&hwpx_tree, "윤두꺼비 시절 부친 말대가리")
        .expect("HWPX page 22 first body line bbox");
    let hwp3_table_bottom = hwp3_table.1 + hwp3_table.3;
    // [Task #1841] 관계도 표(자리차지, outer_margin_bottom=852HU=11.36px) 아래 본문은
    // 표 하단 + 바깥 여백 bottom 에서 시작한다. 권위 PDF(pdf/SO-SUEOP-2024.pdf p22)
    // 실측: 표 하단 경계 246.0pt → 본문 첫 줄 상단 ≈255.6pt (gap ≈9.6pt ≈ om_bottom).
    // 종전 "+1.5px 이내" 핀은 om_bottom 누락 렌더의 보상값이었다. 본 assert 의 목적
    // (본문이 표 아래 flow 로 예약되는지)은 불변.
    let om_bottom_px = 852.0 / 7200.0 * 96.0; // 11.36px
    assert!(
        hwp3_body.1 >= hwp3_table_bottom + om_bottom_px - 1.5
            && hwp3_body.1 <= hwp3_table_bottom + om_bottom_px + 1.5,
        "HWP3 p22 body y={} must start at relationship table bottom={} + outer_margin_bottom={:.2}",
        hwp3_body.1,
        hwp3_table_bottom,
        om_bottom_px
    );
    assert!(
        (hwp3_body.1 - hwpx_body.1).abs() <= 1.0,
        "HWP3 p22 first body y={} must match HWPX y={}",
        hwp3_body.1,
        hwpx_body.1
    );
    let hwp3_body_text = text_concat_in_tree(&hwp3_tree, "Body");
    for expected in [
        "① ․윤두꺼비",
        "② 상훈은유학까지",
        "③ 종수는 할아버지가",
        "④ 종학은 이 작품에서",
        "⑤ 경손은 할아버지가",
        "경어체  → 풍자의 효과",
        "사실주의의 효과 → 고발",
    ] {
        assert!(
            hwp3_body_text.contains(expected),
            "HWP3 page 22 body text must contain {expected:?}"
        );
    }

    let hwp3_follow = text_bbox_in_tree(&hwp3_tree, " 그와 유사한 인물은?")
        .expect("HWP3 page 22 follow-up question bbox");
    // [Task #1841] 기준값 재측정: SO-SUEOP-2024.pdf p22 를 PyMuPDF line bbox 로
    // 재측정한 y0=366.72pt(=489.0px). 종전 pdftotext -bbox-layout yMin=359.796pt
    // (479.7px) 핀은 좌표 관례 차이로, 자리차지 표 outer_margin_bottom 누락 시절의
    // 렌더(479.3px)와 우연히 일치했던 값이다. om_bottom 반영 후 본문 첫 줄도
    // 권위 PDF y0=337.8px 와 0.8px 정합 (종전 325.6px = 12px 상향 오차).
    let pdf_follow_y = 366.72 * 96.0 / 72.0;
    assert!(
        (hwp3_follow.1 - pdf_follow_y).abs() <= 2.5,
        "HWP3 follow-up question y={} must match PDF y={}",
        hwp3_follow.1,
        pdf_follow_y
    );
}

#[test]
fn issue_1692_so_sueop_hwp3_endnote_internal_vpos_zero_is_normalized() {
    let hwp3_doc = load("samples/SO-SUEOP.hwp");

    let mut hwp3_endnotes = Vec::new();
    collect_endnotes(&hwp3_doc.sections[0].paragraphs, &mut hwp3_endnotes);

    let endnote_22 = hwp3_endnotes
        .iter()
        .find(|endnote| endnote.number == 22)
        .expect("HWP3 endnote 22");
    let line_vpos: Vec<i32> = endnote_22.paragraphs[0]
        .line_segs
        .iter()
        .map(|seg| seg.vertical_pos)
        .collect();

    assert_eq!(
        line_vpos,
        vec![0, 960, 1920, 2880],
        "HWP3 note-internal line vpos=0 must be normalized as a continuation line"
    );
}

#[test]
fn issue_1692_so_sueop_hwpx_endnote_internal_vpos_zero_is_normalized() {
    let hwpx_doc = load("samples/SO-SUEOP.hwpx");

    let mut hwpx_endnotes = Vec::new();
    collect_endnotes(&hwpx_doc.sections[0].paragraphs, &mut hwpx_endnotes);

    let endnote_161 = hwpx_endnotes
        .iter()
        .find(|endnote| endnote.number == 161)
        .expect("HWPX endnote 161");
    let para = &endnote_161.paragraphs[0];
    assert_eq!(para.line_segs.len(), 2);

    let first = &para.line_segs[0];
    let second = &para.line_segs[1];
    assert_eq!(
        second.vertical_pos,
        first
            .vertical_pos
            .saturating_add(first.line_height)
            .saturating_add(first.line_spacing),
        "HWPX note-internal line vpos=0 must be normalized as a continuation line"
    );
}
