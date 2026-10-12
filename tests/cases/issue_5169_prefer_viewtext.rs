//! #5169/#7698: ViewText contains review history, not just visible content.
//!
//! The unchanged public HWP5 source has ten raw tables. Independent Hancom
//! Print (six pages) shows four; ordered TRACK_CHANGE hide metadata and ranges
//! select visible content before layout. The original history must still survive
//! an unedited save. Evidence: mydocs/pr/pr_7698_review_impl.md (Native/fresh
//! Docker WASM all-page minimum 91.69%, and identical original/saved Print).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::paragraph::Paragraph;

const SAMPLE: &str = "samples/issue5169_viewtext_changetracking.hwp";

fn source() -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE))
        .unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"))
}

fn count_tables(paras: &[Paragraph]) -> usize {
    paras
        .iter()
        .map(|p| {
            p.controls
                .iter()
                .map(|c| match c {
                    Control::Table(t) => {
                        1 + t
                            .cells
                            .iter()
                            .map(|cell| count_tables(&cell.paragraphs))
                            .sum::<usize>()
                    }
                    _ => 0,
                })
                .sum::<usize>()
        })
        .sum()
}

fn text(paras: &[Paragraph], result: &mut String) {
    for p in paras {
        result.push_str(&p.text);
        result.push('\n');
        for c in &p.controls {
            if let Control::Table(t) = c {
                for cell in &t.cells {
                    text(&cell.paragraphs, result);
                }
            }
        }
    }
}

fn visible_content(doc: &Document) -> String {
    let mut result = String::new();
    for section in &doc.sections {
        text(&section.paragraphs, &mut result);
    }
    result
}

fn assert_visible_content(doc: &Document) {
    assert_eq!(
        doc.sections
            .iter()
            .map(|s| count_tables(&s.paragraphs))
            .sum::<usize>(),
        4,
        "hidden review-history controls must not enter visible IR"
    );
    let content = visible_content(doc);
    assert!(content.contains("연구계획서"));
    assert!(content.contains("2018. 6"));
    assert!(
        !content.contains("연구제안계획서"),
        "partially hidden deletion"
    );
    assert!(
        !content.contains("무역구제제도분야 협상 관련"),
        "fully hidden paragraph and terminator"
    );
}

#[test]
fn viewtext_applies_saved_change_visibility_before_layout() {
    assert_visible_content(&rhwp::parser::parse_document(&source()).expect("parse"));
}

#[test]
fn unedited_save_preserves_review_stream_and_visible_content() {
    let before = rhwp::parser::parse_document(&source()).expect("parse");
    let saved = rhwp::serializer::serialize_document(&before).expect("save");
    let mut cfb = cfb::CompoundFile::open(std::io::Cursor::new(saved.clone())).expect("CFB");
    assert!(cfb.open_stream("/BodyText/Section0").is_ok());
    assert!(
        cfb.open_stream("/ViewText/Section0").is_ok(),
        "Hancom needs the review stream as ViewText"
    );
    let after = rhwp::parser::parse_document(&saved).expect("reopen");
    assert_visible_content(&after);
    assert_eq!(visible_content(&before), visible_content(&after));
    assert_eq!(before.sections.len(), after.sections.len());
    for (original, reopened) in before.sections.iter().zip(&after.sections) {
        assert!(original.raw_stream.is_some());
        assert_eq!(
            original.raw_stream, reopened.raw_stream,
            "raw review history is retained"
        );
    }
}

#[test]
fn unaffected_visible_paragraphs_keep_valid_saved_line_boundaries() {
    let doc = rhwp::parser::parse_document(&source()).expect("parse");
    // Original ViewText LineSeg starts independently match Hancom's Print
    // wrapping. Neither paragraph has hidden tracking ranges.
    for (prefix, starts) in [("따라서", [0, 39]), ("고위관리", [0, 56])] {
        let p = doc
            .sections
            .iter()
            .flat_map(|s| &s.paragraphs)
            .find(|p| {
                p.text
                    .trim_start_matches([' ', 'o'])
                    .trim_start()
                    .starts_with(prefix)
            })
            .unwrap_or_else(|| panic!("missing paragraph: {prefix}"));
        assert_eq!(
            p.line_segs
                .iter()
                .map(|line| line.text_start)
                .collect::<Vec<_>>(),
            starts
        );
    }
}

// The HWPX is an independent Hancom 2020 conversion of the same public
// HWP input. Its six-page Print, Native and fresh Docker WASM all-page minima
// exceed 90%; XML inter-element spacing has the same independent Print oracle.
fn hwpx_source() -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "tests/fixtures/issue5169_tracked_visibility/issue5169_viewtext_changetracking.hwpx",
        ),
    )
    .expect("public Hancom-converted HWPX")
}

#[test]
fn hwpx_applies_saved_change_visibility_to_text_and_owned_objects() {
    let doc = rhwp::parser::parse_document(&hwpx_source()).expect("HWPX parse");
    assert_visible_content(&doc);
}

fn with_interparagraph_xml_spacing(bytes: &[u8]) -> Vec<u8> {
    use std::io::{Cursor, Read, Write};
    let mut source = zip::ZipArchive::new(Cursor::new(bytes)).expect("ZIP");
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..source.len() {
        let mut entry = source.by_index(index).expect("ZIP entry");
        let name = entry.name().expect("ZIP 엔트리 이름").into_owned();
        let method = entry.compression();
        let mut contents = Vec::new();
        entry.read_to_end(&mut contents).expect("entry bytes");
        if name == "Contents/section0.xml" {
            contents = String::from_utf8(contents)
                .expect("section XML")
                .replace("</hp:p><hp:p", "</hp:p>\n<hp:p")
                .into_bytes();
        }
        output
            .start_file(
                name,
                zip::write::SimpleFileOptions::default().compression_method(method),
            )
            .expect("write entry");
        output.write_all(&contents).expect("entry contents");
    }
    output.finish().expect("ZIP finish").into_inner()
}

#[test]
fn hwpx_xml_formatting_keeps_owned_paragraph_merges_and_visible_content() {
    let source = hwpx_source();
    let spaced = with_interparagraph_xml_spacing(&source);
    let before = rhwp::parser::parse_document(&source).expect("original");
    let after = rhwp::parser::parse_document(&spaced).expect("formatted XML");
    assert_visible_content(&after);
    assert_eq!(visible_content(&before), visible_content(&after));
    assert_eq!(before.sections.len(), after.sections.len());
    for (original, formatted) in before.sections.iter().zip(&after.sections) {
        assert_eq!(
            original.paragraphs.len(),
            formatted.paragraphs.len(),
            "inter-element XML whitespace must not change paragraph ownership"
        );
    }
    assert_eq!(
        rhwp::document_core::DocumentCore::from_bytes(&spaced)
            .expect("formatted layout")
            .page_count(),
        6
    );
}

fn paragraph_line_boxes(
    node: &rhwp::renderer::render_tree::RenderNode,
    para: usize,
    out: &mut Vec<(f64, f64)>,
) {
    use rhwp::renderer::render_tree::RenderNodeType;
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if line.section_index == Some(0) && line.para_index == Some(para) {
            out.push((node.bbox.y, node.bbox.height));
        }
    }
    for child in &node.children {
        paragraph_line_boxes(child, para, out);
    }
}

#[test]
fn hwpx_saved_leading_empty_lines_and_heading_share_the_new_page() {
    let core = rhwp::document_core::DocumentCore::from_bytes(&hwpx_source()).expect("HWPX layout");
    assert_eq!(core.page_count(), 6, "independent Hancom Print");
    let paras = &core.document().sections[0].paragraphs;
    let heading = paras
        .iter()
        .position(|p| p.text.contains("연구의 필요성 및 목적"))
        .expect("first body heading");
    assert!(heading >= 2);
    assert!(paras[heading - 2].text.trim().is_empty());
    assert!(paras[heading - 1].text.trim().is_empty());
    let first = core.build_page_render_tree(0).expect("cover");
    let second = core.build_page_render_tree(1).expect("first body page");
    let mut previous_page = Vec::new();
    paragraph_line_boxes(&first.root, heading - 2, &mut previous_page);
    assert!(
        previous_page.is_empty(),
        "leading blank line must not stay on cover"
    );
    let mut blank = Vec::new();
    let mut host = Vec::new();
    let mut title = Vec::new();
    paragraph_line_boxes(&second.root, heading - 2, &mut blank);
    paragraph_line_boxes(&second.root, heading - 1, &mut host);
    paragraph_line_boxes(&second.root, heading, &mut title);
    assert_eq!(blank.len(), 1);
    assert_eq!(host.len(), 1);
    assert!(!title.is_empty());
    assert!(blank[0].0 + blank[0].1 <= host[0].0);
    assert!(host[0].0 + host[0].1 <= title[0].0);
}
