//! One generated PDF: disjoint text objects with a word gap, contiguous text,
//! and negative TJ kerning. Independent Poppler / Native / fresh Canvas = 100%.
#![cfg(feature = "pdf-import")]
use rhwp::{
    model::{control::Control, document::Document, paragraph::Paragraph, shape::ShapeObject},
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

const INPUT: &[u8] = include_bytes!("../../pdf/issue7958/positioned-word-gap.pdf");
const EXPECTED: [&str; 3] = ["가나 다라", "가나다라", "가나다라"];

fn check_document(document: &Document) {
    assert_eq!(document.sections.len(), 1);
    let paragraphs: Vec<&Paragraph> = document.sections[0]
        .paragraphs
        .iter()
        .flat_map(|p| &p.controls)
        .filter_map(|c| match c {
            Control::Shape(s) => match s.as_ref() {
                ShapeObject::Rectangle(r) => r.drawing.text_box.as_ref(),
                _ => None,
            },
            _ => None,
        })
        .flat_map(|b| &b.paragraphs)
        .collect();
    assert_eq!(
        paragraphs.len(),
        EXPECTED.len(),
        "preserve the three source lines"
    );
    for (paragraph, expected) in paragraphs.iter().zip(EXPECTED) {
        assert_eq!(
            paragraph.text, expected,
            "recover the geometric word gap only"
        );
        assert_eq!(
            paragraph.char_count,
            expected.encode_utf16().count() as u32 + 1
        );
        assert_eq!(
            paragraph
                .text
                .chars()
                .filter(|c| !c.is_whitespace())
                .count(),
            4,
            "no source glyph is lost or duplicated"
        );
    }
}

fn text(node: &RenderNode) -> String {
    let mut result = match &node.node_type {
        RenderNodeType::TextRun(run) => run.display_text.as_ref().unwrap_or(&run.text).clone(),
        _ => String::new(),
    };
    for child in &node.children {
        result.push_str(&text(child));
    }
    result
}

fn lines(node: &RenderNode, output: &mut Vec<(String, f64, f64, f64)>) {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        let content = text(node);
        if !content.is_empty() {
            output.push((content, node.bbox.y, node.bbox.height, node.bbox.width));
        }
    }
    for child in &node.children {
        lines(child, output);
    }
}

fn check_layout(core: &DocumentCore) {
    assert_eq!(core.page_count(), 1);
    let tree = core.build_page_render_tree(0).unwrap();
    let mut rendered = Vec::new();
    lines(&tree.root, &mut rendered);
    assert_eq!(
        rendered.iter().map(|l| l.0.as_str()).collect::<Vec<_>>(),
        EXPECTED,
        "the final text lines must consume the restored text in source order"
    );
    for pair in rendered.windows(2) {
        assert!(
            pair[0].1 + pair[0].2 <= pair[1].1,
            "source lines must remain separate"
        );
    }
    // Source-defined positive gap / no gap / negative kerning, no px pins.
    assert!(rendered[0].3 > rendered[1].3 && rendered[1].3 > rendered[2].3);
}

#[test]
fn positioned_word_gap_reaches_final_lines_and_both_saved_formats() {
    let core = DocumentCore::from_pdf_bytes(INPUT, &Default::default()).unwrap();
    check_document(core.document());
    check_layout(&core);
    for saved in [
        core.export_hwpx_native().unwrap(),
        core.export_hwp_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&saved).unwrap();
        check_document(reopened.document());
        check_layout(&reopened);
    }
}
